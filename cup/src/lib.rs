use rustc_hash::FxHashMap;

use crate::{
    auth::AuthToken, image_config::ImageConfig, image_to_check::ImageToCheck, reference::Reference,
    reference_matcher::ReferenceMatcher, registry::Registry, registry_config::RegistryConfig,
    socket::Socket,
};

pub mod auth;
pub mod image_config;
pub mod image_to_check;
pub mod reference;
pub mod reference_matcher;
pub mod registry;
pub mod registry_config;
pub mod socket;
pub mod version;
pub mod docker;

/// This struct is the main interface to Cup's functions.
/// Any useful user-provided or generated data is stored here for reuse during the lifetime of the struct.
#[derive(Default)]
pub struct Cup {
    images: Vec<ImageToCheck>,
    socket: Socket,
}

impl<'a> Cup {
    /// Returns a new empty instance with all values initialized to their defaults.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> CupBuilder {
        CupBuilder::default()
    }

    /// Check for updates to the images provided (and any images configured through options)
    pub async fn check(images: &[&Reference]) -> Vec<Result<CheckResult, CheckError>> {
        todo!()
    }
}

#[derive(Default)]
pub struct CupBuilder {
    /// Whether or not to check for updates to local images in addition to the other images.
    check_local_images: bool,
    /// Whether or not to check for updates to images used in the swarm in addition to the other images. If the host is not a manager node this will only check the locally running swarm images.
    check_swarm_images: bool,
    registry_config: FxHashMap<Registry, RegistryConfig>,
    overrides: FxHashMap<ReferenceMatcher, ImageConfig>,
    socket: Socket,
}

impl CupBuilder {
    pub fn with_local_images(&mut self) {
        self.check_local_images = true
    }

    pub fn with_swarm_images(&mut self) {
        self.check_swarm_images = true
    }

    pub fn with_registry_auth(&mut self, registry: Registry, token: AuthToken) {
        match self.registry_config.get_mut(&registry) {
            Some(registry_config) => registry_config.auth = Some(token),
            None => {
                self.registry_config.insert(
                    registry,
                    RegistryConfig {
                        auth: Some(token),
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn with_insecure_registry(&mut self, registry: Registry) {
        match self.registry_config.get_mut(&registry) {
            Some(registry_config) => registry_config.insecure = true,
            None => {
                self.registry_config.insert(
                    registry,
                    RegistryConfig {
                        insecure: true,
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn with_image_config(&mut self, reference: ReferenceMatcher, config: ImageConfig) {
        self.overrides.insert(reference, config);
    }

    pub fn with_socket(&mut self, socket: Socket) {
        self.socket = socket
    }

    pub fn build(&self) -> Cup {
        
        
        todo!()
    }
}

pub struct CheckResult {}
pub struct CheckError {}
