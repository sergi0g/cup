use std::collections::HashMap;

use bollard::{secret::ImageSummary, ClientVersion, Docker};
use clap::{Parser, Subcommand};
use console::Term;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use tokio::runtime::Runtime;
use lazy_static::lazy_static;

macro_rules! error {
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        std::process::exit(1);
    })
}

lazy_static! {
    static ref CONSOLE: Term = Term::stdout();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    #[arg(short, long, default_value = "unix:///var/run/docker.sock")]
    socket: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Check {
        #[arg(default_value = None)]
        image: Option<String>,
    },
}

struct SyncDockerClient {
    inner: Docker,
    rt: Runtime,
}

impl SyncDockerClient {
    pub fn create(socket: String) -> Result<SyncDockerClient, bollard::errors::Error> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let inner = match Docker::connect_with_local(
            &socket,
            120,
            &ClientVersion {
                major_version: 1,
                minor_version: 44,
            },
        ) {
            Ok(docker) => docker,
            Err(e) => {
                error!("Failed to connect to the docker socket!\n{}", e)
            }
        };
        Ok(SyncDockerClient { inner, rt })
    }
    pub fn list_images(&self) -> Result<Vec<ImageSummary>, bollard::errors::Error> {
        self.rt.block_on(self.inner.list_images::<String>(None))
    }
}

fn main() {
    let cli = CLI::parse();

    match &cli.command {
        Some(Commands::Check { image }) => match image {
            Some(name) => {
                get_updates(name.clone(), cli.socket);
            }
            None => {
                get_all_updates(cli.socket);
            }
        },
        None => {}
    }
}

fn list_images(socket: String) -> Vec<ImageSummary> {
    let client = match SyncDockerClient::create(socket) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create docker client!\n{}", e)
        }
    };
    match client.list_images() {
        Ok(images) => images,
        Err(e) => {
            error!("Failed to retrieve list of images available!\n{}", e)
        }
    }
}

fn get_updates(image: String, socket: String) {
    match has_update(get_metadata(image.clone(), socket)) {
        Some(value) => {
            if value {
                CONSOLE.write_line(&format!("Image {} has an update", image)).unwrap();
            } else {
                CONSOLE.write_line(&format!("No updates available for image {}", image)).unwrap();
            }
        }
        None => {
            CONSOLE.write_line(&format!("Can't check for updates to {}", image)).unwrap()
        }
    }
}

fn get_all_updates(socket: String) {
    let mut list: HashMap<String, Option<bool>> = HashMap::new();
    let images = list_images(socket);
    let bar = ProgressBar::new(images.len() as u64);
    bar.set_style(ProgressStyle::with_template("{bar:50.cyan/blue} {wide_msg}   Overall progress: {pos:>4}/{len:4}")
    .unwrap()
    .progress_chars("##-"));
    for image in images {
        if image.repo_tags.len() != 0 {
            bar.set_message(String::from("Checking") + &image.repo_tags[0].clone());
        };
        let update_available = has_update(image.clone());
        for tag in image.repo_tags {
            list.insert(tag, update_available);
        }
        bar.inc(1);
    }
    bar.finish_with_message("Done!\n");
    let outdated_images = list
        .iter()
        .filter(|&(_, &image)| match image {
            Some(value) => {
                if value {
                    return true;
                } else {
                    return false;
                }
            }
            None => return false,
        })
        .map(|(key, _)| key.to_string())
        .collect::<Vec<String>>();
    if outdated_images.len() > 0 {
        CONSOLE.write_line("The following images have updates:").unwrap();
        outdated_images
            .iter()
            .for_each(|image| CONSOLE.write_line(&format!("{}", image)).unwrap());
        CONSOLE.write_line("").unwrap();
    }
    let unprocessable_images = list
        .iter()
        .filter(|&(_, &image)| match image {
            Some(value) => {
                if value {
                    return false;
                } else {
                    return false;
                }
            }
            None => return true,
        })
        .map(|(key, _)| key.to_string())
        .collect::<Vec<String>>();
    if unprocessable_images.len() > 0 {
        CONSOLE.write_line("The following images couldn't be processed:").unwrap();
        unprocessable_images
            .iter()
            .for_each(|image| CONSOLE.write_line(&format!("{}", image)).unwrap());
    }
}

fn has_update(metadata: ImageSummary) -> Option<bool> {
    if metadata.repo_tags.len() == 0 || metadata.repo_digests.len() == 0 {
        return None;
    };
    let (repo, tag, uncleaned_repo) = split_image(&metadata.repo_tags[0]);
    if repo == "" {
        return None;
    };
    let latest_digest = get_digest(&repo, &tag, &get_token(&repo));
    if metadata
        .repo_digests
        .iter()
        .any(|digest| digest == &format!("{}@{}", uncleaned_repo, latest_digest))
    {
        return Some(false);
    }
    Some(true)
}

fn split_image(mut image: &str) -> (String, String, String) {
    let slash_count = image.chars().filter(|&c| c == '/').count();
    if slash_count > 1 {
        return (String::new(), String::new(), String::new());
    };
    if image.starts_with("library/") {
        image = image.split("library/").collect::<Vec<&str>>()[1]
    }
    let split = image.split(":").collect::<Vec<&str>>();
    let repo = if image.contains("/") {
        split[0].to_string()
    } else {
        format!("{}{}", "library/", split[0].to_string())
    };
    let tag = split[1].to_string();
    (repo, tag, split[0].to_string())
}

fn get_metadata(mut image: String, socket: String) -> ImageSummary {
    let images = list_images(socket);
    if !image.contains(":") {
        image = format!("{}{}", image, ":latest")
    }
    if image.starts_with("library/") {
        image = image.split("library/").collect::<Vec<&str>>()[1].to_string()
    }
    images
        .iter()
        .filter(|img| img.repo_tags.contains(&image.to_string()))
        .cloned()
        .collect::<Vec<ImageSummary>>()[0]
        .to_owned()
}

fn get_digest(repo: &str, tag: &str, token: &str) -> String {
    let raw_response = match ureq::head(&format!(
        "https://registry-1.docker.io/v2/{}/manifests/{}",
        repo, tag
    ))
    .set("Authorization", &format!("Bearer {}", token))
    .set(
        "Accept",
        "application/vnd.docker.distribution.manifest.list.v2+json",
    )
    .call()
    {
        Ok(response) => response,
        Err(e) => {
            error!("Manifest request failed!\n{}", e)
        }
    };
    match raw_response.header("docker-content-digest") {
        Some(digest) => digest.to_string(),
        None => {
            error!("Server returned invalid response!")
        }
    }
}

fn get_token(scope: &str) -> String {
    let raw_response = match ureq::get(&format!(
        "https://auth.docker.io/token?service=registry.docker.io&scope=repository:{}:pull",
        scope
    ))
    .set(
        "Accept",
        "application/vnd.docker.distribution.manifest.list.v2+json",
    )
    .call()
    {
        Ok(response) => match response.into_string() {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to parse response into string!\n{}", e)
            }
        },
        Err(e) => {
            error!("Manifest request failed!\n{}", e)
        }
    };
    let parsed_token_response = match json::parse(&raw_response) {
        Ok(parsed) => parsed,
        Err(e) => {
            error!("Failed to parse server response\n{}", e)
        }
    };
    parsed_token_response["token"].to_string()
}
