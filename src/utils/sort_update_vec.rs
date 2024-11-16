use std::cmp::Ordering;

use crate::structs::image::Image;

/// Sorts the update vector alphabetically and where Some(true) > Some(false) > None
pub fn sort_image_vec(updates: &[Image]) -> Vec<Image> {
    let mut sorted_updates = updates.to_vec();
    sorted_updates.sort_by(|a, b| {
        let cmp = a.has_update().cmp(&b.has_update());
        if cmp == Ordering::Equal {
            a.reference.cmp(&b.reference)
        } else {
            cmp
        }
    });
    sorted_updates.to_vec()
}

#[cfg(test)]
mod tests {
    use crate::structs::image::DigestInfo;

    use super::*;

    /// Test the `sort_update_vec` function
    /// TODO: test versioning as well
    #[test]
    fn test_ordering() {
        // Create test objects
        let update_available_1 = Image {
            reference: "busybox".to_string(),
            digest_info: Some(DigestInfo {
                local_digests: vec!["some_digest".to_string(), "some_other_digest".to_string()],
                remote_digest: Some("latest_digest".to_string()),
            }),
            ..Default::default()
        };
        let update_available_2 = Image {
            reference: "library/alpine".to_string(),
            digest_info: Some(DigestInfo {
                local_digests: vec!["some_digest".to_string(), "some_other_digest".to_string()],
                remote_digest: Some("latest_digest".to_string()),
            }),
            ..Default::default()
        };
        let up_to_date_1 = Image {
            reference: "docker:dind".to_string(),
            digest_info: Some(DigestInfo {
                local_digests: vec![
                    "some_digest".to_string(),
                    "some_other_digest".to_string(),
                    "latest_digest".to_string(),
                ],
                remote_digest: Some("latest_digest".to_string()),
            }),
            ..Default::default()
        };
        let up_to_date_2 = Image {
            reference: "ghcr.io/sergi0g/cup".to_string(),
            digest_info: Some(DigestInfo {
                local_digests: vec![
                    "some_digest".to_string(),
                    "some_other_digest".to_string(),
                    "latest_digest".to_string(),
                ],
                remote_digest: Some("latest_digest".to_string()),
            }),
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
