use std::sync::Mutex;

use json::JsonValue;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ureq::Error;

use http_auth::parse_challenges;

use crate::{error, image::Image};

pub fn check_auth(registry: &str) -> Option<String> {
    let response = ureq::get(&format!("https://{}/v2/", registry)).call();
    match response {
        Ok(_) => None,
        Err(Error::Status(401, response)) => match response.header("www-authenticate") {
            Some(challenge) => Some(parse_www_authenticate(challenge)),
            None => error!("Server returned invalid response!"),
        },
        Err(e) => error!("{}", e),
    }
}

pub fn get_latest_digest(image: &Image, token: Option<&String>) -> Image {
    let mut request = ureq::head(&format!(
        "https://{}/v2/{}/manifests/{}",
        &image.registry, &image.repository, &image.tag
    ));
    if let Some(t) = token {
        request = request.set("Authorization", &format!("Bearer {}", t));
    }
    let raw_response = match request
        .set("Accept", "application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.oci.image.index.v1+json")
        .call()
    {
        Ok(response) => response,
        Err(Error::Status(401, response)) => {
            if token.is_some() {
                error!("Failed to authenticate to registry {} with given token!\n{}", &image.registry, token.unwrap())
            } else {
                return get_latest_digest(
                    image,
                    Some(&get_token(
                        vec![image],
                        &parse_www_authenticate(response.header("www-authenticate").unwrap()),
                    )),
                );
            }
        }
        Err(Error::Status(_, _)) => {
            return Image {
                digest: None,
                ..image.clone()
            }
        }
        Err(ureq::Error::Transport(e)) => error!("Failed to send request!\n{}", e),
    };
    match raw_response.header("docker-content-digest") {
        Some(digest) => Image {
            digest: Some(digest.to_string()),
            ..image.clone()
        },
        None => error!("Server returned invalid response! No docker-content-digest!"),
    }
}

pub fn get_latest_digests(images: Vec<&Image>, token: Option<&String>) -> Vec<Image> {
    let result: Mutex<Vec<Image>> = Mutex::new(Vec::new());
    images.par_iter().for_each(|&image| {
        let digest = get_latest_digest(image, token).digest;
        result.lock().unwrap().push(Image {
            digest,
            ..image.clone()
        });
    });
    let r = result.lock().unwrap().clone();
    r
}

pub fn get_token(images: Vec<&Image>, auth_url: &str) -> String {
    let mut final_url = auth_url.to_owned();
    for image in images {
        final_url = format!("{}&scope=repository:{}:pull", final_url, image.repository);
    }
    let raw_response = match ureq::get(&final_url)
        .set("Accept", "application/vnd.oci.image.index.v1+json")
        .call()
    {
        Ok(response) => match response.into_string() {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to parse response into string!\n{}", e)
            }
        },
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
        error!("No challenge provided");
    }
}
