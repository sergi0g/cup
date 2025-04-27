use bollard::{container::ListContainersOptions, models::ImageInspect, ClientVersion, Docker};

use futures::future::join_all;

use crate::{error, structs::image::Image, Context};

fn create_docker_client(socket: Option<&str>) -> Docker {
    let client: Result<Docker, bollard::errors::Error> = match socket {
        Some(sock) => {
            if sock.starts_with("unix://") {
                Docker::connect_with_unix(
                    sock,
                    120,
                    &ClientVersion {
                        major_version: 1,
                        minor_version: 44,
                    },
                )
            } else {
                Docker::connect_with_http(
                    sock,
                    120,
                    &ClientVersion {
                        major_version: 1,
                        minor_version: 44,
                    },
                )
            }
        }
        None => Docker::connect_with_unix_defaults(),
    };

    match client {
        Ok(d) => d,
        Err(e) => error!("Failed to connect to docker daemon!\n{}", e),
    }
}

/// Retrieves images from Docker daemon. If `references` is Some, return only the images whose references match the ones specified.
pub async fn get_images_from_docker_daemon(
    ctx: &Context,
    references: &Option<Vec<String>>,
) -> Vec<Image> {
    let client: Docker = create_docker_client(ctx.config.socket.as_deref());
    let mut swarm_images = match client.list_services::<String>(None).await {
        Ok(services) => services
            .iter()
            .filter_map(|service| match &service.spec {
                Some(service_spec) => match &service_spec.task_template {
                    Some(task_spec) => match &task_spec.container_spec {
                        Some(container_spec) => match &container_spec.image {
                            Some(image) => Image::from_inspect_data(ctx, image),
                            None => None,
                        },
                        None => None,
                    },
                    None => None,
                },
                None => None,
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    let mut local_images = match references {
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
            inspects
                .iter()
                .filter_map(|inspect| Image::from_inspect_data(ctx, inspect.clone()))
                .collect()
        }
        None => {
            let images = match client.list_images::<String>(None).await {
                Ok(images) => images,
                Err(e) => {
                    error!("Failed to retrieve list of images available!\n{}", e)
                }
            };
            images
                .iter()
                .filter_map(|image| Image::from_inspect_data(ctx, image.clone()))
                .collect::<Vec<Image>>()
        }
    };
    local_images.append(&mut swarm_images);
    local_images
}

pub async fn get_in_use_images(ctx: &Context, references: &Option<Vec<String>>) -> Vec<String> {
    let client: Docker = create_docker_client(ctx.config.socket.as_deref());

    let containers = match client
        .list_containers::<String>(Some(ListContainersOptions {
            all: true,
            ..Default::default()
        }))
        .await
    {
        Ok(containers) => containers,
        Err(e) => {
            error!("Failed to retrieve list of containers available!\n{}", e)
        }
    };

    containers
        .iter()
        .filter_map(|container| {
            let image = match container.image.as_deref() {
                Some(image) => {
                    if image.contains(":") {
                        image.to_string()
                    } else {
                        format!("{}:latest", image)
                    }
                }
                None => return None,
            };
            match references {
                Some(refs) => {
                    if refs.contains(&image) {
                        Some(image)
                    } else {
                        None
                    }
                }
                None => Some(image),
            }
        })
        .collect()
}
