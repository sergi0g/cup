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
            let is_default_registry = registry == DEFAULT_REGISTRY;
            return (
                registry,
                match c.name("repository") {
                    Some(repository) => {
                        let repo = repository.as_str().to_owned();
                        if !repo.contains('/') && is_default_registry {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn reference() {
        assert_eq!(split("alpine"),                                                                                                                  (String::from(DEFAULT_REGISTRY),          String::from("library/alpine"),         String::from("latest")));
        assert_eq!(split("alpine:latest"),                                                                                                           (String::from(DEFAULT_REGISTRY),          String::from("library/alpine"),         String::from("latest")));
        assert_eq!(split("library/alpine"),                                                                                                          (String::from(DEFAULT_REGISTRY),          String::from("library/alpine"),         String::from("latest")));
        assert_eq!(split("localhost/test"),                                                                                                          (String::from("localhost"),               String::from("test"),                   String::from("latest")));
        assert_eq!(split("localhost:1234/test"),                                                                                                     (String::from("localhost:1234"),          String::from("test"),                   String::from("latest")));
        assert_eq!(split("test:1234/idk"),                                                                                                           (String::from("test:1234"),               String::from("idk"),                    String::from("latest")));
        assert_eq!(split("alpine:3.7"),                                                                                                              (String::from(DEFAULT_REGISTRY),          String::from("library/alpine"),         String::from("3.7")));
        assert_eq!(split("docker.example.com/examplerepo/alpine:3.7"),                                                                                       (String::from("docker.example.com"),      String::from("examplerepo/alpine"),             String::from("3.7")));
        assert_eq!(split("docker.example.com/examplerepo/alpine/test2:3.7"),                                                                                 (String::from("docker.example.com"),      String::from("examplerepo/alpine/test2"),       String::from("3.7")));
        assert_eq!(split("docker.example.com/examplerepo/alpine/test2/test3:3.7"),                                                                           (String::from("docker.example.com"),      String::from("examplerepo/alpine/test2/test3"), String::from("3.7")));
        assert_eq!(split("docker.example.com:5000/examplerepo/alpine:latest"),                                                                               (String::from("docker.example.com:5000"), String::from("examplerepo/alpine"),             String::from("latest")));
        assert_eq!(split("portainer/portainer:latest"),                                                                                              (String::from(DEFAULT_REGISTRY),          String::from("portainer/portainer"),    String::from("latest")));
    }
}
