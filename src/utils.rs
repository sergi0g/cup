use chrono::Local;
use json::{object, JsonValue};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

use crate::image::{Image, Status};

/// Sorts the update vector alphabetically and where Some(true) > Some(false) > None
pub fn sort_image_vec(updates: &[Image]) -> Vec<Image> {
    let mut sorted_updates = updates.to_vec();
    sorted_updates.sort_unstable_by_key(|img| img.has_update());
    sorted_updates.to_vec()
}

/// Helper function to get metrics used in JSON output
pub fn get_metrics(updates: &[Image]) -> JsonValue {
    let mut up_to_date = 0;
    let mut major_updates = 0;
    let mut minor_updates = 0;
    let mut patch_updates = 0;
    let mut other_updates = 0;
    let mut unknown = 0;
    updates.iter().for_each(|image| {
        let has_update = image.has_update();
        match has_update {
            Status::UpdateMajor => {
                major_updates += 1;
            }
            Status::UpdateMinor => {
                minor_updates += 1;
            }
            Status::UpdatePatch => {
                patch_updates += 1;
            }
            Status::UpdateAvailable => {
                other_updates += 1;
            }
            Status::UpToDate => {
                up_to_date += 1;
            }
            Status::Unknown(_) => {
                unknown += 1;
            }
        };
    });
    object! {
        monitored_images: updates.len(),
        up_to_date: up_to_date,
        major_updates: major_updates,
        minor_updates: minor_updates,
        patch_updates: patch_updates,
        other_updates: other_updates,
        unknown: unknown
    }
}

/// Takes a slice of `Image` objects and returns a `JsonValue` of update info. The output doesn't contain much detail
pub fn to_simple_json(updates: &[Image]) -> JsonValue {
    let mut json_data: JsonValue = object! {
        metrics: get_metrics(updates),
        images: object! {}
    };
    updates.iter().for_each(|image| {
        let _ = json_data["images"].insert(&image.reference, image.has_update().to_option_bool());
    });
    json_data
}

/// Takes a slice of `Image` objects and returns a `JsonValue` of update info. All image data is included, useful for debugging.
pub fn to_full_json(updates: &[Image]) -> JsonValue {
    object! {
        metrics: get_metrics(updates),
        images: updates.iter().map(|image| image.to_json()).collect::<Vec<JsonValue>>(),
    }
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

/// Creates a new reqwest client with automatic retries
pub fn new_reqwest_client() -> ClientWithMiddleware {
    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3),
        ))
        .build()
}

pub fn timestamp() -> i64 {
    Local::now().timestamp_millis()
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
            local_digests: Some(vec![
                "some_digest".to_string(),
                "some_other_digest".to_string(),
            ]),
            remote_digest: Some("latest_digest".to_string()),
            ..Default::default()
        };
        let update_available_2 = Image {
            reference: "library/alpine".to_string(),
            local_digests: Some(vec![
                "some_digest".to_string(),
                "some_other_digest".to_string(),
            ]), // We don't need to mock real data, as this is a generic function
            remote_digest: Some("latest_digest".to_string()),
            ..Default::default()
        };
        let up_to_date_1 = Image {
            reference: "docker:dind".to_string(),
            local_digests: Some(vec![
                "some_digest".to_string(),
                "some_other_digest".to_string(),
                "latest_digest".to_string(),
            ]),
            remote_digest: Some("latest_digest".to_string()),
            ..Default::default()
        };
        let up_to_date_2 = Image {
            reference: "ghcr.io/sergi0g/cup".to_string(),
            local_digests: Some(vec![
                "some_digest".to_string(),
                "some_other_digest".to_string(),
                "latest_digest".to_string(),
            ]),
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
        let input_vec = vec![
            unknown_2.clone(),
            up_to_date_1.clone(),
            unknown_1.clone(),
            update_available_2.clone(),
            update_available_1.clone(),
            up_to_date_2.clone(),
        ];
        let expected_vec = vec![
            update_available_1,
            update_available_2,
            up_to_date_1,
            up_to_date_2,
            unknown_1,
            unknown_2,
        ];

        // Sort the vec
        let sorted_vec = sort_image_vec(&input_vec);

        // Check results
        assert_eq!(sorted_vec, expected_vec);
    }
}
