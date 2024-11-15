use once_cell::sync::Lazy;
use regex::Regex;

use crate::error;

const DEFAULT_REGISTRY: &str = "registry-1.docker.io";

/// Takes an image and splits it into registry, repository and tag, based on the reference.
/// For example, `ghcr.io/sergi0g/cup:latest` becomes `['ghcr.io', 'sergi0g/cup', 'latest']`.
pub fn split(reference: &str) -> (String, String, String) {
    match REFERENCE_REGEX.captures(reference) {
        Some(c) => {
            let registry = match c.name("registry") {
                Some(registry) => registry.as_str().to_owned(),
                None => String::from(DEFAULT_REGISTRY),
            };
            return (
                registry.clone(),
                match c.name("repository") {
                    Some(repository) => {
                        let repo = repository.as_str().to_owned();
                        if !repo.contains('/') && registry == DEFAULT_REGISTRY {
                            format!("library/{}", repo)
                        } else {
                            repo
                        }
                    }
                    None => error!("Failed to parse image {}", reference),
                },
                match c.name("tag") {
                    Some(tag) => tag.as_str().to_owned(),
                    None => String::from("latest"),
                },
            );
        }
        None => error!("Failed to parse image {}", reference),
    }
}

/// Regex to match Docker image references against, so registry, repository and tag can be extracted.
static REFERENCE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^(?P<name>(?:(?P<registry>(?:(?:localhost|[\w-]+(?:\.[\w-]+)+)(?::\d+)?)|[\w]+:\d+)/)?(?P<repository>[a-z0-9_.-]+(?:/[a-z0-9_.-]+)*))(?::(?P<tag>[\w][\w.-]{0,127}))?$"#, // From https://regex101.com/r/nmSDPA/1
    )
    .unwrap()
});
