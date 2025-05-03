use std::cmp::Ordering;

use crate::structs::update::Update;

/// Sorts the update vector alphabetically and by Status
pub fn sort_update_vec(updates: &[Update]) -> Vec<Update> {
    let mut sorted_updates = updates.to_vec();
    sorted_updates.sort_by(|a, b| {
        let cmp = a.get_status().cmp(&b.get_status());
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
    use crate::structs::{status::Status, update::UpdateResult};

    use super::*;

    /// Test the `sort_update_vec` function
    /// We test for sorting based on status (Major > Minor > Patch > Digest > Up to date > Unknown) and that references are sorted alphabetically.
    #[test]
    fn test_ordering() {
        // Create test objects
        let major_update_1 = create_major_update("redis:6.2"); // We're ignoring the tag we passed here, that is tested in version.rs
        let major_update_2 = create_major_update("traefik:v3.0");
        let minor_update_1 = create_minor_update("mysql:8.0");
        let minor_update_2 = create_minor_update("rust:1.80.1-alpine");
        let patch_update_1 = create_patch_update("node:20");
        let patch_update_2 = create_patch_update("valkey/valkey:7.2-alpine");
        let digest_update_1 = create_digest_update("busybox");
        let digest_update_2 = create_digest_update("library/alpine");
        let up_to_date_1 = create_up_to_date("docker:dind");
        let up_to_date_2 = create_up_to_date("ghcr.io/sergi0g/cup");
        let unknown_1 = create_unknown("fake_registry.com/fake/Update");
        let unknown_2 = create_unknown("private_registry.io/private/Update");
        let input_vec = vec![
            major_update_2.clone(),
            unknown_2.clone(),
            minor_update_2.clone(),
            patch_update_2.clone(),
            up_to_date_1.clone(),
            unknown_1.clone(),
            patch_update_1.clone(),
            digest_update_2.clone(),
            minor_update_1.clone(),
            major_update_1.clone(),
            digest_update_1.clone(),
            up_to_date_2.clone(),
        ];
        let expected_vec = vec![
            major_update_1,
            major_update_2,
            minor_update_1,
            minor_update_2,
            patch_update_1,
            patch_update_2,
            digest_update_1,
            digest_update_2,
            up_to_date_1,
            up_to_date_2,
            unknown_1,
            unknown_2,
        ];

        // Sort the vec
        let sorted_vec = sort_update_vec(&input_vec);

        // Check results
        assert_eq!(sorted_vec, expected_vec);
    }

    fn create_unknown(reference: &str) -> Update {
        Update {
            reference: reference.to_string(),
            status: Status::Unknown("".to_string()),
            result: UpdateResult {
                has_update: None,
                info: Default::default(),
                error: Some("Error".to_string()),
            },
            ..Default::default()
        }
    }

    fn create_up_to_date(reference: &str) -> Update {
        Update {
            reference: reference.to_string(),
            status: Status::UpToDate,
            ..Default::default()
        }
    }

    fn create_digest_update(reference: &str) -> Update {
        Update {
            reference: reference.to_string(),
            status: Status::UpdateAvailable,
            ..Default::default()
        }
    }

    fn create_patch_update(reference: &str) -> Update {
        Update {
            reference: reference.to_string(),
            status: Status::UpdatePatch,
            ..Default::default()
        }
    }

    fn create_minor_update(reference: &str) -> Update {
        Update {
            reference: reference.to_string(),
            status: Status::UpdateMinor,
            ..Default::default()
        }
    }

    fn create_major_update(reference: &str) -> Update {
        Update {
            reference: reference.to_string(),
            status: Status::UpdateMajor,
            ..Default::default()
        }
    }
}
