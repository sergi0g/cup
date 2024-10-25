use std::fmt::Display;

use bollard::models::{ImageInspect, ImageSummary};
use json::{object, JsonValue};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::error;

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
}

/// Regex to match Docker image references against, so registry, repository and tag can be extracted.
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(?P<name>(?:(?P<registry>(?:(?:localhost|[\w-]+(?:\.[\w-]+)+)(?::\d+)?)|[\w]+:\d+)/)?(?P<repository>[a-z0-9_.-]+(?:/[a-z0-9_.-]+)*))(?::(?P<tag>[\w][\w.-]{0,127}))?$"#, // From https://regex101.com/r/nmSDPA/1
    )
    .unwrap()
});

/// Enum for image status
pub enum Status {
    UpToDate,
    UpdateAvailable,
    Unknown(String),
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            Self::UpToDate => "Up to date",
            Self::UpdateAvailable => "Update available",
            Self::Unknown(_) => "Unknown",
        })
    }
}

impl Status {
    // Converts the Status into an Option<bool> (useful for JSON serialization)
    pub fn to_option_bool(&self) -> Option<bool> {
        match &self {
            Self::UpdateAvailable => Some(true),
            Self::UpToDate => Some(false),
            Self::Unknown(_) => None,
        }
    }
}
