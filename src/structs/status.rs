use std::fmt::Display;

/// Enum for image status
#[derive(Ord, Eq, PartialEq, PartialOrd, Clone)]
#[cfg_attr(test, derive(Debug))]
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

impl Default for Status {
    fn default() -> Self {
        Self::Unknown("".to_string())
    }
}