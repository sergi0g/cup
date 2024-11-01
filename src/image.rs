use std::{cmp::Ordering, fmt::Display};

use bollard::models::{ImageInspect, ImageSummary};
use json::{object, JsonValue};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest_middleware::ClientWithMiddleware;

use crate::{
    config::Config,
    error,
    registry::{get_latest_digest, get_latest_tag},
};

/// Image struct that contains all information that may be needed by a function.
/// It's designed to be passed around between functions
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Image {
    pub reference: String,
    pub registry: Option<String>,
    pub repository: Option<String>,
    pub tag: Option<String>,
    pub local_digests: Option<Vec<String>>,
    pub remote_digest: Option<String>,
    pub semver_tag: Option<SemVer>,
    pub latest_remote_tag: Option<SemVer>,
    pub error: Option<String>,
    pub time_ms: i64,
}

impl Image {
    /// Creates an populates the fields of an Image object based on the ImageSummary from the Docker daemon
    pub async fn from_summary(image: ImageSummary) -> Option<Self> {
        if !image.repo_tags.is_empty() && !image.repo_digests.is_empty() {
            let mut image = Image {
                reference: image.repo_tags[0].clone(),
                local_digests: Some(
                    image
                        .repo_digests
                        .clone()
                        .iter()
                        .map(|digest| digest.split('@').collect::<Vec<&str>>()[1].to_string())
                        .collect(),
                ),
                ..Default::default()
            };
            let (registry, repository, tag) = image.split();
            image.registry = Some(registry);
            image.repository = Some(repository);
            image.tag = Some(tag);
            image.semver_tag = image.get_version();

            return Some(image);
        }
        None
    }

    pub async fn from_inspect(image: ImageInspect) -> Option<Self> {
        if image.repo_tags.is_some()
            && !image.repo_tags.as_ref().unwrap().is_empty()
            && image.repo_digests.is_some()
            && !image.repo_digests.as_ref().unwrap().is_empty()
        {
            let mut image = Image {
                reference: image.repo_tags.as_ref().unwrap()[0].clone(),
                local_digests: Some(
                    image
                        .repo_digests
                        .unwrap()
                        .clone()
                        .iter()
                        .map(|digest| digest.split('@').collect::<Vec<&str>>()[1].to_string())
                        .collect(),
                ),
                ..Default::default()
            };
            let (registry, repository, tag) = image.split();
            image.registry = Some(registry);
            image.repository = Some(repository);
            image.tag = Some(tag);
            image.semver_tag = image.get_version();

            return Some(image);
        }
        None
    }

    /// Takes an image and splits it into registry, repository and tag, based on the reference.
    /// For example, `ghcr.io/sergi0g/cup:latest` becomes `['ghcr.io', 'sergi0g/cup', 'latest']`.
    pub fn split(&self) -> (String, String, String) {
        match RE.captures(&self.reference) {
            Some(c) => {
                let registry = match c.name("registry") {
                    Some(registry) => registry.as_str().to_owned(),
                    None => String::from("registry-1.docker.io"),
                };
                return (
                    registry.clone(),
                    match c.name("repository") {
                        Some(repository) => {
                            let repo = repository.as_str().to_owned();
                            if !repo.contains('/') && registry == "registry-1.docker.io" {
                                format!("library/{}", repo)
                            } else {
                                repo
                            }
                        }
                        None => error!("Failed to parse image {}", &self.reference),
                    },
                    match c.name("tag") {
                        Some(tag) => tag.as_str().to_owned(),
                        None => String::from("latest"),
                    },
                );
            }
            None => error!("Failed to parse image {}", &self.reference),
        }
    }

    /// Compares remote digest of the image with its local digests to determine if it has an update or not
    pub fn has_update(&self) -> Status {
        if self.error.is_some() {
            Status::Unknown(self.error.clone().unwrap())
        } else if self.latest_remote_tag.is_some() {
            self.latest_remote_tag
                .as_ref()
                .unwrap()
                .to_status(self.semver_tag.as_ref().unwrap())
        } else if self
            .local_digests
            .as_ref()
            .unwrap()
            .contains(self.remote_digest.as_ref().unwrap())
        {
            Status::UpToDate
        } else {
            Status::UpdateAvailable
        }
    }

    /// Converts image data into a `JsonValue`
    pub fn to_json(&self) -> JsonValue {
        let has_update = self.has_update();
        object! {
            reference: self.reference.clone(),
            parts: object! {
                registry: self.registry.clone(),
                repository: self.repository.clone(),
                tag: self.tag.clone()
            },
            local_digests: self.local_digests.clone(),
            remote_digest: self.remote_digest.clone(),
            result: object! { // API here will have to change for semver
                has_update: has_update.to_option_bool(),
                error: match has_update {
                    Status::Unknown(e) => Some(e),
                    _ => None
                }
            },
            time: self.time_ms
        }
    }

    /// Tries to parse the tag into semver parts
    pub fn get_version(&self) -> Option<SemVer> {
        get_version(self.tag.as_ref().unwrap())
    }

    /// Checks if the image has an update
    pub async fn check(
        &self,
        token: Option<&String>,
        config: &Config,
        client: &ClientWithMiddleware,
    ) -> Self {
        match &self.semver_tag {
            Some(version) => get_latest_tag(self, version, token, config, client).await,
            None => get_latest_digest(self, token, config, client).await,
        }
    }
}

/// Tries to parse the tag into semver parts. Should have been included in impl Image, but that would make the tests more complicated
pub fn get_version(tag: &str) -> Option<SemVer> {
    let captures = SEMVER.captures_iter(tag);
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
            let major: i32 = match c.name("major") {
                Some(major) => major.as_str().parse().unwrap(),
                None => return None,
            };
            let minor: Option<i32> = c.name("minor").map(|minor| minor.as_str().parse().unwrap());
            let patch: Option<i32> = c.name("patch").map(|patch| patch.as_str().parse().unwrap());
            Some(SemVer {
                major,
                minor,
                patch,
            })
        }
        None => None,
    }
}

/// Regex to match Docker image references against, so registry, repository and tag can be extracted.
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(?P<name>(?:(?P<registry>(?:(?:localhost|[\w-]+(?:\.[\w-]+)+)(?::\d+)?)|[\w]+:\d+)/)?(?P<repository>[a-z0-9_.-]+(?:/[a-z0-9_.-]+)*))(?::(?P<tag>[\w][\w.-]{0,127}))?$"#, // From https://regex101.com/r/nmSDPA/1
    )
    .unwrap()
});

/// Heavily modified version of the official semver regex based on common tagging schemes for container images. Sometimes it matches more than once, but we'll try to select the best match. Yes, there _will_ be errors.
static SEMVER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?P<major>0|[1-9]\d*)(?:\.(?P<minor>0|[1-9]\d*))?(?:\.(?P<patch>0|[1-9]\d*)+)?"#)
        .unwrap()
});

/// Enum for image status
#[derive(Ord, Eq, PartialEq, PartialOrd)]
pub enum Status {
    UpdateMajor,
    UpdateMinor,
    UpdatePatch,
    UpdateAvailable,
    UpToDate,
    Unknown(String),
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            Self::UpToDate => "Up to date",
            Self::UpdateAvailable => "Update available",
            Self::UpdateMajor => "Major update",
            Self::UpdateMinor => "Minor update",
            Self::UpdatePatch => "Patch update",
            Self::Unknown(_) => "Unknown",
        })
    }
}

impl Status {
    // Converts the Status into an Option<bool> (useful for JSON serialization)
    pub fn to_option_bool(&self) -> Option<bool> {
        match &self {
            Self::UpToDate => Some(false),
            Self::Unknown(_) => None,
            _ => Some(true),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SemVer {
    pub major: i32,
    pub minor: Option<i32>,
    pub patch: Option<i32>,
}

impl SemVer {
    fn to_status(&self, base: &Self) -> Status {
        if self.major == base.major {
            match (self.minor, base.minor) {
                (Some(a_minor), Some(b_minor)) => {
                    if a_minor == b_minor {
                        match (self.patch, base.patch) {
                            (Some(a_patch), Some(b_patch)) => {
                                if a_patch == b_patch {
                                    unreachable!()
                                } else {
                                    Status::UpdatePatch
                                }
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        Status::UpdateMinor
                    }
                }
                _ => unreachable!(),
            }
        } else {
            Status::UpdateMajor
        }
    }
}

impl Ord for SemVer {
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

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn semver() {
        assert_eq!(get_version("5.3.2"                   ), Some(SemVer { major: 5,  minor: Some(3),   patch: Some(2)  }));
        assert_eq!(get_version("14"                      ), Some(SemVer { major: 14, minor: Some(0),   patch: Some(0)  }));
        assert_eq!(get_version("v0.107.53"               ), Some(SemVer { major: 0,  minor: Some(107), patch: Some(53) }));
        assert_eq!(get_version("12-alpine"               ), Some(SemVer { major: 12, minor: Some(0),   patch: Some(0)  }));
        assert_eq!(get_version("0.9.5-nginx"             ), Some(SemVer { major: 0,  minor: Some(9),   patch: Some(5)  }));
        assert_eq!(get_version("v27.0"                   ), Some(SemVer { major: 27, minor: Some(0),   patch: Some(0)  }));
        assert_eq!(get_version("16.1"                    ), Some(SemVer { major: 16, minor: Some(1),   patch: Some(0)  }));
        assert_eq!(get_version("version-1.5.6"           ), Some(SemVer { major: 1,  minor: Some(5),   patch: Some(6)  }));
        assert_eq!(get_version("15.4-alpine"             ), Some(SemVer { major: 15, minor: Some(4),   patch: Some(0)  }));
        assert_eq!(get_version("pg14-v0.2.0"             ), Some(SemVer { major: 0,  minor: Some(2),   patch: Some(0)  }));
        assert_eq!(get_version("18-jammy-full.s6-v0.88.0"), Some(SemVer { major: 0,  minor: Some(88),  patch: Some(0)  }));
        assert_eq!(get_version("fpm-2.1.0-prod"          ), Some(SemVer { major: 2,  minor: Some(1),   patch: Some(0)  }));
        assert_eq!(get_version("7.3.3.50"                ), Some(SemVer { major: 7,  minor: Some(3),   patch: Some(3)  }));
        assert_eq!(get_version("1.21.11-0"               ), Some(SemVer { major: 1,  minor: Some(21),  patch: Some(11) }));
        assert_eq!(get_version("4.1.2.1-full"            ), Some(SemVer { major: 4,  minor: Some(1),   patch: Some(2)  }));
        assert_eq!(get_version("v4.0.3-ls215"            ), Some(SemVer { major: 4,  minor: Some(0),   patch: Some(3)  }));
    }
}
