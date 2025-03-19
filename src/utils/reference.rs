const DEFAULT_REGISTRY: &str = "registry-1.docker.io";

/// Takes an image and splits it into registry, repository and tag, based on the reference.
/// For example, `ghcr.io/sergi0g/cup:latest` becomes `['ghcr.io', 'sergi0g/cup', 'latest']`.
pub fn split(reference: &str) -> (String, String, String) {
    let splits = reference.split('/').collect::<Vec<&str>>();
    let (registry, repository_and_tag) = match splits.len() {
        0 => unreachable!(),
        1 => (DEFAULT_REGISTRY, reference.to_string()),
        _ => {
            // Check if we're looking at a domain
            if splits[0] == "localhost" || splits[0].contains('.') || splits[0].contains(':') {
                (splits[0], splits[1..].join("/"))
            } else {
                (DEFAULT_REGISTRY, reference.to_string())
            }
        }
    };
    let splits = repository_and_tag
        .split('@')
        .next()
        .unwrap()
        .split(':')
        .collect::<Vec<&str>>();
    let (repository, tag) = match splits.len() {
        1 | 2 => {
            let repository_components = splits[0].split('/').collect::<Vec<&str>>();
            let repository = match repository_components.len() {
                0 => unreachable!(),
                1 => {
                    if registry == DEFAULT_REGISTRY {
                        format!("library/{}", repository_components[0])
                    } else {
                        splits[0].to_string()
                    }
                }
                _ => splits[0].to_string(),
            };
            let tag = match splits.len() {
                1 => "latest",
                2 => splits[1],
                _ => unreachable!(),
            };
            (repository, tag)
        }
        _ => {dbg!(splits); panic!()},
    };
    (registry.to_string(), repository, tag.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn reference() {
        assert_eq!(split("alpine"                                               ), (String::from(DEFAULT_REGISTRY         ), String::from("library/alpine"                ), String::from("latest")));
        assert_eq!(split("alpine:latest"                                        ), (String::from(DEFAULT_REGISTRY         ), String::from("library/alpine"                ), String::from("latest")));
        assert_eq!(split("library/alpine"                                       ), (String::from(DEFAULT_REGISTRY         ), String::from("library/alpine"                ), String::from("latest")));
        assert_eq!(split("localhost/test"                                       ), (String::from("localhost"              ), String::from("test"                          ), String::from("latest")));
        assert_eq!(split("localhost:1234/test"                                  ), (String::from("localhost:1234"         ), String::from("test"                          ), String::from("latest")));
        assert_eq!(split("test:1234/idk"                                        ), (String::from("test:1234"              ), String::from("idk"                           ), String::from("latest")));
        assert_eq!(split("alpine:3.7"                                           ), (String::from(DEFAULT_REGISTRY         ), String::from("library/alpine"                ), String::from("3.7"   )));
        assert_eq!(split("docker.example.com/examplerepo/alpine:3.7"            ), (String::from("docker.example.com"     ), String::from("examplerepo/alpine"            ), String::from("3.7"   )));
        assert_eq!(split("docker.example.com/examplerepo/alpine/test2:3.7"      ), (String::from("docker.example.com"     ), String::from("examplerepo/alpine/test2"      ), String::from("3.7"   )));
        assert_eq!(split("docker.example.com/examplerepo/alpine/test2/test3:3.7"), (String::from("docker.example.com"     ), String::from("examplerepo/alpine/test2/test3"), String::from("3.7"   )));
        assert_eq!(split("docker.example.com:5000/examplerepo/alpine:latest"    ), (String::from("docker.example.com:5000"), String::from("examplerepo/alpine"            ), String::from("latest")));
        assert_eq!(split("portainer/portainer:latest"                           ), (String::from(DEFAULT_REGISTRY         ), String::from("portainer/portainer"           ), String::from("latest")));
    }
}
