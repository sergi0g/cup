use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Parts {
    pub registry: String,
    pub repository: String,
    pub tag: String,
}
