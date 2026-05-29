use std::fmt::Write as FmtWrite;
use std::path::{Path, PathBuf};

use crate::simulation::fee_model::FeePoint;

/// Arguments for the `export` subcommand.
pub struct ExportArgs {
    /// Source SQLite database path.
    pub db: PathBuf,
    /// Output file path. Writes to stdout when `None`.
    pub output: Option<PathBuf>,
}

impl ExportArgs {
    /// Run the export, writing CSV to file or stdout.
    pub fn run(&self, points: &[FeePoint]) {
        match &self.output {
            Some(path) => Export::write_csv(points, path).expect("failed to write output"),
            None => print!("{}", Export::to_csv(points)),
        }
    }
}

/// Exports devkit results to external formats.
pub struct Export;

impl Export {
    /// Serialize fee points to CSV.
    pub fn to_csv(points: &[FeePoint]) -> String {
        let mut out = String::from("timestamp,fee,ledger,is_spike\n");
        for p in points {
            writeln!(out, "{},{},{},{}", p.timestamp, p.fee, p.ledger, p.is_spike).unwrap();
        }
        out
    }

    /// Write fee points to a CSV file.
    pub fn write_csv(points: &[FeePoint], path: &Path) -> std::io::Result<()> {
        std::fs::write(path, Self::to_csv(points))
    }
}
