use std::{collections::HashMap, path::PathBuf};

use json::JsonValue;
use once_cell::sync::Lazy;
use regex::Regex;

/// This macro is an alternative to panic. It prints the message you give it and exits the process with code 1, without printing a stack trace. Useful for when the program has to exit due to a user error or something unexpected which is unrelated to the program (e.g. a failed web request)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        std::process::exit(1);
    })
}

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
            return (
                match c.name("registry") {
                    Some(registry) => registry.as_str().to_owned(),
                    None => String::from("registry-1.docker.io"),
                },
                match c.name("repository") {
                    Some(repository) => {
                        let repo = repository.as_str().to_owned();
                        if !repo.contains('/') {
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
            )
        }
        None => error!("Failed to parse image {}", image),
    }
}

/// Given an image's parts which were previously created by split_image, recreate a reference that docker would use. This means removing the registry part, if it's Docker Hub and removing "library" if the image is official
pub fn unsplit_image(registry: &str, repository: &str, tag: &str) -> String {
    let reg = match registry {
        "registry-1.docker.io" => String::new(),
        r => format!("{}/", r),
    };
    let repo = match repository.split('/').collect::<Vec<&str>>()[0] {
        "library" => {
            if reg.is_empty() {
                repository.strip_prefix("library/").unwrap()
            } else {
                repository
            }
        }
        _ => repository,
    };
    format!("{}{}:{}", reg, repo, tag)
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
pub fn load_config(config_path: Option<PathBuf>) -> Config {
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
    let config = match json::parse(&raw_config.unwrap()) {
        Ok(v) => v,
        Err(e) => panic!("Failed to parse config!\n{}", e),
    };
    // Very basic validation
    const TOP_LEVEL_KEYS: [&str; 2] = ["authentication", "theme"];
    let themes: JsonValue = json::object! {default: "neutral", blue: "gray"};
    for (key, _) in config.entries() {
        if !TOP_LEVEL_KEYS.contains(&key) {
            error!("Config contains invalid key {}", key)
        }
    }
    if config.has_key("authentication") && !config["authentication"].is_object() {
        error!("\"{}\" must be an object", "authentication")
    }
    for (registry, token) in config["authentication"].entries() {
        if !token.is_string() {
            error!(
                "Invalid token {} for registry {}. Must be a string",
                token, registry
            )
        }
    }
    if !themes.has_key(&config["theme"].to_string()) {
        error!(
            "Invalid theme {}. Available themes are {:#?}",
            config["theme"],
            themes.entries().map(|(k, _)| k).collect::<Vec<&str>>()
        )
    }
    return Config {
        authentication: HashMap::new(),
        theme: themes[config["theme"].to_string()].to_string(),
    };
}

pub struct Config {
    pub authentication: HashMap<String, String>,
    pub theme: String,
}
