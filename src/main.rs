use clap::{Parser, Subcommand};
#[cfg(feature = "cli")]
use docker::get_image_from_docker_daemon;
use docker::get_images_from_docker_daemon;
#[cfg(feature = "cli")]
use formatting::{print_raw_update, print_raw_updates, print_update, print_updates, Spinner};
use image::Image;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
#[cfg(feature = "cli")]
use registry::get_latest_digest;
use registry::{check_auth, get_latest_digests, get_token};
#[cfg(feature = "server")]
use server::serve;
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};
use utils::unsplit_image;

pub mod docker;
#[cfg(feature = "cli")]
pub mod formatting;
pub mod image;
pub mod registry;
#[cfg(feature = "server")]
pub mod server;
pub mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = None)]
    socket: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "cli")]
    Check {
        #[arg(default_value = None)]
        image: Option<String>,
        #[arg(short, long, default_value_t = false, help = "Enable icons")]
        icons: bool,
        #[arg(short, long, default_value_t = false, help = "Output JSON instead of formatted text")]
        raw: bool,
    },
    #[cfg(feature = "server")]
    Serve {
        #[arg(short, long, default_value_t = 8000, help = "Use a different port for the server")]
        port: u16,
    },
}

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.command {
        #[cfg(feature = "cli")]
        Some(Commands::Check { image, icons, raw }) => match image {
            Some(name) => {
                let has_update = get_update(name, cli.socket).await;
                match raw {
                    true => print_raw_update(name, &has_update),
                    false => print_update(name, &has_update),
                };
            }
            None => {
                match raw {
                    true => print_raw_updates(&get_all_updates(cli.socket).await),
                    false => {
                        let spinner = Spinner::new();
                        let updates = get_all_updates(cli.socket).await;
                        spinner.succeed();
                        print_updates(&updates, icons);
                    }
                };
            }
        },
        #[cfg(feature = "server")]
        Some(Commands::Serve { port }) => {
            let _ = serve(port, cli.socket).await;
        }
        None => (),
    }
}

async fn get_all_updates(socket: Option<String>) -> Vec<(String, Option<bool>)> {
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
        let mut latest_images = match check_auth(registry) {
            Some(auth_url) => {
                let token = get_token(images.clone(), &auth_url);
                get_latest_digests(images, Some(&token))
            }
            None => get_latest_digests(images, None),
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
async fn get_update(image: &str, socket: Option<String>) -> Option<bool> {
    let local_image = get_image_from_docker_daemon(socket, image).await;
    let token = match check_auth(&local_image.registry) {
        Some(auth_url) => get_token(vec![&local_image], &auth_url),
        None => String::new(),
    };
    let remote_image = match token.as_str() {
        "" => get_latest_digest(&local_image, None),
        _ => get_latest_digest(&local_image, Some(&token)),
    };
    match &remote_image.digest {
        Some(d) => Some(d != &local_image.digest.unwrap()),
        None => None,
    }
}