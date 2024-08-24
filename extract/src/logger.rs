use console::Emoji;
use indicatif::{ProgressBar, ProgressStyle};

use crate::io;
use crate::error::Error;

pub static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

pub fn create_progress_bar(fastq_file: &str, threads: usize, max_memory: Option<usize>) -> Result<ProgressBar, Error> {
    let progress_bar_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {per_sec} ({eta})",
    ).expect("Failed to parse a progress bar template")
    .progress_chars("##-");

    let read_num = io::get_lines_count(&fastq_file, threads, max_memory) as u64;

    let progress_bar = ProgressBar::new(read_num as u64);
    progress_bar.set_style(progress_bar_style.clone());

    Ok(progress_bar)
}