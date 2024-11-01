use futures::future::join_all;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    config::Config,
    image::Image,
    registry::{check_auth, get_token},
    utils::new_reqwest_client,
};

/// Trait for a type that implements a function `unique` that removes any duplicates.
/// In this case, it will be used for a Vec.
pub trait Unique<T> {
    fn unique(&mut self) -> Vec<T>;
}

impl<T> Unique<T> for Vec<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    /// Remove duplicates from Vec
    fn unique(self: &mut Vec<T>) -> Self {
        let mut seen: FxHashSet<T> = FxHashSet::default();
        self.retain(|item| seen.insert(item.clone()));
        self.to_vec()
    }
}

/// Returns a list of updates for all images passed in.
pub async fn get_updates(images: &[Image], config: &Config) -> Vec<Image> {
    // Get a list of unique registries our images belong to. We are unwrapping the registry because it's guaranteed to be there.
    let registries: Vec<&String> = images
        .iter()
        .map(|image| image.registry.as_ref().unwrap())
        .collect::<Vec<&String>>()
        .unique();

    // Create request client. All network requests share the same client for better performance.
    // This client is also configured to retry a failed request up to 3 times with exponential backoff in between.
    let client = new_reqwest_client();

    // Create a map of images indexed by registry. This solution seems quite inefficient, since each iteration causes a key to be looked up. I can't find anything better at the moment.
    let mut image_map: FxHashMap<&String, Vec<&Image>> = FxHashMap::default();

    for image in images {
        image_map
            .entry(image.registry.as_ref().unwrap())
            .or_default()
            .push(image);
    }

    // Retrieve an authentication token (if required) for each registry.
    let mut tokens: FxHashMap<&String, Option<String>> = FxHashMap::default();
    for registry in registries {
        let credentials = config.authentication.get(registry);
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

    // Create a Vec to store futures so we can await them all at once.
    let mut handles = Vec::new();
    // Loop through images and get the latest digest for each
    for image in images {
        let token = tokens.get(&image.registry.as_ref().unwrap()).unwrap();
        let future = image.check(token.as_ref(), config, &client);
        handles.push(future);
    }
    // Await all the futures
    join_all(handles).await
}
