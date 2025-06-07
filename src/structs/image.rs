use crate::{
    error,
    http::Client,
    registry::{get_latest_digest, get_latest_tag},
    structs::{standard_version::StandardVersionPart, status::Status, update::Update, version::Version},
    utils::reference::split,
    Context,
};

use super::{
    inspectdata::InspectData,
    parts::Parts,
    update::{DigestUpdateInfo, UpdateResult, VersionUpdateInfo},
};

/// Any local information about the image
#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Info {
    pub local_digests: Vec<String>,
    pub version: Version,
    pub url: Option<String>,
    pub used_by: Vec<String>,
}

/// Any new information obtained about the image
#[derive(Debug, Clone, Default)]
pub struct UpdateInfo {
    pub remote_digest: Option<String>,
    pub latest_version: Option<Version>
}

/// Image struct that contains all information that may be needed by a function working with an image.
/// It's designed to be passed around between functions
#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Image {
    pub reference: String,
    pub parts: Parts,
    pub info: Info,
    pub update_info: UpdateInfo,
    pub error: Option<String>,
    pub time_ms: u32,
}

impl Image {
    /// Creates and populates the fields of an Image object based on the ImageSummary from the Docker daemon
    pub fn from_inspect_data<T: InspectData>(ctx: &Context, image: T) -> Option<Self> {
        let tags = image.tags().unwrap();
        let digests = image.digests().unwrap();
        if !tags.is_empty() && !digests.is_empty() {
            let reference = tags[0].clone();
            if reference.contains('@') {
                return None; // As far as I know, references that contain @ are either manually pulled by the user or automatically created because of swarm. In the first case AFAICT we can't know what tag was originally pulled, so we'd have to make assumptions and I've decided to remove this. The other case is already handled seperately, so this also ensures images aren't displayed twice, once with and once without a digest.
            };
            let (registry, repository, tag) = split(&reference);
            let version_tag = Version::from(&tag, ctx.config.images.get(&reference).map(|cfg| &cfg.tag_type));
            let local_digests = digests
                .iter()
                .filter_map(
                    |digest| match digest.split('@').collect::<Vec<&str>>().get(1) {
                        Some(digest) => Some(digest.to_string()),
                        None => {
                            ctx.logger.warn(format!(
                                "Ignoring invalid digest {} for image {}!",
                                digest, reference
                            ));
                            None
                        }
                    },
                )
                .collect();
            Some(Self {
                reference,
                parts: Parts {
                    registry,
                    repository,
                    tag,
                },
                info: Info { local_digests, version: version_tag, url: image.url(), used_by: Vec::new() },
                ..Default::default()
            })
        } else {
            None
        }
    }

    /// Creates and populates the fields of an Image object based on a reference. If the tag is not recognized as a version string, exits the program with an error.
    pub fn from_reference(reference: &str, ctx: &Context) -> Self {
        let (registry, repository, tag) = split(reference);
        let version_tag = Version::from(&tag, ctx.config.images.get(reference).map(|cfg| &cfg.tag_type));
        match version_tag {
            Version::Unknown => error!(
                "Image {} is not available locally and does not have a recognizable tag format!",
                reference
            ),
            v => Self {
                reference: reference.to_string(),
                parts: Parts {
                    registry,
                    repository,
                    tag,
                },
                info: Info { local_digests: Vec::new(), version: v, url: None, used_by: Vec::new() },
                ..Default::default()
            },
        }
    }

    pub fn has_update(&self) -> Status {
        if self.error.is_some() {
            Status::Unknown(self.error.clone().unwrap())
        } else {
            match self.update_info.latest_version {
                Some(latest_version) => latest_version.to_status(self.info.version),
                None => match self.update_info.remote_digest {
                    Some(remote_digest) => {
                        if self.info.local_digests.contains(&remote_digest) {
                            Status::UpToDate
                        } else {
                            Status::UpdateAvailable
                        }
                    },
                    None => unreachable!() // I hope?
                }
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
            url: self.info.url.clone(),
            result: UpdateResult {
                has_update: has_update.to_option_bool(),
                info: match has_update {
                    Status::Unknown(_) => crate::structs::update::UpdateInfo::None,
                    _ => match update_type {
                        "version" => {
                            let update_info = &self.update_info.latest_version.unwrap().as_standard().unwrap();

                            UpdateInfo::Version(VersionUpdateInfo {
                                version_update_type: match has_update {
                                    Status::UpdateMajor => "major",
                                    Status::UpdateMinor => "minor",
                                    Status::UpdatePatch => "patch",
                                    _ => unreachable!(),
                                }
                                .to_string(),
                                new_tag: update_info.format_str
                                    .replacen("{}", &update_info.major.to_string(), 1)
                                    .replacen("{}", &update_info.minor.unwrap_or_default().to_string(), 1)
                                    .replacen("{}", &update_info.patch.unwrap_or_default().to_string(), 1),
                                // Throwing these in, because they're useful for the CLI output, however we won't (de)serialize them
                                current_version: self.info.version.as_standard().unwrap().to_string()
                                    .version_info
                                    .as_ref()
                                    .unwrap()
                                    .current_tag
                                    .to_string(),
                                new_version: self
                                    .version_info
                                    .as_ref()
                                    .unwrap()
                                    .latest_remote_tag
                                    .as_ref()
                                    .unwrap()
                                    .to_string(),
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
                        _ => unreachable!(),
                    },
                },
                error: self.error.clone(),
            },
            time: self.time_ms,
            server: None,
            used_by: self.used_by.clone(),
            status: has_update,
        }
    }

    /// Checks if the image has an update
    pub async fn check(&self, token: Option<&str>, ctx: &Context, client: &Client) -> Self {
        match &self.version_info {
            Some(data) => get_latest_tag(self, &data.current_tag, token, ctx, client).await,
            None => match self.digest_info {
                Some(_) => get_latest_digest(self, token, ctx, client).await,
                None => unreachable!(),
            },
        }
    }
}
