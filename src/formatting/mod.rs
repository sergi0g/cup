pub mod spinner;

use crate::{
    structs::{image::Image, status::Status},
    utils::{json::to_simple_json, sort_update_vec::sort_image_vec},
};

pub fn print_updates(updates: &[Image], icons: &bool) {
    let sorted_images = sort_image_vec(updates);
    let term_width: usize = termsize::get()
        .unwrap_or(termsize::Size { rows: 24, cols: 80 })
        .cols as usize;
    for image in sorted_images {
        let has_update = image.has_update();
        let description = has_update.to_string();
        let icon = if *icons {
            match has_update {
                Status::UpToDate => "\u{f058} ",
                Status::Unknown(_) => "\u{f059} ",
                _ => "\u{f0aa} ",
            }
        } else {
            ""
        };
        let color = match has_update {
            Status::UpdateAvailable | Status::UpdatePatch => "\u{001b}[38;5;12m",
            Status::UpdateMinor => "\u{001b}[38;5;3m",
            Status::UpdateMajor => "\u{001b}[38;5;1m",
            Status::UpToDate => "\u{001b}[38;5;2m",
            Status::Unknown(_) => "\u{001b}[38;5;8m",
        };
        let dynamic_space =
            " ".repeat(term_width - description.len() - icon.len() - image.reference.len());
        println!(
            "{}{}{}{}{}\u{001b}[0m",
            color, icon, image.reference, dynamic_space, description
        );
    }
}

pub fn print_raw_updates(updates: &[Image]) {
    println!("{}", to_simple_json(updates));
}
