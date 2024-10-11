use std::path::PathBuf;

use json::{object, JsonValue};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

/// Sorts the update vector alphabetically and where Some(true) > Some(false) > None
pub fn sort_update_vec(updates: &[(String, Option<bool>)]) -> Vec<(String, Option<bool>)> {
    let mut sorted_updates = updates.to_vec();
    sorted_updates.sort_unstable_by(|a, b| match (a.1, b.1) {
        (Some(c), Some(d)) => {
            if c == d {
                a.0.cmp(&b.0)
            } else {
                (!c).cmp(&!d)
            }
        }
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.0.cmp(&b.0),
    });
    sorted_updates.to_vec()
}

/// Tries to load the config from the path provided and perform basic validation
pub fn load_config(config_path: Option<PathBuf>) -> JsonValue {
    let raw_config = match &config_path {
        Some(path) => std::fs::read_to_string(path),
        None => Ok(String::from("{\"theme\":\"default\"}")),
    };
    if raw_config.is_err() {
        panic!(
            "Failed to read config file from {}. Are you sure the file exists?",
            &config_path.unwrap().to_str().unwrap()
        )
    };
    match json::parse(&raw_config.unwrap()) {
        Ok(v) => v,
        Err(e) => panic!("Failed to parse config!\n{}", e),
    }
}

pub fn to_json(updates: &[(String, Option<bool>)]) -> JsonValue {
    let mut json_data: JsonValue = object! {
        metrics: object! {},
        images: object! {}
    };
    updates.iter().for_each(|(image, has_update)| {
        let _ = json_data["images"].insert(image, *has_update);
    });
    let up_to_date = updates
        .iter()
        .filter(|&(_, value)| *value == Some(false))
        .count();
    let update_available = updates
        .iter()
        .filter(|&(_, value)| *value == Some(true))
        .count();
    let unknown = updates.iter().filter(|&(_, value)| value.is_none()).count();
    let _ = json_data["metrics"].insert("monitored_images", updates.len());
    let _ = json_data["metrics"].insert("up_to_date", up_to_date);
    let _ = json_data["metrics"].insert("update_available", update_available);
    let _ = json_data["metrics"].insert("unknown", unknown);
    json_data
}

// Logging

/// This macro is an alternative to panic. It prints the message you give it and exits the process with code 1, without printing a stack trace. Useful for when the program has to exit due to a user error or something unexpected which is unrelated to the program (e.g. a failed web request)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        eprintln!("\x1b[38:5:204mERROR \x1b[0m {}", format!($($arg)*));
        std::process::exit(1);
    })
}

// A small macro to print in yellow as a warning
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        eprintln!("\x1b[38:5:192mWARN \x1b[0m {}", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        println!("\x1b[38:5:86mINFO \x1b[0m {}", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        println!("\x1b[38:5:63mDEBUG \x1b[0m {}", format!($($arg)*));
    })
}

pub fn new_reqwest_client() -> ClientWithMiddleware {
    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3),
        ))
        .build()
}
