use json::{object, JsonValue};

use crate::{
    config::Config,
    error,
    http::Client,
    registry::{get_latest_digest, get_latest_tag},
    structs::{status::Status, version::Version},
    utils::reference::split,
};

use super::inspectdata::InspectData;

#[derive(Clone, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct DigestInfo {
    pub local_digests: Vec<String>,
    pub remote_digest: Option<String>,
}

#[derive(Clone, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct VersionInfo {
    pub current_tag: Version,
    pub latest_remote_tag: Option<Version>,
}

/// Image struct that contains all information that may be needed by a function working with an image.
/// It's designed to be passed around between functions
#[derive(Clone, PartialEq, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Image {
    pub reference: String,
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest_info: Option<DigestInfo>,
    pub version_info: Option<VersionInfo>,
    pub error: Option<String>,
    pub time_ms: i64,
}

impl Image {
    /// Creates and populates the fields of an Image object based on the ImageSummary from the Docker daemon
    pub fn from_inspect_data<T: InspectData>(image: T) -> Option<Self> {
        let tags = image.tags().unwrap();
        let digests = image.digests().unwrap();
        if !tags.is_empty() && !digests.is_empty() {
            let reference = tags[0].clone();
            let (registry, repository, tag) = split(&reference);
            let version_tag = Version::from_tag(&tag);
            let local_digests = digests
                .iter()
                .map(|digest| digest.split('@').collect::<Vec<&str>>()[1].to_string())
                .collect();
            Some(Self {
                reference,
                registry,
                repository,
                tag,
                digest_info: Some(DigestInfo {
                    local_digests,
                    remote_digest: None,
                }),
                version_info: version_tag.map(|vtag| VersionInfo {
                    current_tag: vtag,
                    latest_remote_tag: None,
                }),
                ..Default::default()
            })
        } else {
            None
        }
    }

    /// Creates and populates the fields of an Image object based on a reference. If the tag is not recognized as a version string, exits the program with an error.
    pub fn from_reference(reference: &str) -> Self {
        let (registry, repository, tag) = split(reference);
        let version_tag = Version::from_tag(&tag);
        match version_tag {
            Some(version) => Self {
                reference: reference.to_string(),
                registry,
                repository,
                tag,
                version_info: Some(VersionInfo {
                    current_tag: version,
                    latest_remote_tag: None,
                }),
                ..Default::default()
            },
            None => error!(
                "Image {} is not available locally and does not have a recognizable tag format!",
                reference
            ),
        }
    }

    /// Compares remote digest of the image with its local digests to determine if it has an update or not
    pub fn has_update(&self) -> Status {
        if self.error.is_some() {
            Status::Unknown(self.error.clone().unwrap())
        } else {
            match &self.version_info {
                Some(data) => data
                    .latest_remote_tag
                    .as_ref()
                    .unwrap()
                    .to_status(&data.current_tag),
                None => match &self.digest_info {
                    Some(data) => {
                        if data
                            .local_digests
                            .contains(data.remote_digest.as_ref().unwrap())
                        {
                            Status::UpToDate
                        } else {
                            Status::UpdateAvailable
                        }
                    }
                    None => unreachable!(), // I hope?
                },
            }
        }
    }

    /// Converts image data into a `JsonValue`
    pub fn to_json(&self) -> JsonValue {
        let has_update = self.has_update();
        let update_type = match has_update {
            Status::UpdateMajor | Status::UpdateMinor | Status::UpdatePatch => "version",
            _ => "digest",
        };
        object! {
            reference: self.reference.clone(),
            parts: object! {
                registry: self.registry.clone(),
                repository: self.repository.clone(),
                tag: self.tag.clone()
            },
            result: object! {
                has_update: has_update.to_option_bool(),
                info: match has_update {
                    Status::Unknown(_) => None,
                    _ => Some(match update_type {
                        "version" => {
                            let (version_tag, latest_remote_tag) = match &self.version_info {
                                Some(data) => (data.current_tag.clone(), data.latest_remote_tag.clone()),
                                _ => unreachable!()
                            };
                            object! {
                                "type": update_type,
                                version_update_type: match has_update {
                                    Status::UpdateMajor => "major",
                                    Status::UpdateMinor => "minor",
                                    Status::UpdatePatch => "patch",
                                    _ => unreachable!()
                                },
                                new_version: self.tag.replace(&version_tag.to_string(), &latest_remote_tag.as_ref().unwrap().to_string())
                            }
                        },
                        "digest" => {
                            let (local_digests, remote_digest) = match &self.digest_info {
                                Some(data) => (data.local_digests.clone(), data.remote_digest.clone()),
                                _ => unreachable!()
                            };
                            object! {
                                "type": update_type,
                                local_digests: local_digests,
                                remote_digest: remote_digest,
                            }
                        },
                        _ => unreachable!()
                    })
                }},
            time: self.time_ms
        }
    }

    /// Checks if the image has an update
    pub async fn check(&self, token: Option<&String>, config: &Config, client: &Client) -> Self {
        match &self.version_info {
            Some(data) => get_latest_tag(self, &data.current_tag, token, config, client).await,
            None => match self.digest_info {
                Some(_) => get_latest_digest(self, token, config, client).await,
                None => unreachable!(),
            },
        }
    }
}
