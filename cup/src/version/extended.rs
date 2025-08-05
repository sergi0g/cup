use std::num::ParseIntError;
use std::str::FromStr;
use std::{error::Error, fmt::Display};

use regex::Regex;
use rustc_hash::FxHashMap;

use crate::version::version_component::VersionComponent;

/// This doesn't describe a specific versioning scheme, but instead aims to provide the utilites for parsing a version string not covered by any of the other parsers in the crate.
/// Takes a regex with a capture group for each component (either _all_ named or anonymous)
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ExtendedVersion {
    components: FxHashMap<String, VersionComponent>,
}

impl ExtendedVersion {
    pub fn parse(version_string: &str, regex: &str) -> Result<Self, VersionParseError> {
        let regex = Regex::new(regex).map_err(|e| VersionParseError {
            version_string: version_string.to_string(),
            kind: VersionParseErrorKind::InvalidRegex(e),
        })?;

        // Check if all capture groups are named or anonymous
        let is_named = match regex
            .capture_names()
            .skip(1) // The first group will always be the implicit anonymous group for the whole regex
            .try_fold(None, |prev_state, name| {
                match prev_state {
                    None => Ok(Some(name)), // First iteration
                    Some(prev_state) => match (prev_state, name) {
                        (Some(_), None) | (None, Some(_)) => Err(VersionParseError {
                            version_string: version_string.to_string(),
                            kind: VersionParseErrorKind::NonUniformCaptureGroups,
                        }),
                        _ => Ok(Some(name)),
                    },
                }
            }) {
            Ok(Some(Some(_))) => true,
            Ok(Some(None)) => false,
            Ok(None) => false,
            Err(e) => return Err(e),
        };

        // Parse the version string
        Ok(match regex.captures(version_string) {
            Some(captures) => {
                let components = if is_named {
                    regex
                        .capture_names()
                        .flatten()
                        .map(|name| {
                            let capture = captures.name(name).ok_or_else(|| VersionParseError {
                                version_string: version_string.to_string(),
                                kind: VersionParseErrorKind::GroupDidNotMatch(name.to_string()),
                            })?;
                            let component =
                                VersionComponent::from_str(capture.as_str()).map_err(|_| {
                                    VersionParseError {
                                        version_string: version_string.to_string(),
                                        kind: VersionParseErrorKind::GroupDidNotMatch(
                                            name.to_string(),
                                        ),
                                    }
                                })?;
                            Ok((name.to_string(), component))
                        })
                        .collect::<Result<FxHashMap<String, VersionComponent>, VersionParseError>>(
                        )?
                } else {
                    captures
                        .iter()
                        .enumerate()
                        .skip(1) // skip group 0 (whole match)
                        .filter_map(|(i, m)| m.map(|mat| (i.to_string(), mat)))
                        .map(|(i, m)| {
                            VersionComponent::from_str(m.as_str())
                                .map(|comp| (i, comp))
                                .map_err(|e| VersionParseError {
                                    version_string: version_string.to_string(),
                                    kind: VersionParseErrorKind::ParseComponent(e),
                                })
                        })
                        .collect::<Result<FxHashMap<String, VersionComponent>, VersionParseError>>(
                        )?
                };
                Self { components }
            }
            None => {
                return Err(VersionParseError {
                    version_string: version_string.to_string(),
                    kind: VersionParseErrorKind::NoMatch,
                });
            }
        })
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[non_exhaustive]
pub struct VersionParseError {
    version_string: String,
    kind: VersionParseErrorKind,
}

impl Display for VersionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to parse `{}` as extended version",
            self.version_string
        )
    }
}

impl Error for VersionParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum VersionParseErrorKind {
    /// The regex supplied could not be compiled
    InvalidRegex(regex::Error),
    /// The regex supplied has both named and anonymous capture groups
    NonUniformCaptureGroups,
    /// The version string did not match the regex supplied
    NoMatch,
    /// A named group had no matches
    GroupDidNotMatch(String),
    /// A version component could not be parsed
    ParseComponent(ParseIntError),
}

impl Display for VersionParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRegex(_) => write!(f, "invalid regex"),
            Self::NonUniformCaptureGroups => {
                write!(f, "regex has both named and anonymous capture groups")
            }
            Self::NoMatch => write!(f, "version string did not match the regex"),
            Self::GroupDidNotMatch(name) => {
                write!(f, "named group `{}` did not match", name)
            }
            Self::ParseComponent(_) => write!(f, "version component is not a valid integer"),
        }
    }
}

impl Error for VersionParseErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRegex(e) => Some(e),
            Self::ParseComponent(e) => Some(e),
            Self::GroupDidNotMatch(_) => None,
            Self::NoMatch => None,
            Self::NonUniformCaptureGroups => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use rustc_hash::FxHashMap;

    use crate::version::version_component::VersionComponent;

    use super::ExtendedVersion;
    use super::VersionParseError;
    use super::VersionParseErrorKind;

    #[test]
    fn parse_with_anonymous() {
        assert_eq!(
            ExtendedVersion::parse("24.04.11.2.1", r"(\d+)\.(\d+)\.(\d+)\.(\d+)\.(\d+)"),
            Ok(ExtendedVersion {
                components: {
                    let mut map = FxHashMap::default();
                    map.insert(
                        "1".to_string(),
                        VersionComponent {
                            value: 24,
                            length: 2,
                        },
                    );
                    map.insert(
                        "2".to_string(),
                        VersionComponent {
                            value: 4,
                            length: 2,
                        },
                    );
                    map.insert(
                        "3".to_string(),
                        VersionComponent {
                            value: 11,
                            length: 2,
                        },
                    );
                    map.insert(
                        "4".to_string(),
                        VersionComponent {
                            value: 2,
                            length: 1,
                        },
                    );
                    map.insert(
                        "5".to_string(),
                        VersionComponent {
                            value: 1,
                            length: 1,
                        },
                    );
                    map
                }
            })
        )
    }

    #[test]
    fn parse_with_named() {
        assert_eq!(
            ExtendedVersion::parse(
                "v0.7.1-ls84",
                r"v(?<Alpha>\d+)\.(?<Beta>\d+)\.(?<Gamma>\d+)-ls(?<Delta>\d+)"
            ),
            Ok(ExtendedVersion {
                components: {
                    let mut map = FxHashMap::default();
                    map.insert(
                        "Alpha".to_string(),
                        VersionComponent {
                            value: 0,
                            length: 1,
                        },
                    );
                    map.insert(
                        "Beta".to_string(),
                        VersionComponent {
                            value: 7,
                            length: 1,
                        },
                    );
                    map.insert(
                        "Gamma".to_string(),
                        VersionComponent {
                            value: 1,
                            length: 1,
                        },
                    );
                    map.insert(
                        "Delta".to_string(),
                        VersionComponent {
                            value: 84,
                            length: 2,
                        },
                    );
                    map
                }
            })
        )
    }

    #[test]
    fn invalid_regex() {
        assert_eq!(
            ExtendedVersion::parse(
                "1.0.0-test2",
                r"(\d+)\.(\d+))\.(\d+)-test(\d+)" // Whoops, someone left an extra ) somewhere...
            ),
            Err(VersionParseError {
                version_string: String::from("1.0.0-test2"),
                kind: VersionParseErrorKind::InvalidRegex(
                    Regex::new(r"(\d+)\.(\d+))\.(\d+)-test(\d+)").unwrap_err()
                )
            })
        )
    }

    #[test]
    fn invalid_component() {
        assert_eq!(
            ExtendedVersion::parse("50h+2-3_40", r"([a-z0-9]+)\+(\d+)-(\d+)_(\d+)"),
            Err(VersionParseError {
                version_string: String::from("50h+2-3_40"),
                kind: VersionParseErrorKind::ParseComponent("50h".parse::<usize>().unwrap_err())
            })
        )
    }

    #[test]
    fn non_uniform_groups() {
        assert_eq!(
            ExtendedVersion::parse(
                "4.1.2.5",
                r"(?<Major>\d+)\.(?<Minor>\d+)\.(?<Patch>\d+)\.(\d+)"
            ),
            Err(VersionParseError {
                version_string: String::from("4.1.2.5"),
                kind: VersionParseErrorKind::NonUniformCaptureGroups
            })
        )
    }

    #[test]
    fn no_match() {
        assert_eq!(
            ExtendedVersion::parse(
                "1-sometool-0.2.5-alpine",
                r"(?:\d+)-somet0ol-(\d+)\.(\d+)\.(\d+)-alpine"
            ),
            Err(VersionParseError {
                version_string: String::from("1-sometool-0.2.5-alpine"),
                kind: VersionParseErrorKind::NoMatch
            })
        )
    }

    #[test]
    fn no_group_match() {
        assert_eq!(
            ExtendedVersion::parse(
                "2.0.5-alpine",
                r"(?<Eenie>\d+)\.(?<Meenie>\d+)\.(?<Miney>\d+)|(?<Moe>moe)-alpine"
            ),
            Err(VersionParseError {
                version_string: String::from("2.0.5-alpine"),
                kind: VersionParseErrorKind::GroupDidNotMatch(String::from("Moe"))
            })
        )
    }

    // TODO: Maybe check that if an anonymous group does not match everything is fine
}
