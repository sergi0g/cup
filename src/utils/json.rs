// Functions that return JSON data, used for generating output and API responses

use serde_json::{json, Value};

use crate::structs::{status::Status, update::Update};

/// Helper function to get metrics used in JSON output
pub fn get_metrics(updates: &[Update]) -> Value {
    let mut up_to_date = 0;
    let mut major_updates = 0;
    let mut minor_updates = 0;
    let mut patch_updates = 0;
    let mut other_updates = 0;
    let mut unknown = 0;
    updates.iter().for_each(|image| {
        let has_update = image.get_status();
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
    json!({
        "monitored_images": updates.len(),
        "updates_available": major_updates + minor_updates + patch_updates + other_updates,
        "major_updates": major_updates,
        "minor_updates": minor_updates,
        "patch_updates": patch_updates,
        "other_updates": other_updates,
        "up_to_date": up_to_date,
        "unknown": unknown
    })
}

/// Takes a slice of `Image` objects and returns a `Value` with update info. All image data is included, useful for debugging.
pub fn to_json(updates: &[Update]) -> Value {
    json!({
        "metrics": get_metrics(updates),
        "images": updates.iter().map(|update| serde_json::to_value(update).unwrap()).collect::<Vec<Value>>(),
    })
}
