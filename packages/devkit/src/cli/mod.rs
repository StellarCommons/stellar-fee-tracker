//! Command-line interface for the devkit.
//!
//! Every subcommand is wired up here so that the surface is stable while the
//! individual subcommands are filled in one issue at a time.
//!
//! Closes #833.

pub mod version;

use std::io::Write;

use clap::{Parser, Subcommand};

/// Exit code reported by a subcommand that ran to completion.
pub const EXIT_SUCCESS: i32 = 0;

/// Exit code reported by a subcommand that could not run, including the
/// placeholders that have no behaviour yet.
pub const EXIT_FAILURE: i32 = 1;

/// Argument parser for the `stellar-devkit` binary.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(
    name = "stellar-devkit",
    version,
    about = "Developer toolkit for testing and simulating the Stellar fee tracker",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands the devkit exposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Replay a scenario fixture through the sandbox runner.
    Replay,
    /// Convert a recorded fixture into another output format.
    Convert,
    /// Export recorded fee data to a file.
    Export,
    /// Run the devkit benchmark suite.
    Benchmark,
    /// Print the resolved devkit configuration.
    Config,
    /// Print the crate version and build metadata.
    Version,
}

impl Command {
    /// The name the subcommand is invoked by.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Replay => "replay",
            Self::Convert => "convert",
            Self::Export => "export",
            Self::Benchmark => "benchmark",
            Self::Config => "config",
            Self::Version => "version",
        }
    }
}

impl Cli {
    /// Run the selected subcommand, writing its output to `out`, and report the
    /// exit code the binary should terminate with.
    pub fn run<W: Write>(&self, out: &mut W) -> i32 {
        match self.command {
            Command::Version => match version::run(out) {
                Ok(()) => EXIT_SUCCESS,
                Err(error) => {
                    eprintln!("stellar-devkit version: {error}");
                    EXIT_FAILURE
                }
            },
            placeholder => {
                eprintln!("stellar-devkit {}: not implemented yet", placeholder.name());
                EXIT_FAILURE
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(argument: &str) -> Cli {
        Cli::parse_from(["stellar-devkit", argument])
    }

    #[test]
    fn every_subcommand_is_reachable_by_name() {
        let expected = [
            ("replay", Command::Replay),
            ("convert", Command::Convert),
            ("export", Command::Export),
            ("benchmark", Command::Benchmark),
            ("config", Command::Config),
            ("version", Command::Version),
        ];
        for (name, command) in expected {
            assert_eq!(parse(name).command, command);
            assert_eq!(command.name(), name);
        }
    }

    #[test]
    fn an_unknown_subcommand_is_rejected() {
        assert!(Cli::try_parse_from(["stellar-devkit", "meltdown"]).is_err());
    }

    #[test]
    fn a_missing_subcommand_is_rejected() {
        assert!(Cli::try_parse_from(["stellar-devkit"]).is_err());
    }

    #[test]
    fn the_version_subcommand_writes_a_report() {
        let mut out: Vec<u8> = Vec::new();
        assert_eq!(parse("version").run(&mut out), EXIT_SUCCESS);
        let report = String::from_utf8(out).expect("utf-8 report");
        assert!(report.contains(version::CRATE_NAME), "{report}");
        assert!(report.contains(version::CRATE_VERSION), "{report}");
    }

    #[test]
    fn a_placeholder_subcommand_reports_a_failure() {
        for name in ["replay", "convert", "export", "benchmark", "config"] {
            let mut out: Vec<u8> = Vec::new();
            assert_eq!(parse(name).run(&mut out), EXIT_FAILURE, "{name}");
            assert!(out.is_empty(), "{name} wrote output");
        }
    }
}
