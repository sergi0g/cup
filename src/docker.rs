use bollard::{secret::ImageSummary, ClientVersion, Docker};

#[cfg(feature = "cli")]
use bollard::secret::ImageInspect;
use futures::future::join_all;

use crate::{error, image::Image, utils::CliConfig};

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
        handles.push(Image::from_summary(image, options))
    }
    join_all(handles)
        .await
        .iter()
        .filter_map(|img| img.clone())
        .collect()
}

#[cfg(feature = "cli")]
pub async fn get_image_from_docker_daemon(socket: &Option<String>, name: &str) -> Image {
    let client: Docker = create_docker_client(socket.clone());
    let image: ImageInspect = match client.inspect_image(name).await {
        Ok(i) => i,
        Err(e) => error!("Failed to retrieve image {} from daemon\n{}", name, e),
    };
    match Image::from_inspect(image).await {
        Some(img) => img,
        None => error!("No valid tags or digests for image!"),
    }
}
