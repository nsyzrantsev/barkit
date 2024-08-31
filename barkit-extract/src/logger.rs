use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};

pub static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

pub struct CustomProgressBar;

impl CustomProgressBar {
    pub fn new(file_lines_number: usize, quiet: bool) -> Option<ProgressBar> {
        if !quiet {
            println!("{} Estimating reads count...", style("[2/3]").bold().dim());
            return Some(Self::create_progress_bar(file_lines_number));
        }
        None
    }

    fn create_progress_bar(file_lines_number: usize) -> ProgressBar {
        let progress_bar_style = ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {per_sec} ({eta})",
        )
        .expect("Failed to parse a progress bar template")
        .progress_chars("##-");

        let progress_bar = ProgressBar::new(file_lines_number as u64);
        progress_bar.set_style(progress_bar_style);

        progress_bar
    }
}
