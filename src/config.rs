use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde::Deserializer;
use std::env;
use std::path::PathBuf;

use crate::error;

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Default,
    Blue,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Default
    }
}

impl std::str::FromStr for Theme {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(Self::Default),
            "blue" => Ok(Self::Blue),
            _ => Err(format!("Invalid theme: {}", s)),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum UpdateType {
    None,
    Major,
    Minor,
    Patch,
}

impl Default for UpdateType {
    fn default() -> Self {
        Self::None
    }
}

impl std::str::FromStr for UpdateType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            _ => Err(format!("Invalid update type: {}", s)),
        }
    }
}

#[derive(Clone, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct RegistryConfig {
    pub authentication: Option<String>,
    pub insecure: bool,
    pub ignore: bool,
}

#[derive(Clone, Deserialize, Default, Debug)]
#[serde(default)]
pub struct ImageConfig {
    pub extra: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    version: u8,
    pub agent: bool,
    pub ignore_update_type: UpdateType,
    pub images: ImageConfig,
    #[serde(deserialize_with = "empty_as_none")]
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
            ignore_update_type: UpdateType::default(),
            images: ImageConfig::default(),
            refresh_interval: None,
            registries: FxHashMap::default(),
            servers: FxHashMap::default(),
            socket: None,
            theme: Theme::Default,
        }
    }

    /// Loads file and env config and merges them
    pub fn load(path: Option<PathBuf>) -> Self {
        Config::load_file(path).load_env()
    }

    /// Reads the config from the file path provided and returns the parsed result.
    fn load_file(path: Option<PathBuf>) -> Self {
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
        Config::parse(&raw_config.unwrap()) // We can safely unwrap here
    }
    /// Parses and validates the config.
    fn parse(raw_config: &str) -> Self {
        let config: Self = match serde_json::from_str(raw_config) {
            Ok(config) => config,
            Err(e) => error!("Unexpected error occured while parsing config: {}", e),
        };
        if config.version != 3 {
            error!("You are trying to run Cup with an incompatible config file! Please migrate your config file to the version 3, or if you have already done so, add a `version` key with the value `3`.")
        }
        config
    }

    /// Read and parse environment variables into the config object and return it.
    fn load_env(mut self) -> Self {
        env::vars()
            .filter(|(k, _)| k.starts_with("CUP_"))
            .for_each(|(key, value)| match key.as_str() {
                "CUP_AGENT" => self.agent = value.parse().unwrap(),
                "CUP_IGNORE_UPDATE_TYPE" => self.ignore_update_type = value.parse().unwrap(),
                "CUP_REFRESH_INTERVAL" => self.refresh_interval = Some(value),
                "CUP_SOCKET" => self.socket = Some(value),
                "CUP_THEME" => self.theme = value.parse().unwrap(),
                "CUP_IMAGES_EXCLUDE" => {
                    self.images.exclude = value
                        .split(' ')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                }
                "CUP_IMAGES_EXTRA" => {
                    self.images.extra = value
                        .split(' ')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                }
                // "CUP_REGISTRIES" => ...
                // "CUP_SERVERS" => ...
                _ => println!("Warning: Skip unknown CUP_ variable '{}'.", key),
            });
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

fn empty_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}
