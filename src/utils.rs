use fancy_regex::Regex;
use once_cell::sync::Lazy;

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        std::process::exit(1);
    })
}

// Takes an image and splits it into registry, repository and tag. For example ghcr.io/sergi0g/cup:latest becomes ['ghcr.io', 'sergi0g/cup', 'latest']. ONLY REGISTRIES THAT USE A / IN THE REPOSITORY ARE SUPPORTED CURRENTLY. THAT MEANS AZURE WILL NOT WORK.
pub fn split_image(image: &str) -> (String, String, String) {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"^(?P<registry>[\w.\-_]+((?::\d+|)(?=/[a-z0-9._-]+/[a-z0-9._-]+))|)(?:/|)(?P<repository>[a-z0-9.\-_]+(?:/[a-z0-9.\-_]+|))(:(?P<tag>[\w.\-_]{1,127})|)$"#, // From https://regex101.com/r/a98UqN/1
        )
        .unwrap()
    });
    match RE.captures(image).unwrap() {
        Some(c) => {
            return (
                match c.name("registry") {
                    Some(registry) => {
                        let reg = registry.as_str().to_owned();
                        if reg.is_empty() {
                            String::from("registry-1.docker.io")
                        } else {
                            reg
                        }
                    }
                    None => error!("Failed to parse image {}", image),
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

pub fn unsplit_image(registry: &str, repository: &str, tag: &str) -> String {
    let reg = match registry {
        "registry-1.docker.io" => "",
        r => &format!("{}/", r),
    };
    let repo = match repository.split('/').collect::<Vec<&str>>()[0] {
        "library" => repository.strip_prefix("library/").unwrap(),
        _ => repository,
    };
    format!("{}{}:{}", reg, repo, tag)
}

#[cfg(feature = "cli")]
pub fn sort_update_vec(updates: &[(String, Option<bool>)]) -> Vec<(String, Option<bool>)> {
    let mut sorted_updates = updates.to_vec();
    sorted_updates.sort_unstable_by(|a, b| match (a.1, b.1) {
        (Some(a), Some(b)) => (!a).cmp(&!b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    sorted_updates.to_vec()
}
