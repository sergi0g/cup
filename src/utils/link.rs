use std::str::FromStr;

use http_link::parse_link_header;
use reqwest::Url;

use crate::error;

pub fn parse_link(link: &str, base: &str) -> String {
    match parse_link_header(link, &Url::from_str(base).unwrap()) {
        Ok(l) => l[0].target.to_string(),
        Err(e) => error!("Failed to parse link! {}", e),
    }
}
