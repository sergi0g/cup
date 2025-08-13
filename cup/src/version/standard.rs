use std::{error::Error, fmt::Display, num::ParseIntError, str::FromStr};

use rustc_hash::FxHashSet;

use super::version_component::VersionComponent;

/// A versioning scheme I'd call SemVer-inspired. The main difference from [SemVer](https://semver.org) is that the minor and patch versions are optional.
/// It describes a version that is made up of one to three numeric components named `major`, `minor` and `patch`, separated by dots (`.`). Numbers can be prefixed by any number of zeroes.
/// In practice, this versioning scheme works well for most versioned images available and is a good out-of-the-box default.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct StandardVersion {
    major: VersionComponent,
    minor: Option<VersionComponent>,
    patch: Option<VersionComponent>,
}

impl FromStr for StandardVersion {
    type Err = ParseVersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split('.');
        if splits.clone().any(|split| split.is_empty()) {
            return Err(ParseVersionError {
                version_string: s.to_string(),
                kind: ParseVersionErrorKind::IncorrectFormat,
            });
        }
        let mut component_iter = splits.map(|component| {
            VersionComponent::from_str(component).map_err(|e| ParseVersionError {
                version_string: s.to_string(),
                kind: ParseVersionErrorKind::ParseComponent(e),
            })
        });
        let major = component_iter.next().transpose()?.unwrap();
        let minor = component_iter.next().transpose()?;
        let patch = component_iter.next().transpose()?;
        if component_iter.next().is_some() {
            return Err(ParseVersionError {
                version_string: s.to_string(),
                kind: ParseVersionErrorKind::TooManyComponents(4 + component_iter.count()),
            });
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum StandardUpdateType {
    Major,
    Minor,
    Patch
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[non_exhaustive]
pub struct ParseVersionError {
    pub version_string: String,
    pub kind: ParseVersionErrorKind,
}

impl Display for ParseVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to parse `{}` as standard version",
            self.version_string
        )
    }
}

impl Error for ParseVersionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ParseVersionErrorKind {
    /// A version component could not be parsed
    ParseComponent(ParseIntError),
    /// The version string is not in the format expected by `StandardVersion`
    IncorrectFormat,
    /// The version string includes more than 3 components
    TooManyComponents(usize),
}

impl Display for ParseVersionErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::IncorrectFormat => write!(f, "version string is incorrectly formatted"),
            Self::ParseComponent(_) => write!(f, "version component is not a valid integer"),
            Self::TooManyComponents(num_components) => write!(
                f,
                "expected up to three version components, received {}",
                num_components
            ),
        }
    }
}

impl Error for ParseVersionErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            Self::ParseComponent(e) => Some(e),
            Self::IncorrectFormat => None,
            Self::TooManyComponents(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{StandardVersion, ParseVersionError, ParseVersionErrorKind};
    use super::super::version_component::VersionComponent;

    #[test]
    fn parse_single_number() {
        assert_eq!(
            StandardVersion::from_str("3"),
            Ok(StandardVersion {
                major: VersionComponent {
                    value: 3,
                    length: 1
                },
                minor: None,
                patch: None
            })
        )
    }

    #[test]
    fn parse_two_components() {
        assert_eq!(
            StandardVersion::from_str("3.14"),
            Ok(StandardVersion {
                major: VersionComponent {
                    value: 3,
                    length: 1
                },
                minor: Some(VersionComponent {
                    value: 14,
                    length: 2
                }),
                patch: None
            })
        )
    }

    #[test]
    fn parse_three_components() {
        assert_eq!(
            StandardVersion::from_str("3.1.4"),
            Ok(StandardVersion {
                major: VersionComponent {
                    value: 3,
                    length: 1
                },
                minor: Some(VersionComponent {
                    value: 1,
                    length: 1
                }),
                patch: Some(VersionComponent {
                    value: 4,
                    length: 1
                })
            })
        )
    }

    #[test]
    fn parse_zero_prefixed() {
        assert_eq!(
            StandardVersion::from_str("01.28.04"),
            Ok(StandardVersion {
                major: VersionComponent {
                    value: 1,
                    length: 2
                },
                minor: Some(VersionComponent {
                    value: 28,
                    length: 2
                }),
                patch: Some(VersionComponent {
                    value: 4,
                    length: 2
                })
            })
        )
    }

    #[test]
    fn parse_invalid_string() {
        assert_eq!(
            StandardVersion::from_str(".1.0"),
            Err(ParseVersionError {
                version_string: String::from(".1.0"),
                kind: ParseVersionErrorKind::IncorrectFormat
            })
        )
    }

    #[test]
    fn parse_invalid_component() {
        assert_eq!(
            StandardVersion::from_str("0.1.O"),
            Err(ParseVersionError {
                version_string: String::from("0.1.O"),
                kind: ParseVersionErrorKind::ParseComponent(
                    "O".parse::<u32>().unwrap_err()
                )
            })
        )
    }

    #[test]
    fn parse_four_components() {
        assert_eq!(
            StandardVersion::from_str("1.2.4.0"),
            Err(ParseVersionError {
                version_string: String::from("1.2.4.0"),
                kind: ParseVersionErrorKind::TooManyComponents(4)
            })
        )
    }
}
