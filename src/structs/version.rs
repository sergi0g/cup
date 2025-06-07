use crate::{config::TagType, structs::standard_version::StandardVersion};

#[derive(Clone, Default, PartialEq, Debug)]
#[non_exhaustive]
pub enum Version {
    #[default]
    Unknown,
    Semver(StandardVersion),
}

impl Version {
    pub fn from_standard(tag: &str) -> Result<Self, ()> {
        match StandardVersion::from_tag(tag) {
            Ok(version) => Ok(Version::Semver(version)),
            Err(e) => Err(e),
        }
    }

    pub fn format_string(&self) -> Option<String> {
        match self {
            Self::Semver(v) => Some(v.format_str.clone()),
            Self::Unknown => None,
        }
    }

    pub fn from(tag: &str, tag_type: Option<&TagType>) -> Self {
        match tag_type {
            Some(t) => match t {
                TagType::Standard => Self::from_standard(tag).unwrap_or(Self::Unknown),
                TagType::Extended => unimplemented!(),
            },
            None => match Self::from_standard(tag) {
                Ok(v) => v,
                Err(_) => Self::Unknown, // match self.from_...
            },
        }
    }
    
    pub fn r#type(&self) -> Option<TagType> {
        match self {
            Self::Semver(_) => Some(TagType::Standard),
            Self::Unknown => None
        }
    }
    
    pub fn as_standard(&self) -> Option<&StandardVersion> {
        match self {
            Self::Semver(s) => Some(s),
            _ => None
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Unknown, Self::Unknown)
            | (Self::Unknown, Self::Semver(_))
            | (Self::Semver(_), Self::Unknown) => None, // Could also just implement the other arms first and leave this as _, but better be explicit rather than implicit
            (Self::Semver(a), Self::Semver(b)) => a.partial_cmp(b),
        }
    }
}
