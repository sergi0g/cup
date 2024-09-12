use std::path::PathBuf;

use crate::{error, image::Image};
use json::{object, JsonValue};
use once_cell::sync::Lazy;
use regex::Regex;

/// Takes an image and splits it into registry, repository and tag. For example ghcr.io/sergi0g/cup:latest becomes ['ghcr.io', 'sergi0g/cup', 'latest'].
pub fn split_image(image: &str) -> (String, String, String) {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"^(?P<name>(?:(?P<registry>(?:(?:localhost|[\w-]+(?:\.[\w-]+)+)(?::\d+)?)|[\w]+:\d+)/)?(?P<repository>[a-z0-9_.-]+(?:/[a-z0-9_.-]+)*))(?::(?P<tag>[\w][\w.-]{0,127}))?$"#, // From https://regex101.com/r/nmSDPA/1
        )
        .unwrap()
    });
    match RE.captures(image) {
        Some(c) => {
            let registry = match c.name("registry") {
                Some(registry) => registry.as_str().to_owned(),
                None => String::from("registry-1.docker.io"),
            };
            return (
                registry.clone(),
                match c.name("repository") {
                    Some(repository) => {
                        let repo = repository.as_str().to_owned();
                        if !repo.contains('/') && registry == "registry-1.docker.io" {
                            format!("library/{}", repo)
                        } else {
                            repo
                        }
                    }
                    None => error!("Failed to parse image {}", image),
                },
                match c.name("tag") {
                    Some(tag) => tag.as_str().to_owned(),
                    None => String::from("latest"),
                },
            );
        }
        None => error!("Failed to parse image {}", image),
    }
}

/// Given an image's parts which were previously created by split_image, recreate a reference that docker would use. This means removing the registry part, if it's Docker Hub and removing "library" if the image is official
pub fn unsplit_image(image: &Image) -> String {
    let reg = match image.registry.as_str() {
        "registry-1.docker.io" => String::new(),
        r => format!("{}/", r),
    };
    let repo = match image.repository.split('/').collect::<Vec<&str>>()[0] {
        "library" => {
            if reg.is_empty() {
                image.repository.strip_prefix("library/").unwrap()
            } else {
                image.repository.as_str()
            }
        }
        _ => image.repository.as_str(),
    };
    format!("{}{}:{}", reg, repo, image.tag)
}

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
        .collect::<Vec<&(String, Option<bool>)>>()
        .len();
    let update_available = updates
        .iter()
        .filter(|&(_, value)| *value == Some(true))
        .collect::<Vec<&(String, Option<bool>)>>()
        .len();
    let unknown = updates
        .iter()
        .filter(|&(_, value)| value.is_none())
        .collect::<Vec<&(String, Option<bool>)>>()
        .len();
    let _ = json_data["metrics"].insert("monitored_images", updates.len());
    let _ = json_data["metrics"].insert("up_to_date", up_to_date);
    let _ = json_data["metrics"].insert("update_available", update_available);
    let _ = json_data["metrics"].insert("unknown", unknown);
    json_data
}

/// Struct to hold some config values to avoid having to pass them all the time
#[derive(Clone)]
pub struct CliConfig {
    pub socket: Option<String>,
    pub verbose: bool,
    pub config: JsonValue,
}

// Logging

/// This macro is an alternative to panic. It prints the message you give it and exits the process with code 1, without printing a stack trace. Useful for when the program has to exit due to a user error or something unexpected which is unrelated to the program (e.g. a failed web request)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        eprintln!("\x1b[41m ERROR \x1b[0m {}", format!($($arg)*));
        std::process::exit(1);
    })
}

// A small macro to print in yellow as a warning
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        eprintln!("\x1b[103m WARN  \x1b[0m {}", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        println!("\x1b[44m INFO  \x1b[0m {}", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        println!("\x1b[48:5:57m DEBUG \x1b[0m {}", format!($($arg)*));
    })
}
