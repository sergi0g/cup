use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use crate::{image::{Image, Status}, utils::{sort_image_vec, to_simple_json}};

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
                Status::UpdateAvailable => "\u{f0aa} ",
                Status::UpToDate => "\u{f058} ",
                Status::Unknown(_) => "\u{f059} ",
            }
        } else {
            ""
        };
        let color = match has_update {
            Status::UpdateAvailable => "\u{001b}[38;5;12m",
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
    println!("{}", json::stringify(to_simple_json(updates)));
}

pub struct Spinner {
    spinner: ProgressBar,
}

impl Spinner {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Spinner {
        let spinner = ProgressBar::new_spinner();
        let style: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let progress_style = ProgressStyle::default_spinner();

        spinner.set_style(ProgressStyle::tick_strings(progress_style, style));

        spinner.set_message("Checking...");
        spinner.enable_steady_tick(Duration::from_millis(50));

        Spinner { spinner }
    }
    pub fn succeed(&self) {
        const CHECKMARK: &str = "\u{001b}[32;1m\u{2713}\u{001b}[0m";

        let success_message = format!("{} Done!", CHECKMARK);
        self.spinner
            .set_style(ProgressStyle::with_template("{msg}").unwrap());
        self.spinner.finish_with_message(success_message);
    }
}
