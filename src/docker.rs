use bollard::{models::ImageInspect, ClientVersion, Docker};

use futures::future::join_all;

use crate::{error, image::Image, config::Config};

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

/// Retrieves images from Docker daemon. If `references` is Some, return only the images whose references match the ones specified.
pub async fn get_images_from_docker_daemon(
    config: &Config,
    references: &Option<Vec<String>>,
) -> Vec<Image> {
    let client: Docker = create_docker_client(config.socket.clone());
    // If https://github.com/moby/moby/issues/48612 is fixed, this code should be faster. For now a workaround will be used.
    // let mut filters = HashMap::with_capacity(1);
    // match references {
    //     Some(refs) => {
    //         filters.insert("reference".to_string(), refs.clone());
    //     }
    //     None => (),
    // }
    // let images: Vec<ImageSummary> = match client
    //     .list_images::<String>(Some(ListImagesOptions {
    //         filters,
    //         ..Default::default()
    //     }))
    //     .await
    // {
    //     Ok(images) => images,
    //     Err(e) => {
    //         error!("Failed to retrieve list of images available!\n{}", e)
    //     }
    // };
    // let mut handles = Vec::new();
    // for image in images {
    //     handles.push(Image::from(image, options))
    // }
    // join_all(handles)
    //     .await
    //     .iter()
    //     .filter_map(|img| img.clone())
    //     .collect()
    match references {
        Some(refs) => {
            let mut inspect_handles = Vec::with_capacity(refs.len());
            for reference in refs {
                inspect_handles.push(client.inspect_image(reference));
            }
            let inspects: Vec<ImageInspect> = join_all(inspect_handles)
                .await
                .iter()
                .filter(|inspect| inspect.is_ok())
                .map(|inspect| inspect.as_ref().unwrap().clone())
                .collect();
            let mut image_handles = Vec::with_capacity(inspects.len());
            for inspect in inspects {
                image_handles.push(Image::from_inspect(inspect.clone()));
            }
            join_all(image_handles)
                .await
                .iter()
                .filter_map(|img| img.clone())
                .collect()
        }
        None => {
            let images = match client.list_images::<String>(None).await {
                Ok(images) => images,
                Err(e) => {
                    error!("Failed to retrieve list of images available!\n{}", e)
                }
            };
            let mut handles = Vec::new();
            for image in images {
                handles.push(Image::from_summary(image))
            }
            join_all(handles)
                .await
                .iter()
                .filter_map(|img| img.clone())
                .collect()
        }
    }
}
