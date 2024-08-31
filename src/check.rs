use std::{collections::{HashMap, HashSet}, sync::Mutex};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use json::JsonValue;

use crate::{docker::get_images_from_docker_daemon, image::Image, registry::{check_auth, get_token, get_latest_digests}, utils::unsplit_image};

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

pub async fn get_all_updates(socket: Option<String>, config: &JsonValue) -> Vec<(String, Option<bool>)> {
    let image_map_mutex: Mutex<HashMap<String, &Option<String>>> = Mutex::new(HashMap::new());
    let local_images = get_images_from_docker_daemon(socket).await;
    local_images.par_iter().for_each(|image| {
        let img = unsplit_image(&image.registry, &image.repository, &image.tag);
        image_map_mutex.lock().unwrap().insert(img, &image.digest);
    });
    let image_map = image_map_mutex.lock().unwrap().clone();
    let mut registries: Vec<&String> = local_images
        .par_iter()
        .map(|image| &image.registry)
        .collect();
    registries.unique();
    let mut remote_images: Vec<Image> = Vec::new();
    for registry in registries {
        let images: Vec<&Image> = local_images
            .par_iter()
            .filter(|image| &image.registry == registry)
            .collect();
        let credentials = config["authentication"][registry].clone().take_string().or(None);
        let mut latest_images = match check_auth(registry, config) {
            Some(auth_url) => {
                let token = get_token(images.clone(), &auth_url, &credentials);
                get_latest_digests(images, Some(&token), config)
            }
            None => get_latest_digests(images, None, config),
        };
        remote_images.append(&mut latest_images);
    }
    let result_mutex: Mutex<Vec<(String, Option<bool>)>> = Mutex::new(Vec::new());
    remote_images.par_iter().for_each(|image| {
        let img = unsplit_image(&image.registry, &image.repository, &image.tag);
        match &image.digest {
            Some(d) => {
                let r = d != image_map.get(&img).unwrap().as_ref().unwrap();
                result_mutex.lock().unwrap().push((img, Some(r)))
            }
            None => result_mutex.lock().unwrap().push((img, None)),
        }
    });
    let result = result_mutex.lock().unwrap().clone();
    result
}

#[cfg(feature = "cli")]
pub async fn get_update(image: &str, socket: Option<String>, config: &JsonValue) -> Option<bool> {
    let local_image = get_image_from_docker_daemon(socket, image).await;
    let credentials = config["authentication"][&local_image.registry].clone().take_string().or(None);
    let token = match check_auth(&local_image.registry, config) {
        Some(auth_url) => get_token(vec![&local_image], &auth_url, &credentials),
        None => String::new(),
    };
    let remote_image = match token.as_str() {
        "" => get_latest_digest(&local_image, None, config),
        _ => get_latest_digest(&local_image, Some(&token), config),
    };
    match &remote_image.digest {
        Some(d) => Some(d != &local_image.digest.unwrap()),
        None => None,
    }
}