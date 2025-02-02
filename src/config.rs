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

#[derive(Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct RegistryConfig {
    pub authentication: Option<String>,
    pub insecure: bool,
    pub ignore: bool,
}

#[derive(Clone, Deserialize, Default)]
#[serde(default)]
pub struct ImageConfig {
    pub extra: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    version: u8,
    pub agent: bool,
    #[serde(skip_deserializing)]
    pub debug: bool,
    pub images: ImageConfig,
    pub refresh_interval: Option<String>,
    pub registries: FxHashMap<String, RegistryConfig>,
    pub servers: FxHashMap<String, String>,
    pub socket: Option<String>,
    pub theme: Theme,
}

impl Config {
    pub fn new() -> Self {
        Self {
            version: 3,
            agent: false,
            debug: false,
            images: ImageConfig::default(),
            refresh_interval: None,
            registries: FxHashMap::default(),
            servers: FxHashMap::default(),
            socket: None,
            theme: Theme::Default,
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
        let config: Self = match serde_json::from_str(raw_config) {
            Ok(config) => config,
            Err(e) => error!("Unexpected error occured while parsing config: {}", e),
        };
        if config.version != 3 {
            error!("You are trying to run Cup with an incompatible config file! Please migrate your config file to the version 3, or if you have already done so, add a `version` key with the value `3`.")
        }
        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
