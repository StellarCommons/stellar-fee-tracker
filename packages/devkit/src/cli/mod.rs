pub mod benchmark;
pub mod export;
pub mod replay;

use clap::{Parser, Subcommand};

/// Arguments for the `simulate` subcommand.
pub struct SimulateArgs {
    /// Base fee floor in stroops.
    pub base_fee: u64,
    /// Probability of a fee spike on any given ledger [0.0, 1.0].
    pub spike_prob: f64,
    /// Number of ledgers to simulate.
    pub duration: u64,
}

impl Default for SimulateArgs {
    fn default() -> Self {
        Self { base_fee: 100, spike_prob: 0.05, duration: 1_000 }
    }
}

impl SimulateArgs {
    /// Run the fee simulation and stream results to stdout.
    pub fn run(&self) {
        println!(
            "Simulating {} ledgers | base_fee={} stroops | spike_prob={:.2}",
            self.duration, self.base_fee, self.spike_prob
        );
    }
}

/// Arguments for the `mock` subcommand.
pub struct MockArgs {
    /// Scenario to load (e.g. "normal", "congested", "spike").
    pub scenario: String,
    /// Port the mock Horizon server listens on.
    pub port: u16,
}

impl Default for MockArgs {
    fn default() -> Self {
        Self { scenario: String::from("normal"), port: 8090 }
    }
}

impl MockArgs {
    /// Start the mock Horizon server with the configured scenario.
    pub fn run(&self) {
        println!(
            "Starting mock Horizon server on :{} with scenario '{}'",
            self.port, self.scenario
        );
    }
}

// ── validate (issue #391) ────────────────────────────────────────────────────

/// Arguments for the `validate` subcommand.
///
/// Runs data quality checks on a fee CSV or JSON file and prints a quality
/// report. Output format is controlled by `--output-format`.
///
/// # Usage
/// ```text
/// devkit validate --file fees.csv --format csv
/// devkit validate --file fees.json --format json --output-format json
/// ```
pub struct ValidateArgs {
    /// Path to the fee data file.
    pub file: std::path::PathBuf,
    /// Input format: "csv" or "json".
    pub format: String,
    /// Output format: "text" (default) or "json".
    pub output_format: String,
    /// Suppress all output except errors and the final result.
    pub quiet: bool,
}

impl Default for ValidateArgs {
    fn default() -> Self {
        Self {
            file: std::path::PathBuf::from("fees.csv"),
            format: "csv".into(),
            output_format: "text".into(),
            quiet: false,
        }
    }
}

impl ValidateArgs {
    /// Run the validate subcommand on the given fee points.
    pub fn run(&self, points: &[crate::simulation::fee_model::FeePoint]) {
        let report = crate::data_quality::validator::Validator::run(points);
        if self.output_format == "json" {
            println!("{}", report.to_json());
        } else {
            println!("{}", report.display());
        }
        if !self.quiet && !report.is_clean() {
            eprintln!("Validation failed: {} issue(s) found.", report.findings.len());
        }
    }
}

// ── repair (issue #392) ──────────────────────────────────────────────────────

/// Arguments for the `repair` subcommand.
///
/// Runs gap filling and duplicate removal on a fee data file.
/// Use `--dry-run` to preview changes without writing any output.
///
/// # Usage
/// ```text
/// devkit repair --file fees.csv --fill-gaps --remove-duplicates --output repaired.csv
/// devkit repair --file fees.csv --dry-run
/// ```
pub struct RepairArgs {
    /// Path to the fee data file.
    pub file: std::path::PathBuf,
    /// Fill gaps in the ledger sequence by interpolation.
    pub fill_gaps: bool,
    /// Remove duplicate ledger sequence numbers.
    pub remove_duplicates: bool,
    /// Preview changes without writing any output.
    pub dry_run: bool,
    /// Optional path to write the repaired output.
    pub output: Option<std::path::PathBuf>,
    /// Suppress all output except errors and the final result.
    pub quiet: bool,
}

impl Default for RepairArgs {
    fn default() -> Self {
        Self {
            file: std::path::PathBuf::from("fees.csv"),
            fill_gaps: false,
            remove_duplicates: false,
            dry_run: false,
            output: None,
            quiet: false,
        }
    }
}

impl RepairArgs {
    /// Run the repair subcommand on the given fee points.
    pub fn run(&self, points: &[crate::simulation::fee_model::FeePoint]) {
        let issues = crate::data_quality::repair::Repair::detect(points);
        let (cleaned, actions) = crate::data_quality::repair::Repair::apply(points, self.dry_run);

        if !self.quiet {
            println!("Repair report:");
            println!("  Issues found:  {}", issues.len());
            println!("  Actions:       {}", actions.len());
            println!("  Points before: {}", points.len());
            println!("  Points after:  {}", cleaned.len());
            if self.dry_run {
                println!("  (dry-run — no changes applied)");
            }
            for action in &actions {
                println!("  - {:?}", action);
            }
        }
    }
}

// ── compare (issue #393) ─────────────────────────────────────────────────────

/// Arguments for the `compare` subcommand.
///
/// Compares two fee data files and prints a distribution diff including
/// p50/p95/mean deltas and a volatility comparison.
///
/// # Usage
/// ```text
/// devkit compare --baseline fees_jan.csv --target fees_feb.csv
/// devkit compare --baseline fees_jan.csv --target fees_feb.csv --json
/// ```
pub struct CompareArgs {
    /// Path to the baseline fee data file.
    pub baseline: std::path::PathBuf,
    /// Path to the target fee data file.
    pub target: std::path::PathBuf,
    /// Label for the baseline dataset (defaults to the file name).
    pub label_baseline: String,
    /// Label for the target dataset (defaults to the file name).
    pub label_target: String,
    /// Output as JSON instead of text.
    pub json: bool,
    /// Suppress all output except errors and the final result.
    pub quiet: bool,
}

impl Default for CompareArgs {
    fn default() -> Self {
        Self {
            baseline: std::path::PathBuf::from("baseline.csv"),
            target: std::path::PathBuf::from("target.csv"),
            label_baseline: "baseline".into(),
            label_target: "target".into(),
            json: false,
            quiet: false,
        }
    }
}

impl CompareArgs {
    /// Run the compare subcommand on two slices of fee points.
    pub fn run(
        &self,
        baseline: &[crate::simulation::fee_model::FeePoint],
        target: &[crate::simulation::fee_model::FeePoint],
    ) {
        let result = crate::utilities::comparator::FeeComparator::compare(
            baseline, target,
            &self.label_baseline, &self.label_target,
        );
        if self.json {
            println!("{}", result.to_json());
        } else {
            println!("{}", result.display());
            if !self.quiet {
                println!("{}", result.verdict());
            }
        }
    }
}

// ── inspect (issue #394) ─────────────────────────────────────────────────────

/// Arguments for the `inspect` subcommand.
///
/// Prints a summary of a fee data file: record count, time range,
/// min/max/mean, and a full percentile table.
///
/// # Usage
/// ```text
/// devkit inspect --file fees.csv
/// devkit inspect --file fees.csv --json
/// ```
pub struct InspectArgs {
    /// Path to the fee data file.
    pub file: std::path::PathBuf,
    /// Output as JSON instead of text.
    pub json: bool,
    /// Suppress all output except errors and the final result.
    pub quiet: bool,
}

impl Default for InspectArgs {
    fn default() -> Self {
        Self {
            file: std::path::PathBuf::from("fees.csv"),
            json: false,
            quiet: false,
        }
    }
}

impl InspectArgs {
    /// Run the inspect subcommand on the given fee points.
    pub fn run(&self, points: &[crate::simulation::fee_model::FeePoint]) {
        if points.is_empty() {
            eprintln!("No fee points to inspect.");
            return;
        }

        let fees: Vec<u64> = points.iter().map(|p| p.fee).collect();
        let count = fees.len();
        let min   = *fees.iter().min().unwrap();
        let max   = *fees.iter().max().unwrap();
        let mean  = fees.iter().sum::<u64>() as f64 / count as f64;
        let ts_min = points.iter().map(|p| p.timestamp).min().unwrap_or(0);
        let ts_max = points.iter().map(|p| p.timestamp).max().unwrap_or(0);
        let spike_count = points.iter().filter(|p| p.is_spike).count();

        let table = crate::utilities::percentile_table::PercentileTable::from_fees(&fees);

        if self.json {
            let ptable_json = table.as_ref().map(|t| t.to_json()).unwrap_or_else(|| "{}".into());
            println!(
                r#"{{"count":{count},"min":{min},"max":{max},"mean":{mean:.2},"spike_count":{spike_count},"ts_min":{ts_min},"ts_max":{ts_max},"percentiles":{ptable_json}}}"#,
            );
        } else {
            println!("Fee Data Inspection");
            println!("===================");
            println!("records:     {count}");
            println!("time range:  {ts_min} - {ts_max}");
            println!("min fee:     {min} stroops");
            println!("max fee:     {max} stroops");
            println!("mean fee:    {mean:.2} stroops");
            println!("spikes:      {spike_count}");
            if let Some(t) = table {
                println!();
                println!("{}", t.display());
            }
        }
    }
}

// ── clap CLI wiring ──────────────────────────────────────────────────────────

/// Developer toolkit for the Stellar fee tracker.
#[derive(Parser)]
#[command(name = "devkit", about = "Stellar fee tracker developer toolkit")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Replay recorded fee scenarios
    Replay,
    /// Export data to external formats
    Export,
    /// Run performance benchmarks
    Benchmark,
    /// Serve mock fee data
    Mock,
    /// Validate fee data quality
    Validate,
    /// Repair fee data (gap fill, de-duplicate)
    Repair,
    /// Compare two fee data distributions
    Compare,
    /// Inspect a fee data file (summary + percentile table)
    Inspect,
}
