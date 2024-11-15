use bollard::secret::{ImageInspect, ImageSummary};

pub trait InspectData {
    fn tags(&self) -> Option<Vec<String>>;
    fn digests(&self) -> Option<Vec<String>>;
}

impl InspectData for ImageInspect {
    fn tags(&self) -> Option<Vec<String>> {
        self.repo_tags.clone()
    }

    fn digests(&self) -> Option<Vec<String>> {
        self.repo_digests.clone()
    }
}

impl InspectData for ImageSummary {
    fn tags(&self) -> Option<Vec<String>> {
        Some(self.repo_tags.clone())
    }

    fn digests(&self) -> Option<Vec<String>> {
        Some(self.repo_digests.clone())
    }
}
