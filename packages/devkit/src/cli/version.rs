//! The `version` subcommand: prints the crate version and build metadata.
//!
//! Closes #834.

use std::io::Write;

/// Crate name, taken from the package manifest.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// Crate version, taken from the package manifest.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Width the metadata labels are padded to so the values line up.
const LABEL_WIDTH: usize = 17;

/// Details about the binary that is reporting them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildMetadata {
    /// Operating system the binary was compiled for.
    pub os: &'static str,
    /// Architecture the binary was compiled for.
    pub arch: &'static str,
    /// Cargo profile the binary was compiled with, inferred from whether
    /// debug assertions are enabled.
    pub profile: &'static str,
    /// Whether the running binary has debug assertions enabled.
    pub debug_assertions: bool,
}

impl BuildMetadata {
    /// The metadata describing the binary that is currently running.
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            debug_assertions: cfg!(debug_assertions),
        }
    }
}

/// Render the version report for the running binary.
pub fn report() -> String {
    let metadata = BuildMetadata::current();
    let assertions = if metadata.debug_assertions {
        "on"
    } else {
        "off"
    };
    [
        format!("{CRATE_NAME} {CRATE_VERSION}"),
        format!("{:<LABEL_WIDTH$}{}", "os", metadata.os),
        format!("{:<LABEL_WIDTH$}{}", "arch", metadata.arch),
        format!("{:<LABEL_WIDTH$}{}", "profile", metadata.profile),
        format!("{:<LABEL_WIDTH$}{}", "debug assertions", assertions),
    ]
    .join("\n")
}

/// Write the version report for the running binary to `out`.
pub fn run(out: &mut impl Write) -> std::io::Result<()> {
    writeln!(out, "{}", report())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_leads_with_the_crate_name_and_version() {
        let report = report();
        let mut lines = report.lines();
        assert_eq!(
            lines.next(),
            Some(format!("{CRATE_NAME} {CRATE_VERSION}").as_str())
        );
        assert_eq!(lines.count(), 4, "{report}");
    }

    #[test]
    fn the_report_names_the_target_and_profile() {
        let report = report();
        for label in ["os", "arch", "profile", "debug assertions"] {
            assert!(report.contains(label), "{label} missing from {report}");
        }
        assert!(report.contains(BuildMetadata::current().os), "{report}");
        assert!(report.contains(BuildMetadata::current().arch), "{report}");
    }

    #[test]
    fn the_report_agrees_with_the_manifest() {
        assert_eq!(CRATE_NAME, "stellar-devkit");
        assert_eq!(CRATE_VERSION, crate::DEVKIT_VERSION);
    }

    #[test]
    fn the_profile_follows_the_debug_assertions() {
        let metadata = BuildMetadata::current();
        let expected = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        assert_eq!(metadata.debug_assertions, cfg!(debug_assertions));
        assert_eq!(metadata.profile, expected);
    }

    #[test]
    fn the_target_is_the_one_being_compiled_for() {
        let metadata = BuildMetadata::current();
        assert_eq!(metadata.os, std::env::consts::OS);
        assert_eq!(metadata.arch, std::env::consts::ARCH);
    }

    #[test]
    fn running_terminates_the_report_with_a_newline() {
        let mut out: Vec<u8> = Vec::new();
        run(&mut out).expect("writing to a Vec cannot fail");
        let written = String::from_utf8(out).expect("utf-8 report");
        assert_eq!(written, format!("{}\n", report()));
    }
}
