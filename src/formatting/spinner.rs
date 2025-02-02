use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

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
