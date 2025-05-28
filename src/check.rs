use futures::future::join_all;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    docker::{get_images_from_docker_daemon, get_in_use_images},
    http::Client,
    registry::{check_auth, get_token},
    structs::{image::Image, update::Update},
    utils::request::{get_response_body, parse_json},
    Context,
};

/// Fetches image data from other Cup instances
async fn get_remote_updates(ctx: &Context, client: &Client, refresh: bool) -> Vec<Update> {
    let mut remote_images = Vec::new();

    let handles: Vec<_> = ctx.config.servers
        .iter()
        .map(|(name, url)| async move {
            let base_url = if url.starts_with("http://") || url.starts_with("https://") {
                format!("{}/api/v3/", url.trim_end_matches('/'))
            } else {
                format!("https://{}/api/v3/", url.trim_end_matches('/'))
            };
            let json_url = base_url.clone() + "json";
            if refresh {
                let refresh_url = base_url + "refresh";
                match client.get(&refresh_url, &[], false).await {
                    Ok(response) => {
                        if response.status() != 200 {
                            ctx.logger.warn(format!("GET {}: Failed to refresh server. Server returned invalid response code: {}", refresh_url, response.status()));
                            return Vec::new();
                        }
                    },
                    Err(e) => {
                        ctx.logger.warn(format!("GET {}: Failed to refresh server. {}", refresh_url, e));
                        return Vec::new();
                    },
                }

            }
            match client.get(&json_url, &[], false).await {
                Ok(response) => {
                    if response.status() != 200 {
                        ctx.logger.warn(format!("GET {}: Failed to fetch updates from server. Server returned invalid response code: {}", json_url, response.status()));
                        return Vec::new();
                    }
                    let json = parse_json(&get_response_body(response).await);
                    ctx.logger.debug(format!("JSON response for {}: {}", name, json));
                    if let Some(updates) = json["images"].as_array() {
                        let mut server_updates: Vec<Update> = updates
                            .iter()
                            .filter_map(|img| serde_json::from_value(img.clone()).ok())
                            .collect();
                        // Add server origin to each image
                        for update in &mut server_updates {
                            update.server = Some(name.clone());
                            update.status = update.get_status();
                        }
                        ctx.logger.debug(format!("Updates for {}: {:#?}", name, server_updates));
                        return server_updates;
                    }

                    Vec::new()
                }
                Err(e) => {
                    ctx.logger.warn(format!("GET {}: Failed to fetch updates from server. {}", json_url, e));
                    Vec::new()
                },
            }
        })
        .collect();

    for mut images in join_all(handles).await {
        remote_images.append(&mut images);
    }

    remote_images
}

/// Returns a list of updates for all images passed in.
pub async fn get_updates(
    references: &Option<Vec<String>>, // If a user requested _specific_ references to be checked, this will have a value
    refresh: bool,
    ctx: &Context,
) -> Vec<Update> {
    let client = Client::new(ctx);

    // Merge references argument with references from config
    let all_references = match &references {
        Some(refs) => {
            if !ctx.config.images.extra.is_empty() {
                refs.clone().extend_from_slice(&ctx.config.images.extra);
            }
            refs
        }
        None => &ctx.config.images.extra,
    };

    // Get local images
    ctx.logger.debug("Retrieving images to be checked");
    let mut images = get_images_from_docker_daemon(ctx, references).await;
    let in_use_images = get_in_use_images(ctx).await;
    ctx.logger
        .debug(format!("Found {} images in use", in_use_images.len()));

    // Complete in_use field
    images.iter_mut().for_each(|image| {
        if let Some(images) = in_use_images.get(&image.reference) {
            image.used_by = images.clone()
        }
    });

    // Add extra images from references
    if !all_references.is_empty() {
        let image_refs: FxHashSet<&String> = images.iter().map(|image| &image.reference).collect();
        let extra = all_references
            .iter()
            .filter(|&reference| !image_refs.contains(reference))
            .map(|reference| Image::from_reference(reference))
            .collect::<Vec<Image>>();
        images.extend(extra);
    }

    // Get remote images from other servers
    let remote_updates = if !ctx.config.servers.is_empty() {
        ctx.logger.debug("Fetching updates from remote servers");
        get_remote_updates(ctx, &client, refresh).await
    } else {
        Vec::new()
    };

    ctx.logger.debug(format!(
        "Checking {:?}",
        images.iter().map(|image| &image.reference).collect_vec()
    ));

    // Get a list of unique registries our images belong to. We are unwrapping the registry because it's guaranteed to be there.
    let registries: Vec<&String> = images
        .iter()
        .map(|image| &image.parts.registry)
        .unique()
        .filter(|&registry| match ctx.config.registries.get(registry) {
            Some(config) => !config.ignore,
            None => true,
        })
        .collect::<Vec<&String>>();

    // Create request client. All network requests share the same client for better performance.
    // This client is also configured to retry a failed request up to 3 times with exponential backoff in between.
    let client = Client::new(ctx);

    // Create a map of images indexed by registry. This solution seems quite inefficient, since each iteration causes a key to be looked up. I can't find anything better at the moment.
    let mut image_map: FxHashMap<&String, Vec<&Image>> = FxHashMap::default();

    for image in &images {
        image_map
            .entry(&image.parts.registry)
            .or_default()
            .push(image);
    }

    // Retrieve an authentication token (if required) for each registry.
    let mut tokens: FxHashMap<&str, Option<String>> = FxHashMap::default();
    for registry in registries.clone() {
        let credentials = if let Some(registry_config) = ctx.config.registries.get(registry) {
            &registry_config.authentication
        } else {
            &None
        };
        match check_auth(registry, ctx, &client).await {
            Some(auth_url) => {
                let token = get_token(
                    image_map.get(registry).unwrap(),
                    &auth_url,
                    credentials,
                    &client,
                )
                .await;
                tokens.insert(registry, Some(token));
            }
            None => {
                tokens.insert(registry, None);
            }
        }
    }

    ctx.logger.debug(format!("Tokens: {:?}", tokens));

    let mut handles = Vec::with_capacity(images.len());

    // Loop through images check for updates
    for image in &images {
        let is_ignored = !registries.contains(&&image.parts.registry)
            || ctx
                .config
                .images
                .exclude
                .iter()
                .any(|item| image.reference.starts_with(item));
        if !is_ignored {
            let token = tokens.get(image.parts.registry.as_str()).unwrap();
            let future = image.check(token.as_deref(), ctx, &client);
            handles.push(future);
        }
    }
    // Await all the futures
    let images = join_all(handles).await;
    let mut updates: Vec<Update> = images.iter().map(|image| image.to_update()).collect();
    updates.extend_from_slice(&remote_updates);
    updates
}
