use bollard::secret::ImageSummary;

use crate::utils::split_image;

#[derive(Clone, Debug)]
pub struct Image {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: Option<String>,
}

impl Image {
    pub async fn from(image: ImageSummary) -> Option<Self> {
        if !image.repo_tags.is_empty() && !image.repo_digests.is_empty() {
            for t in &image.repo_tags {
                let (registry, repository, tag) = split_image(t);
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
                return Some(image)
            }
        }
        None
    }
}