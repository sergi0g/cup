use std::path::PathBuf;

use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::error;

#[derive(Clone, Deserialize)]
pub enum Theme {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "blue")]
    Blue,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "FxHashMap::default")]
    pub authentication: FxHashMap<String, String>,
    #[serde(default = "Theme::default")]
    pub theme: Theme,
    #[serde(default = "Vec::default")]
    pub insecure_registries: Vec<String>,
    pub socket: Option<String>,
    #[serde(skip_deserializing)]
    pub debug: bool,
}

impl Config {
    /// A stupid new function that exists just so calling `load` doesn't require a self argument
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            authentication: FxHashMap::default(),
            theme: Theme::Default,
            insecure_registries: Vec::with_capacity(0),
            socket: None,
            debug: false,
        }
    }
    /// Reads the config from the file path provided and returns the parsed result.
    pub fn load(&self, path: Option<PathBuf>) -> Self {
        let raw_config = match &path {
            Some(path) => std::fs::read_to_string(path),
            None => return Self::new(), // Empty config
        };
        if raw_config.is_err() {
            error!(
                "Failed to read config file from {}. Are you sure the file exists?",
                &path.unwrap().to_str().unwrap()
            )
        };
        self.parse(&raw_config.unwrap()) // We can safely unwrap here
    }
    /// Parses and validates the config.
    pub fn parse(&self, raw_config: &str) -> Self {
        match serde_json::from_str(raw_config) {
            Ok(config) => config,
            Err(e) => error!("Unexpected error occured while parsing config: {}", e),
        }
    }
}
