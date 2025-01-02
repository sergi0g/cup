// Functions that return JSON data, used for generating output and API responses

use serde_json::{json, Map, Value};

use crate::structs::{image::Image, status::Status};

/// Helper function to get metrics used in JSON output
pub fn get_metrics(updates: &[Image]) -> Value {
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
    json!({
        "monitored_images": updates.len(),
        "up_to_date": up_to_date,
        "updates_available": major_updates + minor_updates + patch_updates + other_updates,
        "major_updates": major_updates,
        "minor_updates": minor_updates,
        "patch_updates": patch_updates,
        "other_updates": other_updates,
        "unknown": unknown
    })
}

/// Takes a slice of `Image` objects and returns a `Value` with update info. The output doesn't contain much detail
pub fn to_simple_json(updates: &[Image]) -> Value {
    let mut images = Map::new();
    updates.iter().for_each(|image| {
        let _ = images.insert(
            image.reference.clone(),
            image.has_update().to_option_bool().into(),
        );
    });
    let json_data: Value = json!({
        "metrics": get_metrics(updates),
        "images": images,
    });
    json_data
}

/// Takes a slice of `Image` objects and returns a `Value` with update info. All image data is included, useful for debugging.
pub fn to_full_json(updates: &[Image]) -> Value {
    json!({
        "metrics": get_metrics(updates),
        "images": updates.iter().map(|image| image.to_json()).collect::<Vec<Value>>(),
    })
}
