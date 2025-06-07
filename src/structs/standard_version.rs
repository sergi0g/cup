use std::{cmp::Ordering, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Clone, PartialEq, Debug, Default)] // Default is so I can avoid constructing a struct every time I want to use a version number of 0 as a default.
pub struct StandardVersionPart {
    value: u32,
    length: u8, // If the value is prefixed by zeroes, the total length, otherwise 0
}

impl StandardVersionPart {
    fn from_split(split: &str) -> Self {
        if split.len() == 1 && split == "0" {
            Self::default()
        } else {
            Self {
                value: split.parse().expect("Expected number to be less than 2^32"), // Unwrapping is safe, because we've verified that the string consists of digits and we don't care about supporting big numbers.
                length: {
                    if split.starts_with('0') {
                        split.len() as u8 // We're casting the zeroes to u8, because no sane person uses more than 255 zeroes as a version prefix. Oh wait, tags can't even be that long
                    } else {
                        0
                    }
                },
            }
        }
    }
}

impl PartialOrd for StandardVersionPart {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.length == other.length {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
    }
}

impl Display for StandardVersionPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:0<zeroes$}",
            self.value,
            zeroes = self.length as usize
        ))
    }
}

/// Represents a semver-like version.
/// While not conforming to the SemVer standard, but was designed to handle common versioning schemes across a wide range of Docker images.
/// Minor and patch versions are considered optional.
/// Matching happens with a regex.
#[derive(Clone, PartialEq, Debug)]
pub struct StandardVersion {
    pub major: StandardVersionPart,
    pub minor: Option<StandardVersionPart>,
    pub patch: Option<StandardVersionPart>,
    pub format_str: String, // The tag with {} in the place the version was matched.
}

impl StandardVersion {
    /// Tries to extract a semver-like version from a tag.
    /// Returns a Result<StandardVersion, ()> indicating whether parsing succeeded
    pub fn from_tag(tag: &str) -> Result<Self, ()> {
        /// Heavily modified version of the official semver regex based on common tagging schemes for container images. Sometimes it matches more than once, but we'll try to select the best match.
        static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?P<major>[0-9]+)(?:\.(?P<minor>[0-9]*))?(?:\.(?P<patch>[0-9]*))?")
                .unwrap()
        });
        let mut captures = VERSION_REGEX.captures_iter(tag);
        // And now... terrible best match selection for everyone! Actually, it's probably not that terrible. I don't know.
        match captures.next() {
            Some(mut best_match) => {
                let mut max_matches: u8 = 0; // Why does Rust not have `u2`s?
                for capture in captures {
                    let count = capture.iter().filter_map(|c| c).count() as u8;
                    if count > max_matches {
                        max_matches = count;
                        best_match = capture;
                    }
                }

                let start_pos;
                let mut end_pos;
                let major: StandardVersionPart = match best_match.name("major") {
                    Some(major) => {
                        start_pos = major.start();
                        end_pos = major.end();
                        StandardVersionPart::from_split(major.as_str())
                    }
                    None => return Err(()),
                };
                let minor: Option<StandardVersionPart> = best_match.name("minor").map(|minor| {
                    end_pos = minor.end();
                    StandardVersionPart::from_split(minor.as_str())
                });
                let patch: Option<StandardVersionPart> = best_match.name("patch").map(|patch| {
                    end_pos = patch.end();
                    StandardVersionPart::from_split(patch.as_str())
                });
                let mut format_str = tag.to_string();
                format_str.replace_range(start_pos..end_pos, "{}");
                Ok(Self {
                    major,
                    minor,
                    patch,
                    format_str,
                })
            }
            None => Err(()),
        }
    }
}

impl PartialOrd for StandardVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.format_str != other.format_str {
            None
        } else {
            match self.major.partial_cmp(&other.major) {
                Some(ordering) => match ordering {
                    Ordering::Equal => match self.minor.partial_cmp(&other.minor) {
                        Some(ordering) => match ordering {
                            Ordering::Equal => self.patch.partial_cmp(&other.patch),
                            _ => Some(ordering),
                        },
                        None => None,
                    },
                    _ => Some(ordering),
                },
                None => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn standard_version() {
        assert_eq!(StandardVersion::from_tag("5.3.2"), Ok(StandardVersion { major: StandardVersionPart { value: 5, length: 0 }, minor: Some(StandardVersionPart { value: 3, length: 0 }), patch: Some(StandardVersionPart { value: 2, length: 0 }) , format_str: String::from("{}") }));
        assert_eq!(StandardVersion::from_tag("14"), Ok(StandardVersion { major: StandardVersionPart { value: 14, length: 0 }, minor: None, patch: None , format_str: String::from("{}") }));
        assert_eq!(StandardVersion::from_tag("v0.107.53"), Ok(StandardVersion { major: StandardVersionPart { value: 0, length: 0 }, minor: Some(StandardVersionPart { value: 107, length: 0 }), patch: Some(StandardVersionPart { value: 53, length: 0 }) , format_str: String::from("v{}") }));
        assert_eq!(StandardVersion::from_tag("12-alpine"), Ok(StandardVersion { major: StandardVersionPart { value: 12, length: 0 }, minor: None, patch: None , format_str: String::from("{}-alpine") }));
        assert_eq!(StandardVersion::from_tag("0.9.5-nginx"), Ok(StandardVersion { major: StandardVersionPart { value: 0, length: 0 }, minor: Some(StandardVersionPart { value: 9, length: 0 }), patch: Some(StandardVersionPart { value: 5, length: 0 }) , format_str: String::from("{}-nginx") }));
        assert_eq!(StandardVersion::from_tag("v27.0"), Ok(StandardVersion { major: StandardVersionPart { value: 27, length: 0 }, minor: Some(StandardVersionPart { value: 0, length: 0 }), patch: None , format_str: String::from("v{}") }));
        assert_eq!(StandardVersion::from_tag("16.1"), Ok(StandardVersion { major: StandardVersionPart { value: 16, length: 0 }, minor: Some(StandardVersionPart { value: 1, length: 0 }), patch: None , format_str: String::from("{}") }));
        assert_eq!(StandardVersion::from_tag("version-1.5.6"), Ok(StandardVersion { major: StandardVersionPart { value: 1, length: 0 }, minor: Some(StandardVersionPart { value: 5, length: 0 }), patch: Some(StandardVersionPart { value: 6, length: 0 }) , format_str: String::from("version-{}") }));
        assert_eq!(StandardVersion::from_tag("15.4-alpine"), Ok(StandardVersion { major: StandardVersionPart { value: 15, length: 0 }, minor: Some(StandardVersionPart { value: 4, length: 0 }), patch: None , format_str: String::from("{}-alpine") }));
        assert_eq!(StandardVersion::from_tag("pg14-v0.2.0"), Ok(StandardVersion { major: StandardVersionPart { value: 0, length: 0 }, minor: Some(StandardVersionPart { value: 2, length: 0 }), patch: Some(StandardVersionPart { value: 0, length: 0 }) , format_str: String::from("pg14-v{}") }));
        assert_eq!(StandardVersion::from_tag("18-jammy-full.s6-v0.88.0"), Ok(StandardVersion { major: StandardVersionPart { value: 0, length: 0 }, minor: Some(StandardVersionPart { value: 88, length: 0 }), patch: Some(StandardVersionPart { value: 0, length: 0 }) , format_str: String::from("18-jammy-full.s6-v{}") }));
        assert_eq!(StandardVersion::from_tag("fpm-2.1.0-prod"), Ok(StandardVersion { major: StandardVersionPart { value: 2, length: 0 }, minor: Some(StandardVersionPart { value: 1, length: 0 }), patch: Some(StandardVersionPart { value: 0, length: 0 }) , format_str: String::from("fpm-{}-prod") }));
        assert_eq!(StandardVersion::from_tag("7.3.3.50"), Ok(StandardVersion { major: StandardVersionPart { value: 7, length: 0 }, minor: Some(StandardVersionPart { value: 3, length: 0 }), patch: Some(StandardVersionPart { value: 3, length: 0 }) , format_str: String::from("{}.50") }));
        assert_eq!(StandardVersion::from_tag("1.21.11-0"), Ok(StandardVersion { major: StandardVersionPart { value: 1, length: 0 }, minor: Some(StandardVersionPart { value: 21, length: 0 }), patch: Some(StandardVersionPart { value: 11, length: 0 }) , format_str: String::from("{}-0") }));
        assert_eq!(StandardVersion::from_tag("4.1.2.1-full"), Ok(StandardVersion { major: StandardVersionPart { value: 4, length: 0 }, minor: Some(StandardVersionPart { value: 1, length: 0 }), patch: Some(StandardVersionPart { value: 2, length: 0 }) , format_str: String::from("{}.1-full") }));
        assert_eq!(StandardVersion::from_tag("v4.0.3-ls215"), Ok(StandardVersion { major: StandardVersionPart { value: 4, length: 0 }, minor: Some(StandardVersionPart { value: 0, length: 0 }), patch: Some(StandardVersionPart { value: 3, length: 0 }) , format_str: String::from("v{}-ls215") }));
        assert_eq!(StandardVersion::from_tag("24.04.11.2.1"), Ok(StandardVersion { major: StandardVersionPart { value: 24, length: 0 }, minor: Some(StandardVersionPart { value: 4, length: 2 }), patch: Some(StandardVersionPart { value: 11, length: 0 }) , format_str: String::from("{}.2.1") }));
        assert_eq!(StandardVersion::from_tag("example15-test"), Ok(StandardVersion { major: StandardVersionPart { value: 15, length: 0 }, minor: None, patch: None , format_str: String::from("example{}-test") }));
        assert_eq!(StandardVersion::from_tag("watch-the-dot-5.3.2.careful"), Ok(StandardVersion { major: StandardVersionPart { value: 5, length: 0 }, minor: Some(StandardVersionPart { value: 3, length: 0 }), patch: Some(StandardVersionPart { value: 2, length: 0 }), format_str: String::from("watch-the-dot-{}.careful") }));
    }

    #[test]
    fn version_part() {
        assert_eq!(
            format!(
                "{:?}",
                StandardVersionPart {
                    value: 21,
                    length: 4
                }
            ),
            String::from("0021")
        );
    }
}
