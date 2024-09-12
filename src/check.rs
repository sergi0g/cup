use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use chrono::Local;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    debug,
    docker::get_images_from_docker_daemon,
    image::Image,
    info,
    registry::{check_auth, get_latest_digests, get_token},
    utils::{unsplit_image, CliConfig},
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

pub async fn get_all_updates(options: &CliConfig) -> Vec<(String, Option<bool>)> {
    let start = Local::now().timestamp_millis();
    let image_map_mutex: Mutex<HashMap<String, &Option<String>>> = Mutex::new(HashMap::new());
    let local_images = get_images_from_docker_daemon(options).await;
    local_images.par_iter().for_each(|image| {
        let img = unsplit_image(image);
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
        if options.verbose {
            debug!("Checking images from registry {}", registry)
        }
        let images: Vec<&Image> = local_images
            .par_iter()
            .filter(|image| &image.registry == registry)
            .collect();
        let credentials = options.config["authentication"][registry]
            .clone()
            .take_string()
            .or(None);
        let mut latest_images = match check_auth(registry, options) {
            Some(auth_url) => {
                let token = get_token(images.clone(), &auth_url, &credentials, options);
                if options.verbose {
                    debug!("Using token {}", token);
                }
                get_latest_digests(images, Some(&token), options)
            }
            None => get_latest_digests(images, None, options),
        };
        remote_images.append(&mut latest_images);
    }
    if options.verbose {
        debug!("Collecting results")
    }
    let result_mutex: Mutex<Vec<(String, Option<bool>)>> = Mutex::new(Vec::new());
    remote_images.par_iter().for_each(|image| {
        let img = unsplit_image(image);
        match &image.digest {
            Some(d) => {
                let r = d != image_map.get(&img).unwrap().as_ref().unwrap();
                result_mutex.lock().unwrap().push((img, Some(r)))
            }
            None => result_mutex.lock().unwrap().push((img, None)),
        }
    });
    let result = result_mutex.lock().unwrap().clone();
    let end = Local::now().timestamp_millis();
    info!("✨ Checked {} images in {}ms", local_images.len(), end - start);
    result
}

#[cfg(feature = "cli")]
pub async fn get_update(image: &str, options: &CliConfig) -> Option<bool> {
    let local_image = get_image_from_docker_daemon(options.socket.clone(), image).await;
    let credentials = options.config["authentication"][&local_image.registry]
        .clone()
        .take_string()
        .or(None);
    let token = match check_auth(&local_image.registry, options) {
        Some(auth_url) => get_token(vec![&local_image], &auth_url, &credentials, options),
        None => String::new(),
    };
    if options.verbose {
        debug!("Using token {}", token);
    };
    let remote_image = match token.as_str() {
        "" => get_latest_digest(&local_image, None, options),
        _ => get_latest_digest(&local_image, Some(&token), options),
    };
    match &remote_image.digest {
        Some(d) => Some(d != &local_image.digest.unwrap()),
        None => None,
    }
}
