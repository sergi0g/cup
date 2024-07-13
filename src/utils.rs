use once_cell::sync::Lazy;
use regex::Regex;

/// This macro is an alternative to panic. It prints the message you give it and exits the process with code 1, without printing a stack trace. Useful for when the program has to exit due to a user error or something unexpected which is unrelated to the program (e.g. a failed web request)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        std::process::exit(1);
    })
}

/// Takes an image and splits it into registry, repository and tag. For example ghcr.io/sergi0g/cup:latest becomes ['ghcr.io', 'sergi0g/cup', 'latest'].
pub fn split_image(image: &str) -> (String, String, String) {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"^(?P<name>(?:(?P<registry>(?:(?:localhost|[\w-]+(?:\.[\w-]+)+)(?::\d+)?)|[\w]+:\d+)/)?(?P<repository>[a-z0-9_.-]+(?:/[a-z0-9_.-]+)*))(?::(?P<tag>[\w][\w.-]{0,127}))?$"#, // From https://regex101.com/r/nmSDPA/1
        )
        .unwrap()
    });
    match RE.captures(image) {
        Some(c) => {
            return (
                match c.name("registry") {
                    Some(registry) => registry.as_str().to_owned(),
                    None => String::from("registry-1.docker.io"),
                },
                match c.name("repository") {
                    Some(repository) => {
                        let repo = repository.as_str().to_owned();
                        if !repo.contains('/') {
                            format!("library/{}", repo)
                        } else {
                            repo
                        }
                    }
                    None => error!("Failed to parse image {}", image),
                },
                match c.name("tag") {
                    Some(tag) => tag.as_str().to_owned(),
                    None => String::from("latest"),
                },
            )
        }
        None => error!("Failed to parse image {}", image),
    }
}

/// Given an image's parts which were previously created by split_image, recreate a reference that docker would use. This means removing the registry part, if it's Docker Hub and removing "library" if the image is official
pub fn unsplit_image(registry: &str, repository: &str, tag: &str) -> String {
    let reg = match registry {
        "registry-1.docker.io" => String::new(),
        r => format!("{}/", r),
    };
    let repo = match repository.split('/').collect::<Vec<&str>>()[0] {
        "library" => {
            if reg.is_empty() {
                repository.strip_prefix("library/").unwrap()
            } else {
                repository
            }
        }
        _ => repository,
    };
    format!("{}{}:{}", reg, repo, tag)
}

/// Sorts the update vector alphabetically and where Some(true) > Some(false) > None
pub fn sort_update_vec(updates: &[(String, Option<bool>)]) -> Vec<(String, Option<bool>)> {
    let mut sorted_updates = updates.to_vec();
    sorted_updates.sort_unstable_by(|a, b| match (a.1, b.1) {
        (Some(c), Some(d)) => {
            if c == d {
                a.0.cmp(&b.0)
            } else {
                (!c).cmp(&!d)
            }
        }
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.0.cmp(&b.0),
    });
    sorted_updates.to_vec()
}
