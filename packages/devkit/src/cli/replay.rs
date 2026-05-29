use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};

/// Replays recorded fee scenarios, optionally showing a progress bar.
pub struct ReplayArgs {
    /// Path to the SQLite database file.
    pub db: PathBuf,
    /// Show a progress bar during replay.
    pub progress: bool,
}

impl ReplayArgs {
    /// Run the replay for `total` records.
    pub fn run(&self, total: u64) {
        if !self.progress {
            eprintln!("Replaying {} records from {}", total, self.db.display());
            return;
        }
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} records")
                .expect("invalid template"),
        );
        for _ in 0..total {
            bar.inc(1);
        }
        bar.finish_with_message("replay complete");
    }
}
