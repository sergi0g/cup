use std::{error::Error, fmt::Display, str::FromStr, sync::OnceLock};

use regex::Regex;

#[derive(PartialEq, Eq, Hash)]
pub struct Registry {
    value: String,
}

/// Regex representing the value of `domainAndPort` from https://github.com/distribution/reference/blob/727f80d42224f6696b8e1ad16b06aadf2c6b833b/regexp.go#L108
/// Used to verify whether a string is a registry hostname
const REGISTRY_REGEX: &str = r"^(?:(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])(?:\.(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]))*|\[(?:[a-fA-F0-9:]+)\])(?::[0-9]+)?$";
static REGISTRY: OnceLock<Regex> = OnceLock::new();

impl FromStr for Registry {
    type Err = ParseRegistryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = REGISTRY.get_or_init(|| Regex::new(REGISTRY_REGEX).unwrap());
        if s.is_empty() {
            Ok(Self {
                value: String::from("registry-1.docker.io"),
            })
        } else if regex.is_match(s) {
            Ok(Self {
                value: s.to_string(),
            })
        } else {
            Err(ParseRegistryError {
                registry: s.to_string(),
            })
        }
    }
}

#[derive(Debug)]
pub struct ParseRegistryError {
    pub registry: String,
}

impl Display for ParseRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` is not a valid registry hostname", self.registry)
    }
}

impl Error for ParseRegistryError {}

// TODO: Add tests