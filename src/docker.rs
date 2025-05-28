use bollard::{
    models::ImageInspect,
    query_parameters::{
        CreateContainerOptionsBuilder, CreateImageOptionsBuilder, InspectContainerOptions,
        ListContainersOptionsBuilder, ListImagesOptions, ListServicesOptions,
        RemoveContainerOptions, RenameContainerOptions, StartContainerOptions,
        StopContainerOptions,
    },
    secret::{ContainerCreateBody, CreateImageInfo},
    ClientVersion, Docker,
};

use futures::{future::join_all, StreamExt};
use rustc_hash::FxHashMap;

use crate::{
    error,
    structs::{
        image::Image,
        update::{Update, UpdateInfo},
    },
    Context,
};

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
    let mut swarm_images = match client.list_services(None::<ListServicesOptions>).await {
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
            let images = match client.list_images(None::<ListImagesOptions>).await {
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

pub async fn get_in_use_images(ctx: &Context) -> FxHashMap<String, Vec<String>> {
    let client: Docker = create_docker_client(ctx.config.socket.as_deref());

    let options = ListContainersOptionsBuilder::new().all(true).build();
    let containers = match client.list_containers(Some(options)).await {
        Ok(containers) => containers,
        Err(e) => {
            error!("Failed to retrieve list of containers available!\n{}", e)
        }
    };

    let mut result: FxHashMap<String, Vec<String>> = FxHashMap::default();

    containers
        .iter()
        .filter(|container| container.image.is_some())
        .for_each(|container| {
            let reference = match &container.image {
                Some(image) => {
                    if image.contains(":") {
                        image.clone()
                    } else {
                        format!("{image}:latest")
                    }
                }
                None => unreachable!(),
            };

            let mut names: Vec<String> = container
                .names
                .as_ref()
                .map(|names| {
                    names
                        .iter()
                        .map(|name| name.trim_start_matches('/').to_owned())
                        .collect()
                })
                .unwrap_or_default();

            match result.get_mut(&reference) {
                Some(containers) => containers.append(&mut names),
                None => {
                    let _ = result.insert(reference, names);
                }
            }
        });
    result.clone()
}

/// Given a container name and the update information returned about the image it uses, tries to recreate it with a new image / latest version of the current image
pub async fn upgrade_container(ctx: &Context, name: &str, update: &Update) -> Result<(), String> {
    let client: Docker = create_docker_client(ctx.config.socket.as_deref()); // TODO: Consider adding all these functions to a long lived struct with a shared client. We don't want to create a new client for every container updated.

    // Create a few variables that will be used later on
    let new_name = format!("{name}__cup_temp"); // A new temporary name for the container. Instead of removing the old one straight away, we'll create a new one and if that succeeds we'll rename it.
    let new_image = match &update.result.info {
        // Find the new reference for the image, based on logic used in the web interface. This will be used to pull the new image
        UpdateInfo::Version(update_info) => format!(
            "{}:{}",
            update
                .reference
                .split_once(':')
                .expect("Reference contains `:`")
                .0,
            update_info.new_tag
        ),
        UpdateInfo::Digest(_) => update.reference.clone(),
        UpdateInfo::None => unreachable!("Tried to update up-to-date image"),
    };
    ctx.logger.debug(format!("Upgrading {name}..."));

    // Retrieve information about current container and construct required structs to create a new container afterwards
    let (create_options, create_config) = match client
        .inspect_container(name, None::<InspectContainerOptions>)
        .await
    {
        Ok(inspect) => {
            let create_options = {
                let mut options = CreateContainerOptionsBuilder::new();
                match inspect.name {
                    Some(_) => options = options.name(&new_name),
                    None => (), // Not sure if this is even reachable
                };
                match inspect.platform {
                    Some(platform) => options = options.platform(&platform),
                    None => (), // Same as above
                };
                options.build()
            };

            let inspect_config = inspect.config.unwrap(); // For easier access later

            let create_config = ContainerCreateBody {
                hostname: inspect_config.hostname,
                domainname: inspect_config.domainname,
                user: inspect_config.user,
                attach_stdin: inspect_config.attach_stdin,
                attach_stderr: inspect_config.attach_stderr,
                attach_stdout: inspect_config.attach_stdout,
                exposed_ports: inspect_config.exposed_ports,
                tty: inspect_config.tty,
                open_stdin: inspect_config.open_stdin,
                stdin_once: inspect_config.stdin_once,
                env: inspect_config.env,
                cmd: inspect_config.cmd,
                healthcheck: inspect_config.healthcheck,
                args_escaped: inspect_config.args_escaped,
                image: Some(new_image.clone()),
                volumes: inspect_config.volumes,
                working_dir: inspect_config.working_dir,
                entrypoint: inspect_config.entrypoint,
                network_disabled: inspect_config.network_disabled,
                mac_address: inspect_config.mac_address,
                on_build: inspect_config.on_build,
                labels: inspect_config.labels,
                stop_signal: inspect_config.stop_signal,
                stop_timeout: inspect_config.stop_timeout,
                shell: inspect_config.shell,
                host_config: inspect.host_config,
                // The commented out code below doesn't work because bollard sends gw_priority as a float and Docker expects an int. Tracking issue: https://github.com/fussybeaver/bollard/issues/537
                // networking_config: Some(bollard::secret::NetworkingConfig {
                //     endpoints_config: inspect.network_settings.unwrap().networks,
                // }),
                networking_config: None,
            };
            (create_options, create_config)
        }
        Err(e) => {
            let message = format!("Failed to inspect container {name}: {e}");
            ctx.logger.warn(&message);
            return Err(message)
        },
    };

    // Stop the current container
    ctx.logger.debug(format!("Stopping {name}..."));
    match client
        .stop_container(name, None::<StopContainerOptions>)
        .await
    {
        Ok(()) => ctx.logger.debug(format!("Successfully stopped {name}")),
        Err(e) => {
            let message = format!("Failed to stop container {name}: {e}");
            ctx.logger.warn(&message);
            return Err(message)
        },
    };

    // Don't let the naming fool you, we're pulling the new image here.
    ctx.logger.debug(format!("Pulling {new_image} for {name}..."));
    let create_image_options = CreateImageOptionsBuilder::new()
        .from_image(&new_image)
        .build();

    client
        .create_image(Some(create_image_options), None, None) // TODO: credentials support
        .collect::<Vec<Result<CreateImageInfo, bollard::errors::Error>>>() // Not entirely sure this is the best way to handle a stream
        .await; // TODO: handle errors here
    ctx.logger.debug(format!("Successfully pulled new image for {name}"));

    // Create the new container
    ctx.logger.debug(format!("Creating new container for {name}..."));
    match client
        .create_container(Some(create_options), create_config)
        .await
    {
        Ok(response) => {
            // Let the user know if any warnings occured
            response
                .warnings
                .iter()
                .for_each(|warning| ctx.logger.warn(format!("[DAEMON]: {}", warning)));
        },
        Err(e) => {
            let message = format!("Failed to create new container for {name}: {e}");
            ctx.logger.warn(&message);
            return Err(message)
        },
    };

    // Start the new container
    match client
        .start_container(&new_name, None::<StartContainerOptions>)
        .await
    {
        Ok(()) => ctx.logger.debug(format!("Successfully created new container for {name}")),
        Err(e) => {
            let message = format!("Failed to start new container for {name}: {e}");
            ctx.logger.warn(&message);
            return Err(message)
        },
    }    
    
    // Remove the old container
    ctx.logger.debug(format!("Removing old {name} container"));
    match client
        .remove_container(name, None::<RemoveContainerOptions>)
        .await
    {
        Ok(()) => ctx.logger.debug(format!("Successfully removed old {name} container")),
        Err(e) => {
            match e {
                bollard::errors::Error::DockerResponseServerError { status_code: 404, message } => {
                    ctx.logger.warn(format!("Failed to remove container {name}, it was probably started with `--rm` and has been automatically cleaned up. Message from server: {message}"))
                },
                _ => {
                    let message = format!("Failed to remove container {name}: {e}");
                    ctx.logger.warn(&message);
                    return Err(message)
                },
            }
        }
    }

    // Rename the new container
    match client
        .rename_container(
            &new_name,
            RenameContainerOptions {
                name: name.to_owned(),
            },
        )
        .await
    {
        Ok(()) => (),
        Err(e) => {
            let message = format!("Failed to rename container {name}: {e}");
            ctx.logger.warn(&message);
            return Err(message)
        },
    }
    
    ctx.logger.debug(format!("Successfully upgraded {name}!"));

    Ok(())
}
