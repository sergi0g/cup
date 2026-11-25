use std::time::SystemTime;

use itertools::Itertools;

use crate::{
    config::UpdateType,
    error,
    http::Client,
    structs::{
        image::{DigestInfo, Image, VersionInfo},
        version::Version,
    },
    utils::{
        link::parse_link,
        request::{
            get_protocol, get_response_body, parse_json, parse_www_authenticate, to_bearer_string,
        },
        time::{elapsed, now},
    },
    Context,
};

/// Checks whether the registry requires authentication. Returns the URL to fetch a token from if it does, `None` if it doesn't, or an error message if the check failed in a way that prevents checking images from this registry.
pub async fn check_auth(
    registry: &str,
    ctx: &Context,
    client: &Client,
) -> Result<Option<String>, String> {
    let protocol = get_protocol(registry, &ctx.config.registries);
    let url = format!("{}://{}/v2/", protocol, registry);
    let response = client.get(&url, &[], true).await;
    match response {
        Ok(response) => {
            let status = response.status();
            if status == 401 {
                match response
                    .headers()
                    .get("www-authenticate")
                    .and_then(|challenge| challenge.to_str().ok())
                {
                    Some(challenge) => parse_www_authenticate(challenge)
                        .map(Some)
                        .map_err(|e| format!("GET {}: {}", url, e)),
                    None => Err(format!(
                        "Unauthorized to access registry {} and no way to authenticate was provided",
                        registry
                    )),
                }
            } else {
                Ok(None)
            }
        }
        Err(_) => Ok(None),
    }
}

pub async fn get_latest_digest(
    image: &Image,
    token: Option<&str>,
    ctx: &Context,
    client: &Client,
) -> Image {
    ctx.logger
        .debug(format!("Checking for digest update to {}", image.reference));
    let start = SystemTime::now();
    let protocol = get_protocol(&image.parts.registry, &ctx.config.registries);
    let url = format!(
        "{}://{}/v2/{}/manifests/{}",
        protocol, image.parts.registry, image.parts.repository, image.parts.tag
    );
    let authorization = to_bearer_string(&token);
    let headers = [("Accept", Some("application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.oci.image.index.v1+json, application/vnd.oci.image.manifest.v1+json")), ("Authorization", authorization.as_deref())];

    let response = client.head(&url, &headers).await;
    let time = start.elapsed().unwrap().as_millis() as u32;
    ctx.logger.debug(format!(
        "Checked for digest update to {} in {}ms",
        image.reference, time
    ));
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
            None => {
                let message = format!(
                    "HEAD {}: Server returned invalid response! No docker-content-digest!",
                    url
                );
                ctx.logger.warn(&message);
                Image {
                    error: Some(message),
                    time_ms: image.time_ms + time,
                    ..image.clone()
                }
            }
        },
        Err(error) => Image {
            error: Some(error),
            time_ms: image.time_ms + time,
            ..image.clone()
        },
    }
}

pub async fn get_token(
    images: &[&Image],
    auth_url: &str,
    credentials: &Option<String>,
    client: &Client,
) -> Result<String, String> {
    let mut url = auth_url.to_owned();
    for image in images {
        url = format!("{}&scope=repository:{}:pull", url, image.parts.repository);
    }
    let authorization = credentials.as_ref().map(|creds| format!("Basic {}", creds));
    let headers = [("Authorization", authorization.as_deref())];

    let response = client.get(&url, &headers, false).await?;
    let body = response
        .text()
        .await
        .map_err(|e| format!("GET {}: Failed to read token response: {}", url, e))?;
    let response_json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("GET {}: Failed to parse token response: {}", url, e))?;
    response_json["token"]
        .as_str()
        .map(|token| token.to_string())
        .ok_or_else(|| format!("GET {}: Token response did not contain a token", url))
}

pub async fn get_latest_tag(
    image: &Image,
    base: &Version,
    token: Option<&str>,
    ctx: &Context,
    client: &Client,
    excluded_tags: Vec<String>,
    ignored_update_type: UpdateType,
) -> Image {
    ctx.logger
        .debug(format!("Checking for tag update to {}", image.reference));
    let start = now();
    let protocol = get_protocol(&image.parts.registry, &ctx.config.registries);
    let url = format!(
        "{}://{}/v2/{}/tags/list",
        protocol, image.parts.registry, image.parts.repository,
    );
    let authorization = to_bearer_string(&token);
    let headers = [
        ("Accept", Some("application/json")),
        ("Authorization", authorization.as_deref()),
    ];

    let mut tags: Vec<Version> = Vec::new();
    let mut next_url = Some(url);

    while next_url.is_some() {
        ctx.logger.debug(format!(
            "{} has extra tags! Current number of valid tags: {}",
            image.reference,
            tags.len()
        ));
        let (new_tags, next) = match get_extra_tags(
            &next_url.unwrap(),
            &headers,
            base,
            &image.version_info.as_ref().unwrap().format_str,
            ctx,
            client,
            &excluded_tags,
            ignored_update_type,
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
    ctx.logger.debug(format!(
        "Checked for tag update to {} in {}ms",
        image.reference,
        elapsed(start)
    ));
    match tag {
        Some(t) => {
            if t == base && image.digest_info.is_some() {
                // Tags are equal so we'll compare digests
                ctx.logger.debug(format!(
                    "Tags for {} are equal, comparing digests.",
                    image.reference
                ));
                get_latest_digest(
                    &Image {
                        version_info: None, // Overwrite previous version info, since it isn't useful anymore (equal tags means up to date and an image is truly up to date when its digests are up to date, and we'll be checking those anyway)
                        time_ms: image.time_ms + elapsed(start),
                        ..image.clone()
                    },
                    token,
                    ctx,
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
        None => error!(
            "Image {} has no remote version tags! Local tag: {}",
            image.reference, image.parts.tag
        ),
    }
}

/// Checks if a tag matches any of the excluded tag prefixes.
fn is_excluded_tag(tag: &str, excluded_tags: &[String], ctx: &Context) -> bool {
    for excluded in excluded_tags {
        if tag.starts_with(excluded) {
            ctx.logger.debug(format!(
                "Ignoring tag \"{}\" as it matches excluded prefix \"{}\"",
                tag, excluded
            ));
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
pub async fn get_extra_tags(
    url: &str,
    headers: &[(&str, Option<&str>)],
    base: &Version,
    format_str: &str,
    ctx: &Context,
    client: &Client,
    excluded_tags: &[String],
    ignored_update_type: UpdateType,
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
                .filter(|tag| !is_excluded_tag(tag.as_str().unwrap(), excluded_tags, ctx))
                .filter_map(|tag| Version::from_tag(tag.as_str().unwrap()))
                .filter(|(tag, format_string)| match (base.minor, tag.minor) {
                    (Some(_), Some(_)) | (None, None) => {
                        matches!((base.patch, tag.patch), (Some(_), Some(_)) | (None, None))
                            && format_str == *format_string
                    }
                    _ => false,
                })
                .filter_map(|(tag, _)| match ignored_update_type {
                    UpdateType::None => Some(tag),
                    UpdateType::Major => Some(tag).filter(|tag| base.major == tag.major),
                    UpdateType::Minor => {
                        Some(tag).filter(|tag| base.major == tag.major && base.minor == tag.minor)
                    }
                    UpdateType::Patch => Some(tag).filter(|tag| {
                        base.major == tag.major
                            && base.minor == tag.minor
                            && base.patch == tag.patch
                    }),
                })
                .dedup()
                .collect();
            Ok((result, next_url))
        }
        Err(message) => Err(message),
    }
}
