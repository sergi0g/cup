use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::env;
use std::mem;
use std::path::PathBuf;

use crate::error;

// We can't assign `a` to `b` in the loop in `Config::load`, so we'll have to use swap. It looks ugly so now we have a macro for it.
macro_rules! swap {
    ($a:expr, $b:expr) => {
        mem::swap(&mut $a, &mut $b)
    };
}

#[derive(Clone, Deserialize)]
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

#[derive(Clone, Deserialize)]
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
    pub ignore_update_type: UpdateType,
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
    pub fn load(&mut self, path: Option<PathBuf>) -> Self {
        let mut config = self.load_file(path);

        // Get environment variables with CUP_ prefix
        let env_vars: FxHashMap<String, String> =
            env::vars().filter(|(k, _)| k.starts_with("CUP_")).collect();

        if !env_vars.is_empty() {
            if let Ok(mut cfg) = envy::prefixed("CUP_").from_env::<Config>() {
                // If we have environment variables, override config.json options
                for (key, _) in env_vars {
                    match key.as_str() {
                        "CUP_AGENT" => config.agent = cfg.agent,
                        #[rustfmt::skip]
                        "CUP_IGNORE_UPDATE_TYPE" => swap!(config.ignore_update_type, cfg.ignore_update_type),
                        #[rustfmt::skip]
                        "CUP_REFRESH_INTERVAL" => swap!(config.refresh_interval, cfg.refresh_interval),
                        "CUP_SOCKET" => swap!(config.socket, cfg.socket),
                        "CUP_THEME" => swap!(config.theme, cfg.theme),
                        // The syntax for these is slightly more complicated, not sure if they should be enabled or not. Let's stick to simple types for now.
                        // "CUP_IMAGES" => swap!(config.images, cfg.images),
                        // "CUP_REGISTRIES" => swap!(config.registries, cfg.registries),
                        // "CUP_SERVERS" => swap!(config.servers, cfg.servers),
                        _ => (), // Maybe print a warning if other CUP_ variables are detected
                    }
                }
            }
        }

        config
    }

    /// Reads the config from the file path provided and returns the parsed result.
    fn load_file(&self, path: Option<PathBuf>) -> Self {
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
    fn parse(&self, raw_config: &str) -> Self {
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
