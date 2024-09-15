use bollard::{secret::ImageSummary, ClientVersion, Docker};

#[cfg(feature = "cli")]
use bollard::secret::ImageInspect;
use futures::future::join_all;

use crate::{
    error,
    image::Image,
    utils::{split_image, CliConfig},
};

fn create_docker_client(socket: Option<String>) -> Docker {
    let client: Result<Docker, bollard::errors::Error> = match socket {
        Some(sock) => Docker::connect_with_local(
            &sock,
            120,
            &ClientVersion {
                major_version: 1,
                minor_version: 44,
            },
        ),
        None => Docker::connect_with_local_defaults(),
    };

    match client {
        Ok(d) => d,
        Err(e) => error!("Failed to connect to docker socket!\n{}", e),
    }
}

pub async fn get_images_from_docker_daemon(options: &CliConfig) -> Vec<Image> {
    let client: Docker = create_docker_client(options.socket.clone());
    let images: Vec<ImageSummary> = match client.list_images::<String>(None).await {
        Ok(images) => images,
        Err(e) => {
            error!("Failed to retrieve list of images available!\n{}", e)
        }
    };
    let mut handles = Vec::new();
    for image in images {
        handles.push(Image::from(image, options))
    }
    join_all(handles)
        .await
        .iter()
        .filter(|img| img.is_some())
        .map(|img| img.clone().unwrap())
        .collect()
}

#[cfg(feature = "cli")]
pub async fn get_image_from_docker_daemon(socket: Option<String>, name: &str) -> Image {
    let client: Docker = create_docker_client(socket);
    let image: ImageInspect = match client.inspect_image(name).await {
        Ok(i) => i,
        Err(e) => error!("Failed to retrieve image {} from daemon\n{}", name, e),
    };
    match image.repo_tags {
        Some(_) => (),
        None => error!("Image has no tags"), // I think this is actually unreachable
    }
    match image.repo_digests {
        Some(d) => {
            let (registry, repository, tag) = split_image(&image.repo_tags.unwrap()[0]);
            Image {
                registry,
                repository,
                tag,
                digest: Some(d[0].clone().split('@').collect::<Vec<&str>>()[1].to_string()),
            }
        }
        None => error!("No digests found for image {}", name),
    }
}
