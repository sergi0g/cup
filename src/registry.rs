use std::time::SystemTime;

use itertools::Itertools;

use crate::{
    config::Config,
    debug, error,
    http::Client,
    structs::{
        image::{DigestInfo, Image, VersionInfo},
        version::Version,
    },
    utils::{
        link::parse_link,
        request::{
            get_protocol, get_response_body, parse_json, parse_www_authenticate, to_bearer_string,
        }, time::{elapsed, now},
    },
};

pub async fn check_auth(registry: &str, config: &Config, client: &Client) -> Option<String> {
    let protocol = get_protocol(registry, &config.registries);
    let url = format!("{}://{}/v2/", protocol, registry);
    let response = client.get(&url, Vec::new(), true).await;
    match response {
        Ok(response) => {
            let status = response.status();
            if status == 401 {
                match response.headers().get("www-authenticate") {
                        Some(challenge) => Some(parse_www_authenticate(challenge.to_str().unwrap())),
                        None => error!(
                            "Unauthorized to access registry {} and no way to authenticate was provided",
                            registry
                        ),
                    }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub async fn get_latest_digest(
    image: &Image,
    token: Option<&str>,
    config: &Config,
    client: &Client,
) -> Image {
    debug!(
        config.debug,
        "Checking for digest update to {}", image.reference
    );
    let start = SystemTime::now();
    let protocol = get_protocol(&image.registry, &config.registries);
    let url = format!(
        "{}://{}/v2/{}/manifests/{}",
        protocol, &image.registry, &image.repository, &image.tag
    );
    let authorization = to_bearer_string(&token);
    let headers = vec![("Accept", Some("application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.oci.image.index.v1+json")), ("Authorization", authorization.as_deref())];

    let response = client.head(&url, headers).await;
    let time = start.elapsed().unwrap().as_millis() as u32;
    debug!(
        config.debug,
        "Checked for digest update to {} in {}ms", image.reference, time
    );
    match response {
        Ok(res) => match res.headers().get("docker-content-digest") {
            Some(digest) => {
                let local_digests = match &image.digest_info {
                    Some(data) => data.local_digests.clone(),
                    None => return image.clone(),
                };
                Image {
                    digest_info: Some(DigestInfo {
                        remote_digest: Some(digest.to_str().unwrap().to_string()),
                        local_digests,
                    }),
                    time_ms: image.time_ms + time,
                    ..image.clone()
                }
            }
            None => error!(
                "Server returned invalid response! No docker-content-digest!\n{:#?}",
                res
            ),
        },
        Err(error) => Image {
            error: Some(error),
            time_ms: image.time_ms + time,
            ..image.clone()
        },
    }
}

pub async fn get_token(
    images: &Vec<&Image>,
    auth_url: &str,
    credentials: &Option<String>,
    client: &Client,
) -> String {
    let mut url = auth_url.to_owned();
    for image in images {
        url = format!("{}&scope=repository:{}:pull", url, image.repository);
    }
    let authorization = credentials.as_ref().map(|creds| format!("Basic {}", creds));
    let headers = vec![("Authorization", authorization.as_deref())];

    let response = client.get(&url, headers, false).await;
    let response_json = match response {
        Ok(response) => parse_json(&get_response_body(response).await),
        Err(_) => error!("GET {}: Request failed!", url),
    };
    response_json["token"].as_str().unwrap().to_string()
}

pub async fn get_latest_tag(
    image: &Image,
    base: &Version,
    token: Option<&str>,
    config: &Config,
    client: &Client,
) -> Image {
    debug!(
        config.debug,
        "Checking for tag update to {}", image.reference
    );
    let start = now();
    let protocol = get_protocol(&image.registry, &config.registries);
    let url = format!(
        "{}://{}/v2/{}/tags/list",
        protocol, &image.registry, &image.repository,
    );
    let authorization = to_bearer_string(&token);
    let headers = vec![
        ("Accept", Some("application/json")),
        ("Authorization", authorization.as_deref()),
    ];

    let mut tags: Vec<Version> = Vec::new();
    let mut next_url = Some(url);

    while next_url.is_some() {
        debug!(
            config.debug,
            "{} has extra tags! Current number of valid tags: {}",
            image.reference,
            tags.len()
        );
        let (new_tags, next) = match get_extra_tags(
            &next_url.unwrap(),
            headers.clone(),
            base,
            &image.version_info.as_ref().unwrap().format_str,
            client,
        )
        .await
        {
            Ok(t) => t,
            Err(message) => {
                return Image {
                    error: Some(message),
                    time_ms: image.time_ms + elapsed(start),
                    ..image.clone()
                }
            }
        };
        tags.extend_from_slice(&new_tags);
        next_url = next;
    }
    let tag = tags.iter().max();
    debug!(
        config.debug,
        "Checked for tag update to {} in {}ms", image.reference, elapsed(start)
    );
    match tag {
        Some(t) => {
            if t == base && image.digest_info.is_some() {
                // Tags are equal so we'll compare digests
                get_latest_digest(
                    &Image {
                        version_info: Some(VersionInfo {
                            latest_remote_tag: Some(t.clone()),
                            ..image.version_info.as_ref().unwrap().clone()
                        }),
                        time_ms: image.time_ms + elapsed(start),
                        ..image.clone()
                    },
                    token,
                    config,
                    client,
                )
                .await
            } else {
                Image {
                    version_info: Some(VersionInfo {
                        latest_remote_tag: Some(t.clone()),
                        ..image.version_info.as_ref().unwrap().clone()
                    }),
                    time_ms: image.time_ms + elapsed(start),
                    ..image.clone()
                }
            }
        }
        None => unreachable!("{:?}", tags),
    }
}

pub async fn get_extra_tags(
    url: &str,
    headers: Vec<(&str, Option<&str>)>,
    base: &Version,
    format_str: &str,
    client: &Client,
) -> Result<(Vec<Version>, Option<String>), String> {
    let response = client.get(url, headers, false).await;

    match response {
        Ok(res) => {
            let next_url = res
                .headers()
                .get("Link")
                .map(|link| parse_link(link.to_str().unwrap(), url));
            let response_json = parse_json(&get_response_body(res).await);
            let result = response_json["tags"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|tag| Version::from_tag(tag.as_str().unwrap()))
                .filter(|(tag, format_string)| match (base.minor, tag.minor) {
                    (Some(_), Some(_)) | (None, None) => {
                        matches!((base.patch, tag.patch), (Some(_), Some(_)) | (None, None))
                            && format_str == *format_string
                    }
                    _ => false,
                })
                .map(|(tag, _)| tag)
                .dedup()
                .collect();
            Ok((result, next_url))
        }
        Err(message) => Err(message),
    }
}
