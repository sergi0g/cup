use serde::{ser::SerializeStruct, Deserialize, Serialize};

use super::{parts::Parts, status::Status};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Default))]
pub struct Update {
    pub reference: String,
    pub parts: Parts,
    pub result: UpdateResult,
    pub time: u32,
    pub server: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Default))]
pub struct UpdateResult {
    pub has_update: Option<bool>,
    pub info: UpdateInfo,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Default))]
#[serde(untagged)]
pub enum UpdateInfo {
    #[cfg_attr(test, default)]
    None,
    Version(VersionUpdateInfo),
    Digest(DigestUpdateInfo),
}

#[derive(Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct VersionUpdateInfo {
    pub version_update_type: String,
    pub new_tag: String,
    pub current_version: String,
    pub new_version: String,
}

#[derive(Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DigestUpdateInfo {
    pub local_digests: Vec<String>,
    pub remote_digest: Option<String>,
}

impl Serialize for VersionUpdateInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("VersionUpdateInfo", 5)?;
        let _ = state.serialize_field("type", "version");
        let _ = state.serialize_field("version_update_type", &self.version_update_type);
        let _ = state.serialize_field("new_tag", &self.new_tag);
        let _ = state.serialize_field("current_version", &self.current_version);
        let _ = state.serialize_field("new_version", &self.new_version);
        state.end()
    }
}

impl Serialize for DigestUpdateInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("DigestUpdateInfo", 3)?;
        let _ = state.serialize_field("type", "digest");
        let _ = state.serialize_field("local_digests", &self.local_digests);
        let _ = state.serialize_field("remote_digest", &self.remote_digest);
        state.end()
    }
}

impl Update {
    pub fn get_status(&self) -> Status {
        match &self.status {
            Status::Unknown(s) => {
                if s.is_empty() {
                    match self.result.has_update {
                        Some(true) => match &self.result.info {
                            UpdateInfo::Version(info) => match info.version_update_type.as_str() {
                                "major" => Status::UpdateMajor,
                                "minor" => Status::UpdateMinor,
                                "patch" => Status::UpdatePatch,
                                _ => unreachable!(),
                            },
                            UpdateInfo::Digest(_) => Status::UpdateAvailable,
                            _ => unreachable!(),
                        },
                        Some(false) => Status::UpToDate,
                        None => Status::Unknown(self.result.error.clone().unwrap()),
                    }
                } else {
                    self.status.clone()
                }
            }
            status => status.clone(),
        }
    }
}
