use json::JsonValue;

use http_auth::parse_challenges;
use reqwest_middleware::ClientWithMiddleware;

use crate::{config::Config, error, image::Image, warn};

pub async fn check_auth(
    registry: &str,
    config: &Config,
    client: &ClientWithMiddleware,
) -> Option<String> {
    let protocol = if config.insecure_registries.contains(&registry.to_string()) {
        "http"
    } else {
        "https"
    };
    let response = client
        .get(format!("{}://{}/v2/", protocol, registry))
        .send()
        .await;
    match response {
        Ok(r) => {
            let status = r.status().as_u16();
            if status == 401 {
                match r.headers().get("www-authenticate") {
                    Some(challenge) => Some(parse_www_authenticate(challenge.to_str().unwrap())),
                    None => error!(
                        "Unauthorized to access registry {} and no way to authenticate was provided",
                        registry
                    ),
                }
            } else if status == 200 {
                None
            } else {
                warn!(
                    "Received unexpected status code {}\nResponse: {}",
                    status,
                    r.text().await.unwrap()
                );
                None
            }
        }
        Err(e) => {
            if e.is_connect() {
                warn!("Connection to registry {} failed.", &registry);
                None
            } else {
                error!("Unexpected error: {}", e.to_string())
            }
        }
    }
}

pub async fn get_latest_digest(
    image: &Image,
    token: Option<&String>,
    config: &Config,
    client: &ClientWithMiddleware,
) -> Image {
    let protocol = if config.insecure_registries.contains(&image.registry.clone().unwrap())
    {
        "http"
    } else {
        "https"
    };
    let mut request = client.head(format!(
        "{}://{}/v2/{}/manifests/{}",
        protocol,
        &image.registry.as_ref().unwrap(),
        &image.repository.as_ref().unwrap(),
        &image.tag.as_ref().unwrap()
    ));
    if let Some(t) = token {
        request = request.header("Authorization", &format!("Bearer {}", t));
    }
    let raw_response = match request
        .header("Accept", "application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.oci.image.index.v1+json")
        .send().await
    {
        Ok(response) => {
            let status = response.status();
            if status == 401 {
                if token.is_some() {
                    warn!("Failed to authenticate to registry {} with given token!\n{}", &image.registry.as_ref().unwrap(), token.unwrap());
                } else {
                    warn!("Registry requires authentication");
                }
                return Image { remote_digest: None, ..image.clone() }
            } else if status == 404 {
                warn!("Image {:?} not found", &image);
                return Image { remote_digest: None, ..image.clone() }
            } else {
                response
            }
        },
        Err(e) => {
            if e.is_connect() {
                warn!("Connection to registry failed.");
                return Image { remote_digest: None, ..image.clone() }
            } else {
                error!("Unexpected error: {}", e.to_string())
            }
        },
    };
    match raw_response.headers().get("docker-content-digest") {
        Some(digest) => Image {
            remote_digest: Some(digest.to_str().unwrap().to_string()),
            ..image.clone()
        },
        None => error!(
            "Server returned invalid response! No docker-content-digest!\n{:#?}",
            raw_response
        ),
    }
}

pub async fn get_token(
    images: &Vec<&Image>,
    auth_url: &str,
    credentials: &Option<&String>,
    client: &ClientWithMiddleware,
) -> String {
    let mut final_url = auth_url.to_owned();
    for image in images {
        final_url = format!(
            "{}&scope=repository:{}:pull",
            final_url,
            image.repository.as_ref().unwrap()
        );
    }
    let mut base_request = client
        .get(&final_url)
        .header("Accept", "application/vnd.oci.image.index.v1+json"); // Seems to be unnecessary. Will probably remove in the future
    base_request = match credentials {
        Some(creds) => base_request.header("Authorization", &format!("Basic {}", creds)),
        None => base_request,
    };
    let raw_response = match base_request.send().await {
        Ok(response) => match response.text().await {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to parse response into string!\n{}", e)
            }
        },
        Err(e) => {
            if e.is_connect() {
                error!("Connection to registry failed.");
            } else {
                error!("Token request failed!\n{}", e.to_string())
            }
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
