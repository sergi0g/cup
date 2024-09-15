use bollard::secret::ImageSummary;

use crate::{
    debug,
    utils::{split_image, CliConfig},
};

#[derive(Clone, Debug)]
pub struct Image {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: Option<String>,
}

impl Image {
    pub async fn from(image: ImageSummary, options: &CliConfig) -> Option<Self> {
        if !image.repo_tags.is_empty() && !image.repo_digests.is_empty() {
            let (registry, repository, tag) = split_image(&image.repo_tags[0]);
            let image = Image {
                registry,
                repository,
                tag,
                digest: Some(
                    image.repo_digests[0]
                        .clone()
                        .split('@')
                        .collect::<Vec<&str>>()[1]
                        .to_string(),
                ),
            };
            return Some(image);
        } else if options.verbose {
            debug!(
                "Skipped an image\nTags: {:#?}\nDigests: {:#?}",
                image.repo_tags, image.repo_digests
            )
        }
        None
    }
}
