use futures::future::join_all;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    config::Config,
    debug,
    docker::get_images_from_docker_daemon,
    http::Client,
    registry::{check_auth, get_token},
    structs::image::Image,
};

/// Returns a list of updates for all images passed in.
pub async fn get_updates(references: &Option<Vec<String>>, config: &Config) -> Vec<Image> {
    // Get images
    debug!(config.debug, "Retrieving images to be checked");
    let mut images = get_images_from_docker_daemon(config, references).await;
    let extra_images = match references {
        Some(refs) => {
            let image_refs: FxHashSet<&String> =
                images.iter().map(|image| &image.reference).collect();
            let extra = refs
                .iter()
                .filter(|&reference| !image_refs.contains(reference))
                .collect::<Vec<&String>>();
            Some(
                extra
                    .iter()
                    .map(|reference| Image::from_reference(reference))
                    .collect::<Vec<Image>>(),
            )
        }
        None => None,
    };
    if let Some(extra_imgs) = extra_images {
        images.extend_from_slice(&extra_imgs);
    }
    debug!(
        config.debug,
        "Checking {:?}",
        images.iter().map(|image| &image.reference).collect_vec()
    );

    // Get a list of unique registries our images belong to. We are unwrapping the registry because it's guaranteed to be there.
    let registries: Vec<&String> = images
        .iter()
        .map(|image| &image.registry)
        .unique()
        .collect::<Vec<&String>>();

    // Create request client. All network requests share the same client for better performance.
    // This client is also configured to retry a failed request up to 3 times with exponential backoff in between.
    let client = Client::new();

    // Create a map of images indexed by registry. This solution seems quite inefficient, since each iteration causes a key to be looked up. I can't find anything better at the moment.
    let mut image_map: FxHashMap<&String, Vec<&Image>> = FxHashMap::default();

    for image in &images {
        image_map.entry(&image.registry).or_default().push(image);
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
                    &credentials,
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

    // Create a Vec to store futures so we can await them all at once.
    let mut handles = Vec::with_capacity(images.len());

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

    // Loop through images and get the latest digest for each
    for image in &images {
        let is_ignored = ignored_registries.contains(&&image.registry) || config.images.exclude.iter().any(|item| image.reference.starts_with(item));
        if !is_ignored {
            let token = tokens.get(image.registry.as_str()).unwrap();
            let future = image.check(token.as_ref(), config, &client);
            handles.push(future);
        }
    }
    // Await all the futures
    join_all(handles).await
}
