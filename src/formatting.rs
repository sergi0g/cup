use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use json::object;

use crate::utils::{sort_update_vec, to_json};

pub fn print_updates(updates: &[(String, Option<bool>)], icons: &bool) {
    let sorted_updates = sort_update_vec(updates);
    let term_width: usize = termsize::get()
        .unwrap_or_else(|| termsize::Size { rows: 24, cols: 80 })
        .cols as usize;
    for update in sorted_updates {
        let description = match update.1 {
            Some(true) => "Update available",
            Some(false) => "Up to date",
            None => "Unknown",
        };
        let icon = if *icons {
            match update.1 {
                Some(true) => "\u{f0aa} ",
                Some(false) => "\u{f058} ",
                None => "\u{f059} ",
            }
        } else {
            ""
        };
        let color = match update.1 {
            Some(true) => "\u{001b}[38;5;12m",
            Some(false) => "\u{001b}[38;5;2m",
            None => "\u{001b}[38;5;8m",
        };
        let dynamic_space =
            " ".repeat(term_width - description.len() - icon.len() - update.0.len());
        println!(
            "{}{}{}{}{}",
            color, icon, update.0, dynamic_space, description
        );
    }
}

pub fn print_raw_updates(updates: &[(String, Option<bool>)]) {
    println!("{}", json::stringify(to_json(updates)));
}

pub fn print_update(name: &str, has_update: &Option<bool>) {
    let color = match has_update {
        Some(true) => "\u{001b}[38;5;12m",
        Some(false) => "\u{001b}[38;5;2m",
        None => "\u{001b}[38;5;8m",
    };
    let description = match has_update {
        Some(true) => "has an update available",
        Some(false) => "is up to date",
        None => "wasn't found",
    };
    println!("{}{} {}", color, name, description);
}

pub fn print_raw_update(name: &str, has_update: &Option<bool>) {
    let result = object! {images: {[name]: *has_update}};
    println!("{}", result);
}

pub struct Spinner {
    spinner: ProgressBar,
}

impl Spinner {
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
