#[derive(Clone, Debug)]
pub struct Image {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: Option<String>,
}
