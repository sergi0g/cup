use std::sync::Mutex;

use json::JsonValue;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ureq::{Error, ErrorKind};

use http_auth::parse_challenges;

use crate::{error, image::Image, warn};

pub fn check_auth(registry: &str, config: &JsonValue) -> Option<String> {
    let protocol = if config["insecure_registries"].contains(registry) {
        "http"
    } else {
        "https"
    };
    let response = ureq::get(&format!("{}://{}/v2/", protocol, registry)).call();
    match response {
        Ok(_) => None,
        Err(Error::Status(401, response)) => match response.header("www-authenticate") {
            Some(challenge) => Some(parse_www_authenticate(challenge)),
            None => error!(
                "Unauthorized to access registry {} and no way to authenticate was provided",
                registry
            ),
        },
        Err(Error::Transport(error)) => {
            match error.kind() {
                ErrorKind::Dns => {
                    warn!("Failed to lookup the IP of the registry, retrying.");
                    return check_auth(registry, config);
                } // If something goes really wrong, this can get stuck in a loop
                ErrorKind::ConnectionFailed => {
                    warn!("Connection probably timed out, retrying.");
                    return check_auth(registry, config);
                } // Same here
                _ => error!("{}", error),
            }
        }
        Err(e) => error!("{}", e),
    }
}

pub fn get_latest_digest(image: &Image, token: Option<&String>, config: &JsonValue) -> Image {
    let protocol =
        if config["insecure_registries"].contains(json::JsonValue::from(image.registry.clone())) {
            "http"
        } else {
            "https"
        };
    let mut request = ureq::head(&format!(
        "{}://{}/v2/{}/manifests/{}",
        protocol, &image.registry, &image.repository, &image.tag
    ));
    if let Some(t) = token {
        request = request.set("Authorization", &format!("Bearer {}", t));
    }
    let raw_response = match request
        .set("Accept", "application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.oci.image.index.v1+json")
        .call()
    {
        Ok(response) => response,
        Err(Error::Status(401, response)) => {
            if token.is_some() {
                warn!("Failed to authenticate to registry {} with given token!\n{}", &image.registry, token.unwrap());
                return Image { digest: None, ..image.clone() }
            } else {
                return get_latest_digest(
                    image,
                    Some(&get_token(
                        vec![image],
                        &parse_www_authenticate(response.header("www-authenticate").unwrap()),
                        &None // I think?
                    )),
                    config
                );
            }
        }
        Err(Error::Status(_, _)) => {
            return Image {
                digest: None,
                ..image.clone()
            }
        },
        Err(Error::Transport(error)) => {
            match error.kind() {
                ErrorKind::Dns => {
                    warn!("Failed to lookup the IP of the registry, retrying.");
                    return get_latest_digest(image, token, config)
                }, // If something goes really wrong, this can get stuck in a loop
                ErrorKind::ConnectionFailed => {
                    warn!("Connection probably timed out, retrying.");
                    return get_latest_digest(image, token, config)
                }, // Same here
                _ => error!("Failed to retrieve image digest\n{}!", error)
            }
        },
    };
    match raw_response.header("docker-content-digest") {
        Some(digest) => Image {
            digest: Some(digest.to_string()),
            ..image.clone()
        },
        None => error!("Server returned invalid response! No docker-content-digest!"),
    }
}

pub fn get_latest_digests(
    images: Vec<&Image>,
    token: Option<&String>,
    config: &JsonValue,
) -> Vec<Image> {
    let result: Mutex<Vec<Image>> = Mutex::new(Vec::new());
    images.par_iter().for_each(|&image| {
        let digest = get_latest_digest(image, token, config).digest;
        result.lock().unwrap().push(Image {
            digest,
            ..image.clone()
        });
    });
    let r = result.lock().unwrap().clone();
    r
}

pub fn get_token(images: Vec<&Image>, auth_url: &str, credentials: &Option<String>) -> String {
    let mut final_url = auth_url.to_owned();
    for image in &images {
        final_url = format!("{}&scope=repository:{}:pull", final_url, image.repository);
    }
    let mut base_request =
        ureq::get(&final_url).set("Accept", "application/vnd.oci.image.index.v1+json"); // Seems to be unnecesarry. Will probably remove in the future
    base_request = match credentials {
        Some(creds) => base_request.set("Authorization", &format!("Basic {}", creds)),
        None => base_request,
    };
    let raw_response = match base_request.call() {
        Ok(response) => match response.into_string() {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to parse response into string!\n{}", e)
            }
        },
        Err(Error::Transport(error)) => {
            match error.kind() {
                ErrorKind::Dns => {
                    warn!("Failed to lookup the IP of the registry, retrying.");
                    return get_token(images, auth_url, credentials);
                } // If something goes really wrong, this can get stuck in a loop
                ErrorKind::ConnectionFailed => {
                    warn!("Connection probably timed out, retrying.");
                    return get_token(images, auth_url, credentials);
                } // Same here
                _ => error!("Token request failed\n{}!", error),
            }
        }
        Err(e) => {
            error!("Token request failed!\n{}", e)
        }
    };
    let parsed_token_response: JsonValue = match json::parse(&raw_response) {
        Ok(parsed) => parsed,
        Err(e) => {
            error!("Failed to parse server response\n{}", e)
        }
    };
    parsed_token_response["token"].to_string()
}

fn parse_www_authenticate(www_auth: &str) -> String {
    let challenges = parse_challenges(www_auth).unwrap();
    if !challenges.is_empty() {
        let challenge = &challenges[0];
        if challenge.scheme == "Bearer" {
            format!(
                "{}?service={}",
                challenge.params[0].1.as_escaped(),
                challenge.params[1].1.as_escaped()
            )
        } else {
            error!("Unsupported scheme {}", &challenge.scheme)
        }
    } else {
        error!("No challenge provided by the server");
    }
}
