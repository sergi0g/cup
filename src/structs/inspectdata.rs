use bollard::secret::{ImageInspect, ImageSummary};

pub trait InspectData {
    fn tags(&self) -> Option<&Vec<String>>;
    fn digests(&self) -> Option<&Vec<String>>;
    fn url(&self) -> Option<String>;
}

impl InspectData for ImageInspect {
    fn tags(&self) -> Option<&Vec<String>> {
        self.repo_tags.as_ref()
    }

    fn digests(&self) -> Option<&Vec<String>> {
        self.repo_digests.as_ref()
    }

    fn url(&self) -> Option<String> {
        match &self.config {
            Some(config) => match &config.labels {
                Some(labels) => labels.get("org.opencontainers.image.url").cloned(),
                None => None,
            },
            None => None,
        }
    }
}

impl InspectData for ImageSummary {
    fn tags(&self) -> Option<&Vec<String>> {
        Some(&self.repo_tags)
    }

    fn digests(&self) -> Option<&Vec<String>> {
        Some(&self.repo_digests)
    }

    fn url(&self) -> Option<String> {
        self.labels.get("org.opencontainers.image.url").cloned()
    }
}
