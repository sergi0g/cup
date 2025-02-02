use http_auth::parse_challenges;
use reqwest::Response;
use rustc_hash::FxHashMap;
use serde_json::Value;

use crate::{config::RegistryConfig, error};

/// Parses the www-authenticate header the registry sends into a challenge URL
pub fn parse_www_authenticate(www_auth: &str) -> String {
    let challenges = parse_challenges(www_auth).unwrap();
    if !challenges.is_empty() {
        let challenge = &challenges[0];
        if challenge.scheme == "Bearer" {
            challenge
                .params
                .iter()
                .fold(String::new(), |acc, (key, value)| {
                    if *key == "realm" {
                        return acc.to_owned() + value.as_escaped() + "?";
                    } else {
                        return format!("{}&{}={}", acc, key, value.as_escaped());
                    }
                })
        } else {
            error!("Unsupported scheme {}", &challenge.scheme)
        }
    } else {
        error!("No challenge provided by the server");
    }
}

pub fn get_protocol(
    registry: &str,
    registry_config: &FxHashMap<String, RegistryConfig>,
) -> &'static str {
    match registry_config.get(registry) {
        Some(config) => {
            if config.insecure {
                "http"
            } else {
                "https"
            }
        }
        None => "https",
    }
}

pub fn to_bearer_string(token: &Option<&str>) -> Option<String> {
    token.as_ref().map(|t| format!("Bearer {}", t))
}

pub async fn get_response_body(response: Response) -> String {
    match response.text().await {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to parse registry response into string!\n{}", e)
        }
    }
}

pub fn parse_json(body: &str) -> Value {
    match serde_json::from_str(body) {
        Ok(parsed) => parsed,
        Err(e) => {
            error!("Failed to parse server response\n{}", e)
        }
    }
}
