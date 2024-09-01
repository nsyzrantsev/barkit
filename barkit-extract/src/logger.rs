use std::time::Instant;

use console::{style, Emoji};
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};

pub static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
static PROGRESS_BAR_TEMPLATE: &str =
    "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {per_sec} ({eta})";

pub struct Logger {
    /// Index of the current step
    current: usize,

    /// Total number of steps
    total: usize,

    /// If true, logs will not be displayed
    quiet: bool,

    /// Start time of execution
    execution_start: Instant,

    /// Progress bar
    progress_bar: Option<ProgressBar>,
}

impl Logger {
    /// Creates `Logger` instance with default values.
    ///
    /// # Example
    ///
    /// ```
    /// use barkit_extract::logger::Logger;
    ///
    /// let logger = Logger::new(3, false);
    /// ```
    pub fn new(total: usize, quiet: bool) -> Self {
        Self {
            current: 0,
            total,
            quiet,
            execution_start: Instant::now(),
            progress_bar: None,
        }
    }

    /// Prints logging message for the current step.
    ///
    /// # Example
    ///
    /// ```
    /// use barkit_extract::logger::Logger;
    ///
    /// let mut logger = Logger::new(2, false);
    ///
    /// logger.message("first logging message");
    /// // Output: "[1/2] first logging message"
    ///
    /// logger.message("second logging message");
    /// // Output: "[2/2] second logging message"
    ///
    /// logger.message("third logging message");
    /// // Output: "Warning: Current step exceeds total steps."
    /// ```
    pub fn message(&mut self, text: &str) {
        if self.current < self.total {
            self.current += 1;
            if !self.quiet {
                println!(
                    "{} {}",
                    style(format!("[{}/{}]", self.current, self.total))
                        .bold()
                        .dim(),
                    text
                );
            }
        } else {
            // Optionally handle the case where `current` exceeds `total`
            eprintln!("Warning: Current step exceeds total steps.");
        }
    }

    /// Increments progress in the progress bar
    pub fn increment_progress(&self, done_lines: usize) {
        if let Some(ref pb) = self.progress_bar {
            pb.inc(done_lines as u64)
        }
    }

    /// Sets progress bar instance with specified length
    pub fn set_progress_bar(&mut self, size: usize) {
        if !self.quiet {
            let progress_bar_style = ProgressStyle::with_template(PROGRESS_BAR_TEMPLATE)
                .expect("Failed to parse a progress bar template")
                .progress_chars("##-");

            let progress_bar = ProgressBar::new(size as u64);
            progress_bar.set_style(progress_bar_style);

            self.progress_bar = Some(progress_bar);
        }
    }

    /// Prints a final message when all steps are completed
    pub fn final_message(&self) {
        if self.progress_bar.is_some() {
            println!(
                "{} Done in {}",
                SPARKLE,
                HumanDuration(self.execution_start.elapsed())
            )
        }
    }
}
