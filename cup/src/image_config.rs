use crate::version::{VersionType, standard::StandardUpdateType};

pub struct ImageConfig {
    version_type: VersionType,
    // Will only be read if update type is standard
    ignored_update_types: Vec<StandardUpdateType>,
    /// Whether to skip checking the image (default is false)
    ignore: bool
}

impl ImageConfig {
    
}