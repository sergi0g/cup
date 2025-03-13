use bollard::secret::{ImageInspect, ImageSummary};

pub trait InspectData {
    fn tags(&self) -> Option<Vec<String>>;
    fn digests(&self) -> Option<Vec<String>>;
    fn url(&self) -> Option<String>;
}

impl InspectData for ImageInspect {
    fn tags(&self) -> Option<Vec<String>> {
        self.repo_tags.clone()
    }

    fn digests(&self) -> Option<Vec<String>> {
        self.repo_digests.clone()
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
    fn tags(&self) -> Option<Vec<String>> {
        Some(self.repo_tags.clone())
    }

    fn digests(&self) -> Option<Vec<String>> {
        Some(self.repo_digests.clone())
    }

    fn url(&self) -> Option<String> {
        self.labels.get("org.opencontainers.image.url").cloned()
    }
}

impl InspectData for &String {
    fn tags(&self) -> Option<Vec<String>> {
        self.split('@').next().map(|tag| vec![tag.to_string()])
    }

    fn digests(&self) -> Option<Vec<String>> {
        match self.split_once('@') {
            Some((reference, digest)) => Some(vec![format!(
                "{}@{}",
                reference.split(':').next().unwrap(),
                digest
            )]),
            None => Some(vec![]),
        }
    }

    fn url(&self) -> Option<String> {
        None
    }
}
