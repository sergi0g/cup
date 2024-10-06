use bollard::secret::{ImageInspect, ImageSummary};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{debug, error, utils::CliConfig};

/// Image struct that contains all information that may be needed by a function.
/// It's designed to be passed around between functions
#[derive(Clone, Debug)]
pub struct Image {
    pub reference: String,
    pub registry: Option<String>,
    pub repository: Option<String>,
    pub tag: Option<String>,
    pub local_digests: Option<Vec<String>>,
    pub remote_digest: Option<String>,
}

impl Image {
    /// Creates an populates the fields of an Image object based on the ImageSummary from the Docker daemon
    pub async fn from_summary(image: ImageSummary, options: &CliConfig) -> Option<Self> {
        if !image.repo_tags.is_empty() && !image.repo_digests.is_empty() {
            let mut image = Image {
                reference: image.repo_tags[0].clone(),
                registry: None,
                repository: None,
                tag: None,
                local_digests: Some(
                    image
                        .repo_digests
                        .clone()
                        .iter()
                        .map(|digest| digest.split('@').collect::<Vec<&str>>()[1].to_string())
                        .collect(),
                ),
                remote_digest: None,
            };
            let (registry, repository, tag) = image.split();
            image.registry = Some(registry);
            image.repository = Some(repository);
            image.tag = Some(tag);

            return Some(image);
        } else if options.verbose {
            debug!(
                "Skipped an image\nTags: {:#?}\nDigests: {:#?}",
                image.repo_tags, image.repo_digests
            )
        }
        None
    }

    pub async fn from_inspect(image: ImageInspect) -> Option<Self> {
        if image.repo_tags.is_some()
            && !image.repo_tags.as_ref().unwrap().is_empty()
            && image.repo_digests.is_some()
            && !image.repo_digests.as_ref().unwrap().is_empty()
        {
            let mut image = Image {
                reference: image.repo_tags.as_ref().unwrap()[0].clone(),
                registry: None,
                repository: None,
                tag: None,
                local_digests: Some(
                    image
                        .repo_digests
                        .unwrap()
                        .clone()
                        .iter()
                        .map(|digest| digest.split('@').collect::<Vec<&str>>()[1].to_string())
                        .collect(),
                ),
                remote_digest: None,
            };
            let (registry, repository, tag) = image.split();
            image.registry = Some(registry);
            image.repository = Some(repository);
            image.tag = Some(tag);

            return Some(image);
        }
        None
    }

    /// Takes an image and splits it into registry, repository and tag, based on the reference.
    /// For example, `ghcr.io/sergi0g/cup:latest` becomes `['ghcr.io', 'sergi0g/cup', 'latest']`.
    pub fn split(&self) -> (String, String, String) {
        match RE.captures(&self.reference) {
            Some(c) => {
                let registry = match c.name("registry") {
                    Some(registry) => registry.as_str().to_owned(),
                    None => String::from("registry-1.docker.io"),
                };
                return (
                    registry.clone(),
                    match c.name("repository") {
                        Some(repository) => {
                            let repo = repository.as_str().to_owned();
                            if !repo.contains('/') && registry == "registry-1.docker.io" {
                                format!("library/{}", repo)
                            } else {
                                repo
                            }
                        }
                        None => error!("Failed to parse image {}", &self.reference),
                    },
                    match c.name("tag") {
                        Some(tag) => tag.as_str().to_owned(),
                        None => String::from("latest"),
                    },
                );
            }
            None => error!("Failed to parse image {}", &self.reference),
        }
    }
}

/// Regex to match Docker image references against, so registry, repository and tag can be extracted.
static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(?P<name>(?:(?P<registry>(?:(?:localhost|[\w-]+(?:\.[\w-]+)+)(?::\d+)?)|[\w]+:\d+)/)?(?P<repository>[a-z0-9_.-]+(?:/[a-z0-9_.-]+)*))(?::(?P<tag>[\w][\w.-]{0,127}))?$"#, // From https://regex101.com/r/nmSDPA/1
    )
    .unwrap()
});
