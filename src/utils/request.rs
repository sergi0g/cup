use http_auth::parse_challenges;
use json::JsonValue;
use reqwest::Response;

use crate::error;

/// Parses the www-authenticate header the registry sends into a challenge URL
pub fn parse_www_authenticate(www_auth: &str) -> String {
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

pub fn get_protocol(registry: &String, insecure_registries: &[String]) -> String {
    if insecure_registries.contains(registry) {
        "http"
    } else {
        "https"
    }
    .to_string()
}

pub fn to_bearer_string(token: &Option<&String>) -> Option<String> {
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

pub fn parse_json(body: &str) -> JsonValue {
    match json::parse(body) {
        Ok(parsed) => parsed,
        Err(e) => {
            error!("Failed to parse server response\n{}", e)
        }
    }
}
