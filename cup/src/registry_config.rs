use crate::auth::AuthToken;

#[derive(Default)]
pub struct RegistryConfig {
    /// An optional authentication token to use when making requests to this registry
    pub(super) auth: Option<AuthToken>,
    /// Whether or not to connect to the registry over unencrypted HTTP
    pub(super) insecure: bool
}