use std::{cmp::Ordering, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

use super::status::Status;

/// Semver-like version struct
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Version {
    pub major: u32,
    pub minor: Option<u32>,
    pub patch: Option<u32>,
}

impl Version {
    /// Tries to parse the tag into semver-like parts. Returns a Version object and a string usable in format! with {} in the positions matches were found
    pub fn from_tag(tag: &str) -> Option<(Self, String)> {
        /// Heavily modified version of the official semver regex based on common tagging schemes for container images. Sometimes it matches more than once, but we'll try to select the best match.
        static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?P<major>0|[1-9][0-9]*)(?:\.(?P<minor>0|[1-9][0-9]*))?(?:\.(?P<patch>0|[1-9][0-9]*))?",
            )
            .unwrap()
        });
        let captures = VERSION_REGEX.captures_iter(tag);
        // And now... terrible best match selection for everyone!
        let mut max_matches = 0;
        let mut best_match = None;
        for capture in captures {
            let mut count = 0;
            for idx in 1..capture.len() {
                if capture.get(idx).is_some() {
                    count += 1
                } else {
                    break;
                }
            }
            if count > max_matches {
                max_matches = count;
                best_match = Some(capture);
            }
        }
        match best_match {
            Some(c) => {
                let mut positions = Vec::new();
                let major: u32 = match c.name("major") {
                    Some(major) => {
                        positions.push((major.start(), major.end()));
                        major.as_str().parse().unwrap()
                    }
                    None => return None,
                };
                let minor: Option<u32> = c.name("minor").map(|minor| {
                    positions.push((minor.start(), minor.end()));
                    minor.as_str().parse().unwrap()
                });
                let patch: Option<u32> = c.name("patch").map(|patch| {
                    positions.push((patch.start(), patch.end()));
                    patch.as_str().parse().unwrap()
                });
                let mut format_str = tag.to_string();
                positions.reverse();
                positions.iter().for_each(|(start, end)| {
                    format_str.replace_range(*start..*end, "{}");
                });
                Some((
                    Version {
                        major,
                        minor,
                        patch,
                    },
                    format_str,
                ))
            }
            None => None,
        }
    }

    pub fn to_status(&self, base: &Self) -> Status {
        if self.major > base.major {
            Status::UpdateMajor
        } else if self.major == base.major {
            match (self.minor, base.minor) {
                (Some(a_minor), Some(b_minor)) => {
                    if a_minor > b_minor {
                        Status::UpdateMinor
                    } else if a_minor == b_minor {
                        match (self.patch, base.patch) {
                            (Some(a_patch), Some(b_patch)) => {
                                if a_patch > b_patch {
                                    Status::UpdatePatch
                                } else if a_patch == b_patch {
                                    Status::UpToDate
                                } else {
                                    Status::Unknown(format!("Tag {} does not exist", base))
                                }
                            }
                            (None, None) => Status::UpToDate,
                            _ => unreachable!(),
                        }
                    } else {
                        Status::Unknown(format!("Tag {} does not exist", base))
                    }
                }
                (None, None) => Status::UpToDate,
                _ => unreachable!(
                    "Version error: {} and {} should either both be Some or None",
                    self, base
                ),
            }
        } else {
            Status::Unknown(format!("Tag {} does not exist", base))
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let major_ordering = self.major.cmp(&other.major);
        match major_ordering {
            Ordering::Equal => match (self.minor, other.minor) {
                (Some(self_minor), Some(other_minor)) => {
                    let minor_ordering = self_minor.cmp(&other_minor);
                    match minor_ordering {
                        Ordering::Equal => match (self.patch, other.patch) {
                            (Some(self_patch), Some(other_patch)) => self_patch.cmp(&other_patch),
                            _ => Ordering::Equal,
                        },
                        _ => minor_ordering,
                    }
                }
                _ => Ordering::Equal,
            },
            _ => major_ordering,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}{}{}",
            self.major,
            match self.minor {
                Some(minor) => format!(".{}", minor),
                None => String::new(),
            },
            match self.patch {
                Some(patch) => format!(".{}", patch),
                None => String::new(),
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn version() {
        assert_eq!(Version::from_tag("5.3.2"                   ), Some((Version { major: 5,  minor: Some(3),   patch: Some(2)  }, String::from("{}.{}.{}"                  ))));
        assert_eq!(Version::from_tag("14"                      ), Some((Version { major: 14, minor: None,      patch: None     }, String::from("{}"                        ))));
        assert_eq!(Version::from_tag("v0.107.53"               ), Some((Version { major: 0,  minor: Some(107), patch: Some(53) }, String::from("v{}.{}.{}"                 ))));
        assert_eq!(Version::from_tag("12-alpine"               ), Some((Version { major: 12, minor: None,      patch: None     }, String::from("{}-alpine"                 ))));
        assert_eq!(Version::from_tag("0.9.5-nginx"             ), Some((Version { major: 0,  minor: Some(9),   patch: Some(5)  }, String::from("{}.{}.{}-nginx"            ))));
        assert_eq!(Version::from_tag("v27.0"                   ), Some((Version { major: 27, minor: Some(0),   patch: None     }, String::from("v{}.{}"                    ))));
        assert_eq!(Version::from_tag("16.1"                    ), Some((Version { major: 16, minor: Some(1),   patch: None     }, String::from("{}.{}"                     ))));
        assert_eq!(Version::from_tag("version-1.5.6"           ), Some((Version { major: 1,  minor: Some(5),   patch: Some(6)  }, String::from("version-{}.{}.{}"          ))));
        assert_eq!(Version::from_tag("15.4-alpine"             ), Some((Version { major: 15, minor: Some(4),   patch: None     }, String::from("{}.{}-alpine"              ))));
        assert_eq!(Version::from_tag("pg14-v0.2.0"             ), Some((Version { major: 0,  minor: Some(2),   patch: Some(0)  }, String::from("pg14-v{}.{}.{}"            ))));
        assert_eq!(Version::from_tag("18-jammy-full.s6-v0.88.0"), Some((Version { major: 0,  minor: Some(88),  patch: Some(0)  }, String::from("18-jammy-full.s6-v{}.{}.{}"))));
        assert_eq!(Version::from_tag("fpm-2.1.0-prod"          ), Some((Version { major: 2,  minor: Some(1),   patch: Some(0)  }, String::from("fpm-{}.{}.{}-prod"         ))));
        assert_eq!(Version::from_tag("7.3.3.50"                ), Some((Version { major: 7,  minor: Some(3),   patch: Some(3)  }, String::from("{}.{}.{}.50"               ))));
        assert_eq!(Version::from_tag("1.21.11-0"               ), Some((Version { major: 1,  minor: Some(21),  patch: Some(11) }, String::from("{}.{}.{}-0"                ))));
        assert_eq!(Version::from_tag("4.1.2.1-full"            ), Some((Version { major: 4,  minor: Some(1),   patch: Some(2)  }, String::from("{}.{}.{}.1-full"           ))));
        assert_eq!(Version::from_tag("v4.0.3-ls215"            ), Some((Version { major: 4,  minor: Some(0),   patch: Some(3)  }, String::from("v{}.{}.{}-ls215"           ))));
    }
}
