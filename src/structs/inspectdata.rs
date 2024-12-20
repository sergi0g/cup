use bollard::secret::{ImageInspect, ImageSummary};

pub trait InspectData {
    fn tags(&self) -> Option<&Vec<String>>;
    fn digests(&self) -> Option<&Vec<String>>;
}

impl InspectData for ImageInspect {
    fn tags(&self) -> Option<&Vec<String>> {
        self.repo_tags.as_ref()
    }

    fn digests(&self) -> Option<&Vec<String>> {
        self.repo_digests.as_ref()
    }
}

impl InspectData for ImageSummary {
    fn tags(&self) -> Option<&Vec<String>> {
        Some(&self.repo_tags)
    }

    fn digests(&self) -> Option<&Vec<String>> {
        Some(&self.repo_digests)
    }
}
