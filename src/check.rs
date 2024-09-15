use std::collections::{HashMap, HashSet};

use json::JsonValue;

use crate::{
    docker::get_images_from_docker_daemon,
    image::Image,
    registry::{check_auth, get_latest_digests, get_token},
    utils::{new_reqwest_client, unsplit_image},
};

#[cfg(feature = "cli")]
use crate::docker::get_image_from_docker_daemon;
#[cfg(feature = "cli")]
use crate::registry::get_latest_digest;

pub trait Unique<T> {
    // So we can filter vecs for duplicates
    fn unique(&mut self);
}

impl<T> Unique<T> for Vec<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    fn unique(self: &mut Vec<T>) {
        let mut seen: HashSet<T> = HashSet::new();
        self.retain(|item| seen.insert(item.clone()));
    }
}

pub async fn get_all_updates(
    socket: Option<String>,
    config: &JsonValue,
) -> Vec<(String, Option<bool>)> {
    let local_images = get_images_from_docker_daemon(socket).await;
    let mut image_map: HashMap<String, Option<String>> = HashMap::with_capacity(local_images.len());
    for image in &local_images {
        let img = unsplit_image(&image.registry, &image.repository, &image.tag);
        image_map.insert(img, image.digest.clone());
    };
    let mut registries: Vec<&String> = local_images
        .iter()
        .map(|image| &image.registry)
        .collect();
    registries.unique();
    let mut remote_images: Vec<Image> = Vec::with_capacity(local_images.len());
    let client = new_reqwest_client();
    for registry in registries {
        let images: Vec<&Image> = local_images
            .iter()
            .filter(|image| &image.registry == registry)
            .collect();
        let credentials = config["authentication"][registry]
            .clone()
            .take_string()
            .or(None);
        let mut latest_images = match check_auth(registry, config, &client).await {
            Some(auth_url) => {
                let token = get_token(images.clone(), &auth_url, &credentials, &client).await;
                get_latest_digests(images, Some(&token), config, &client).await
            }
            None => get_latest_digests(images, None, config, &client).await,
        };
        remote_images.append(&mut latest_images);
    }
    let mut result: Vec<(String, Option<bool>)> = Vec::new();
    remote_images.iter().for_each(|image| {
        let img = unsplit_image(&image.registry, &image.repository, &image.tag);
        match &image.digest {
            Some(d) => {
                let r = d != image_map.get(&img).unwrap().as_ref().unwrap();
                result.push((img, Some(r)))
            }
            None => result.push((img, None)),
        }
    });
    result
}

#[cfg(feature = "cli")]
pub async fn get_update(image: &str, socket: Option<String>, config: &JsonValue) -> Option<bool> {
    let local_image = get_image_from_docker_daemon(socket, image).await;
    let credentials = config["authentication"][&local_image.registry]
        .clone()
        .take_string()
        .or(None);
    let client = new_reqwest_client();
    let token = match check_auth(&local_image.registry, config, &client).await {
        Some(auth_url) => get_token(vec![&local_image], &auth_url, &credentials, &client).await,
        None => String::new(),
    };
    let remote_image = match token.as_str() {
        "" => get_latest_digest(&local_image, None, config, &client).await,
        _ => get_latest_digest(&local_image, Some(&token), config, &client).await,
    };
    match &remote_image.digest {
        Some(d) => Some(d != &local_image.digest.unwrap()),
        None => None,
    }
}
