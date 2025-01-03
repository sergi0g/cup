use futures::future::join_all;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    config::Config,
    debug,
    docker::get_images_from_docker_daemon,
    http::Client,
    registry::{check_auth, get_token},
    structs::{image::Image, update::Update},
    utils::request::{get_response_body, parse_json},
    warn,
};

/// Fetches image data from other Cup instances
async fn get_remote_updates(servers: &FxHashMap<String, String>, client: &Client) -> Vec<Update> {
    let mut remote_images = Vec::new();

    let handles: Vec<_> = servers
        .iter()
        .map(|(name, url)| async {
            let url = if url.starts_with("http://") || url.starts_with("https://") {
                format!("{}/api/v3/json", url.trim_end_matches('/'))
            } else {
                format!("https://{}/api/v3/json", url.trim_end_matches('/'))
            };
            match client.get(&url, vec![], false).await {
                Ok(response) => {
                    if response.status() != 200 {
                        warn!("GET {}: Failed to fetch updates from server. Server returned invalid response code: {}",url,response.status());
                        return Vec::new();
                    }
                    let json = parse_json(&get_response_body(response).await);
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
                        return server_updates;
                    }

                    Vec::new()
                }
                Err(e) => {
                    warn!("Failed to fetch updates from server. {}", e);
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
pub async fn get_updates(references: &Option<Vec<String>>, config: &Config) -> Vec<Update> {
    let client = Client::new();

    // Get local images
    debug!(config.debug, "Retrieving images to be checked");
    let mut images = get_images_from_docker_daemon(config, references).await;

    // Add extra images from references
    if let Some(refs) = references {
        let image_refs: FxHashSet<&String> = images.iter().map(|image| &image.reference).collect();
        let extra = refs
            .iter()
            .filter(|&reference| !image_refs.contains(reference))
            .map(|reference| Image::from_reference(reference))
            .collect::<Vec<Image>>();
        images.extend(extra);
    }

    // Get remote images from other servers
    let remote_updates = if !config.servers.is_empty() {
        debug!(config.debug, "Fetching updates from remote servers");
        get_remote_updates(&config.servers, &client).await
    } else {
        Vec::new()
    };

    debug!(
        config.debug,
        "Checking {:?}",
        images.iter().map(|image| &image.reference).collect_vec()
    );

    // Get a list of unique registries our images belong to. We are unwrapping the registry because it's guaranteed to be there.
    let registries: Vec<&String> = images
        .iter()
        .map(|image| &image.parts.registry)
        .unique()
        .collect::<Vec<&String>>();

    // Create request client. All network requests share the same client for better performance.
    // This client is also configured to retry a failed request up to 3 times with exponential backoff in between.
    let client = Client::new();

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
    for registry in registries {
        let credentials = if let Some(registry_config) = config.registries.get(registry) {
            &registry_config.authentication
        } else {
            &None
        };
        match check_auth(registry, config, &client).await {
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

    debug!(config.debug, "Tokens: {:?}", tokens);

    let ignored_registries = config
        .registries
        .iter()
        .filter_map(|(registry, registry_config)| {
            if registry_config.ignore {
                Some(registry)
            } else {
                None
            }
        })
        .collect::<Vec<&String>>();

    let mut handles = Vec::with_capacity(images.len());

    // Loop through images check for updates
    for image in &images {
        let is_ignored = ignored_registries.contains(&&image.parts.registry)
            || config
                .images
                .exclude
                .iter()
                .any(|item| image.reference.starts_with(item));
        if !is_ignored {
            let token = tokens.get(image.parts.registry.as_str()).unwrap();
            let future = image.check(token.as_deref(), config, &client);
            handles.push(future);
        }
    }
    // Await all the futures
    let images = join_all(handles).await;
    let mut updates: Vec<Update> = images.iter().map(|image| image.to_update()).collect();
    updates.extend_from_slice(&remote_updates);
    updates
}
