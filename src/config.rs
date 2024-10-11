use std::path::PathBuf;

use rustc_hash::FxHashMap;

use crate::error;

const VALID_KEYS: [&str; 4] = ["authentication", "theme", "insecure_registries", "socket"];

#[derive(Clone)]
pub enum Theme {
    Default,
    Blue,
}

#[derive(Clone)]
pub struct Config {
    pub authentication: FxHashMap<String, String>,
    pub theme: Theme,
    pub insecure_registries: Vec<String>,
    pub socket: Option<String>,
}

impl Config {
    /// A stupid new function that exists just so calling `load` doesn't require a self argument
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            authentication: FxHashMap::default(),
            theme: Theme::Default,
            insecure_registries: Vec::with_capacity(0),
            socket: None
        }
    }
    /// Reads the config from the file path provided and returns the parsed result.
    pub fn load(&self, path: Option<PathBuf>) -> Self {
        let raw_config = match &path {
            Some(path) => std::fs::read_to_string(path),
            None => Ok(String::from("{}")), // Empty config
        };
        if raw_config.is_err() {
            panic!(
                "Failed to read config file from {}. Are you sure the file exists?",
                &path.unwrap().to_str().unwrap()
            )
        };
        self.parse(&raw_config.unwrap()) // We can safely unwrap here
    }
    /// Parses and validates the config. The process is quite manual and I would rather use a library, but I don't want to grow the dependency tree, for a config as simple as this one.
    /// Many of these checks are stupid, but we either validate the config properly, or we don't at all, so... this is the result. I _am not_ proud of this code.
    pub fn parse(&self, raw_config: &str) -> Self {
        let json = match json::parse(raw_config) {
            Ok(v) => v,
            Err(e) => panic!("Failed to parse config!\n{}", e),
        };
        // In the code, raw_<key> means the JsonValue from the parsed config, before it's validated.

        // Authentication
        let raw_authentication = &json["authentication"];
        if !raw_authentication.is_null() && !raw_authentication.is_object() {
            error!("Config key `authentication` must be an object!");
        }
        let mut authentication: FxHashMap<String, String> = FxHashMap::default();
        raw_authentication.entries().for_each(|(registry, key)| {
            if !key.is_string() {
                error!("Config key `authentication.{}` must be a string!", registry);
            }
            authentication.insert(registry.to_string(), key.to_string());
        });

        // Theme
        let raw_theme = &json["theme"];
        if !raw_theme.is_null() && !raw_theme.is_string() {
            error!("Config key `theme` must be a string!");
        }
        let theme: Theme = {
            if raw_theme.is_null() {
                Theme::Default
            } else {
                match raw_theme.as_str().unwrap() {
                    "default" => Theme::Default,
                    "blue" => Theme::Blue,
                    _ => {
                        error!("Config key `theme` must be one of: `default`, `blue`!");
                    }
                }
            }
        };

        // Insecure registries
        let raw_insecure_registries = &json["insecure_registries"];
        if !raw_insecure_registries.is_null() && !raw_insecure_registries.is_array() {
            error!("Config key `insecure_registries` must be an array!");
        }
        let insecure_registries: Vec<String> = raw_insecure_registries
            .members()
            .map(|registry| {
                if !registry.is_string() {
                    error!("Config key `insecure_registries` must only consist of strings!");
                } else {
                    registry.as_str().unwrap().to_owned()
                }
            })
            .collect();

        // Socket
        let raw_socket = &json["socket"];
        if !raw_socket.is_null() && !raw_socket.is_string() {
            error!("Config key `socket` must be a string!");
        }
        let socket: Option<String> = if raw_socket.is_null() {
            None
        } else {
            Some(raw_socket.to_string())
        };

        // Check for extra keys
        json.entries().for_each(|(key, _)| {
            if !VALID_KEYS.contains(&key) {
                error!("Invalid key `{}`", key)
            }
        });

        Self {
            authentication,
            theme,
            insecure_registries,
            socket,
        }
    }
}
