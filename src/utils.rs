use json::{object, JsonValue};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

use crate::image::{Image, Status};

/// Sorts the update vector alphabetically and where Some(true) > Some(false) > None
pub fn sort_image_vec(updates: &[Image]) -> Vec<Image> {
    let mut sorted_updates = updates.to_vec();
    sorted_updates.sort_unstable_by(|a, b| match (a.has_update(), b.has_update()) {
        (Status::UpdateAvailable, Status::UpdateAvailable) => {
            a.reference.cmp(&b.reference)
        }
        (Status::UpdateAvailable, Status::UpToDate | Status::Unknown(_)) => std::cmp::Ordering::Less,
        (Status::UpToDate, Status::UpdateAvailable) => std::cmp::Ordering::Greater,
        (Status::UpToDate, Status::UpToDate) => {
            a.reference.cmp(&b.reference)
        },
        (Status::UpToDate, Status::Unknown(_)) => std::cmp::Ordering::Less,
        (Status::Unknown(_), Status::UpdateAvailable | Status::UpToDate) => std::cmp::Ordering::Greater,
        (Status::Unknown(_), Status::Unknown(_)) => {
            a.reference.cmp(&b.reference)
        }
    });
    sorted_updates.to_vec()
}

/// Takes a slice of `Image` objects and returns a `JsonValue` of update info. The output doesn't contain much detail
pub fn to_simple_json(updates: &[Image]) -> JsonValue {
    let mut json_data: JsonValue = object! {
        metrics: object! {},
        images: object! {}
    };
    let mut up_to_date = 0;
    let mut update_available = 0;
    let mut unknown = 0;
    updates.iter().for_each(|image| {
        let has_update = image.has_update();
        match has_update {
            Status::UpdateAvailable => {
                update_available += 1;
            },
            Status::UpToDate => {
                up_to_date += 1;
            },
            Status::Unknown(_) => {
                unknown += 1;
            }
        };
        let _ = json_data["images"].insert(&image.reference, has_update.to_option_bool());
    });
    let _ = json_data["metrics"].insert("monitored_images", updates.len());
    let _ = json_data["metrics"].insert("up_to_date", up_to_date);
    let _ = json_data["metrics"].insert("update_available", update_available);
    let _ = json_data["metrics"].insert("unknown", unknown);
    json_data
}

// Logging

/// This macro is an alternative to panic. It prints the message you give it and exits the process with code 1, without printing a stack trace. Useful for when the program has to exit due to a user error or something unexpected which is unrelated to the program (e.g. a failed web request)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        eprintln!("\x1b[38:5:204mERROR \x1b[0m {}", format!($($arg)*));
        std::process::exit(1);
    })
}

// A small macro to print in yellow as a warning
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        eprintln!("\x1b[38:5:192mWARN \x1b[0m {}", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        println!("\x1b[38:5:86mINFO \x1b[0m {}", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        println!("\x1b[38:5:63mDEBUG \x1b[0m {}", format!($($arg)*));
    })
}

pub fn new_reqwest_client() -> ClientWithMiddleware {
    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3),
        ))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the `sort_update_vec` function
    #[test]
    fn test_ordering() {
        // Create test objects
        let update_available_1 = Image {
            reference: "busybox".to_string(),
            local_digests: Some(vec!["some_digest".to_string(), "some_other_digest".to_string()]),
            remote_digest: Some("latest_digest".to_string()),
            ..Default::default()
        };
        let update_available_2 = Image {
            reference: "library/alpine".to_string(),
            local_digests: Some(vec!["some_digest".to_string(), "some_other_digest".to_string()]), // We don't need to mock real data, as this is a generic function
            remote_digest: Some("latest_digest".to_string()),
            ..Default::default()
        };
        let up_to_date_1 = Image {
            reference: "docker:dind".to_string(),
            local_digests: Some(vec!["some_digest".to_string(), "some_other_digest".to_string(), "latest_digest".to_string()]),
            remote_digest: Some("latest_digest".to_string()),
            ..Default::default()
        };
        let up_to_date_2 = Image {
            reference: "ghcr.io/sergi0g/cup".to_string(),
            local_digests: Some(vec!["some_digest".to_string(), "some_other_digest".to_string(), "latest_digest".to_string()]),
            remote_digest: Some("latest_digest".to_string()),
            ..Default::default()
        };
        let unknown_1 = Image {
            reference: "fake_registry.com/fake/image".to_string(),
            error: Some("whoops".to_string()),
            ..Default::default()
        };
        let unknown_2 = Image {
            reference: "private_registry.io/private/image".to_string(),
            error: Some("whoops".to_string()),
            ..Default::default()
        };
        let input_vec = vec![unknown_2.clone(), up_to_date_1.clone(), unknown_1.clone(), update_available_2.clone(), update_available_1.clone(), up_to_date_2.clone()];
        let expected_vec = vec![update_available_1, update_available_2, up_to_date_1, up_to_date_2, unknown_1, unknown_2];

        // Sort the vec
        let sorted_vec = sort_image_vec(&input_vec);

        // Check results
        assert_eq!(sorted_vec, expected_vec);
    }
}