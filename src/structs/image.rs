use crate::{
    config::Config,
    error,
    http::Client,
    registry::{get_latest_digest, get_latest_tag},
    structs::{status::Status, version::Version},
    utils::reference::split,
};

use super::{
    inspectdata::InspectData,
    parts::Parts,
    update::{DigestUpdateInfo, Update, UpdateInfo, UpdateResult, VersionUpdateInfo},
};

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
    pub format_str: String,
}

/// Image struct that contains all information that may be needed by a function working with an image.
/// It's designed to be passed around between functions
#[derive(Clone, PartialEq, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Image {
    pub reference: String,
    pub parts: Parts,
    pub digest_info: Option<DigestInfo>,
    pub version_info: Option<VersionInfo>,
    pub error: Option<String>,
    pub time_ms: u32,
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
                parts: Parts {
                    registry,
                    repository,
                    tag,
                },
                digest_info: Some(DigestInfo {
                    local_digests,
                    remote_digest: None,
                }),
                version_info: version_tag.map(|(vtag, format_str)| VersionInfo {
                    current_tag: vtag,
                    format_str,
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
            Some((version, format_str)) => Self {
                reference: reference.to_string(),
                parts: Parts {
                    registry,
                    repository,
                    tag,
                },
                version_info: Some(VersionInfo {
                    current_tag: version,
                    format_str,
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

    /// Converts image data into an `Update`
    pub fn to_update(&self) -> Update {
        let has_update = self.has_update();
        let update_type = match has_update {
            Status::UpToDate => "none",
            Status::UpdateMajor | Status::UpdateMinor | Status::UpdatePatch => "version",
            _ => "digest",
        };
        Update {
            reference: self.reference.clone(),
            parts: self.parts.clone(),
            result: UpdateResult {
                has_update: has_update.to_option_bool(),
                info: match has_update {
                    Status::Unknown(_) => UpdateInfo::None,
                    _ => match update_type {
                        "version" => {
                            let (new_tag, format_str) = match &self.version_info {
                                Some(data) => (
                                    data.latest_remote_tag.clone().unwrap(),
                                    data.format_str.clone(),
                                ),
                                _ => unreachable!(),
                            };

                            UpdateInfo::Version(VersionUpdateInfo {
                                version_update_type: match has_update {
                                    Status::UpdateMajor => "major",
                                    Status::UpdateMinor => "minor",
                                    Status::UpdatePatch => "patch",
                                    _ => unreachable!(),
                                }
                                .to_string(),
                                new_version: format_str
                                    .replacen("{}", &new_tag.major.to_string(), 1)
                                    .replacen("{}", &new_tag.minor.unwrap_or(0).to_string(), 1)
                                    .replacen("{}", &new_tag.patch.unwrap_or(0).to_string(), 1),
                            })
                        }
                        "digest" => {
                            let (local_digests, remote_digest) = match &self.digest_info {
                                Some(data) => {
                                    (data.local_digests.clone(), data.remote_digest.clone())
                                }
                                _ => unreachable!(),
                            };
                            UpdateInfo::Digest(DigestUpdateInfo {
                                local_digests,
                                remote_digest,
                            })
                        }
                        "none" => UpdateInfo::None,
                        _ => unreachable!()
                    },
                },
                error: self.error.clone(),
            },
            time: self.time_ms,
            server: None,
            status: has_update,
        }
    }

    /// Checks if the image has an update
    pub async fn check(&self, token: Option<&str>, config: &Config, client: &Client) -> Self {
        match &self.version_info {
            Some(data) => get_latest_tag(self, &data.current_tag, token, config, client).await,
            None => match self.digest_info {
                Some(_) => get_latest_digest(self, token, config, client).await,
                None => unreachable!(),
            },
        }
    }
}
