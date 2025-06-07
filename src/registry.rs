use std::{cmp::Ordering, time::SystemTime};

use itertools::Itertools;

use crate::{
    config::UpdateType,
    error,
    http::Client,
    structs::{image::Image, version::Version},
    utils::{
        link::parse_link,
        request::{
            get_protocol, get_response_body, parse_json, parse_www_authenticate, to_bearer_string,
        },
        time::{elapsed, now},
    },
    Context,
};

pub async fn check_auth(registry: &str, ctx: &Context, client: &Client) -> Option<String> {
    let protocol = get_protocol(registry, &ctx.config.registries);
    let url = format!("{}://{}/v2/", protocol, registry);
    let response = client.get(&url, &[], true).await;
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
    image: &mut Image,
    token: Option<&str>,
    ctx: &Context,
    client: &Client,
) -> () {
    ctx.logger
        .debug(format!("Checking for digest update to {}", image.reference));
    let start = SystemTime::now();
    let protocol = get_protocol(&image.parts.registry, &ctx.config.registries);
    let url = format!(
        "{}://{}/v2/{}/manifests/{}",
        protocol, &image.parts.registry, &image.parts.repository, &image.parts.tag
    );
    let authorization = to_bearer_string(&token);
    let headers = [("Accept", Some("application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.oci.image.index.v1+json")), ("Authorization", authorization.as_deref())];

    let response = client.head(&url, &headers).await;
    let time = start.elapsed().unwrap().as_millis() as u32;
    ctx.logger.debug(format!(
        "Checked for digest update to {} in {}ms",
        image.reference, time
    ));
    match response {
        Ok(res) => match res.headers().get("docker-content-digest") {
            Some(digest) => {
                image.update_info.remote_digest = Some(digest.to_str().unwrap().to_owned());
            }
            None => error!(
                "Server returned invalid response! No docker-content-digest!\n{:#?}",
                res
            ),
        },
        Err(error) => {
            image.error = Some(error);
            image.time_ms = image.time_ms + elapsed(start)
        }
    }
}

pub async fn get_token(
    images: &[&Image],
    auth_url: &str,
    credentials: &Option<String>,
    client: &Client,
) -> String {
    let mut url = auth_url.to_owned();
    for image in images {
        url = format!("{}&scope=repository:{}:pull", url, image.parts.repository);
    }
    let authorization = credentials.as_ref().map(|creds| format!("Basic {}", creds));
    let headers = [("Authorization", authorization.as_deref())];

    let response = client.get(&url, &headers, false).await;
    let response_json = match response {
        Ok(response) => parse_json(&get_response_body(response).await),
        Err(_) => error!("GET {}: Request failed!", url),
    };
    response_json["token"].as_str().unwrap().to_string()
}

pub async fn get_latest_tag(
    image: &mut Image,
    base: &Version,
    token: Option<&str>,
    ctx: &Context,
    client: &Client,
) -> () {
    ctx.logger
        .debug(format!("Checking for tag update to {}", image.reference));
    let start = now();
    let protocol = get_protocol(&image.parts.registry, &ctx.config.registries);
    let url = format!(
        "{}://{}/v2/{}/tags/list",
        protocol, &image.parts.registry, &image.parts.repository,
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
            &image.reference,
            ctx,
            client,
        )
        .await
        {
            Ok(t) => t,
            Err(message) => {
                image.error = Some(message);
                image.time_ms += elapsed(start);
                return;
            }
        };
        tags.extend_from_slice(&new_tags);
        next_url = next;
    }
    let tag = tags.iter().reduce(|a, b| match a.partial_cmp(b) {
        Some(ordering) => match ordering {
            Ordering::Greater => a,
            Ordering::Equal => b,
            Ordering::Less => b,
        },
        None => unreachable!(),
    });
    ctx.logger.debug(format!(
        "Checked for tag update to {} in {}ms",
        image.reference,
        elapsed(start)
    ));
    match tag {
        Some(t) => {
            if t == base && !image.info.local_digests.is_empty() {
                // Tags are equal so we'll compare digests
                ctx.logger.debug(format!(
                    "Tags for {} are equal, comparing digests.",
                    image.reference
                ));
                image.time_ms += elapsed(start);
                get_latest_digest(image, token, ctx, client).await
            } else {
                image.update_info.latest_version = Some(t.clone());
                image.time_ms += elapsed(start);
            }
        }
        None => error!(
            "Image {} has no remote version tags! Local tag: {}",
            image.reference, image.parts.tag
        ),
    }
}

pub async fn get_extra_tags(
    url: &str,
    headers: &[(&str, Option<&str>)],
    base: &Version,
    reference: &str,
    ctx: &Context,
    client: &Client,
) -> Result<(Vec<Version>, Option<String>), String> {
    let response = client.get(url, headers, false).await;
    let base_type = base.r#type();
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
                .map(|tag| Version::from(tag.as_str().unwrap(), base_type.as_ref()))
                .filter(|tag| tag.r#type() == base_type)
                .filter(|tag| tag.partial_cmp(base).is_some())
                .filter_map(|tag| {
                    match ctx
                        .config
                        .images
                        .iter()
                        .filter(|&(i, _)| reference.starts_with(i))
                        .sorted_by(|(a, _), (b, _)| a.len().cmp(&b.len()))
                        .next()
                        .map(|(_, cfg)| &cfg.ignore)
                        .unwrap_or(&UpdateType::None)
                    {
                        // TODO: Please don't ship it like this
                        UpdateType::None => Some(tag),
                        UpdateType::Major => Some(tag).filter(|tag| {
                            base.as_standard().unwrap().major == tag.as_standard().unwrap().major
                        }),
                        UpdateType::Minor => Some(tag).filter(|tag| {
                            base.as_standard().unwrap().major == tag.as_standard().unwrap().major
                                && base.as_standard().unwrap().minor
                                    == tag.as_standard().unwrap().minor
                        }),
                        UpdateType::Patch => Some(tag).filter(|tag| {
                            base.as_standard().unwrap().major == tag.as_standard().unwrap().major
                                && base.as_standard().unwrap().minor
                                    == tag.as_standard().unwrap().minor
                                && base.as_standard().unwrap().patch
                                    == tag.as_standard().unwrap().patch
                        }),
                    }
                })
                .dedup()
                .collect();
            Ok((result, next_url))
        }
        Err(message) => Err(message),
    }
}
