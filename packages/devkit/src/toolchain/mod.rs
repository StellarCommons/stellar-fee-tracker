//! Developer productivity tools: scenario generation, linting, diffing,
//! and reporting for Stellar fee-tracker test data.
//!
//! Not yet wired into the crate root (`pub mod toolchain;` in `lib.rs`);
//! this is scaffolding for the submodules built out incrementally:
//! `generator`, `linter`, `differ`, `reporter`, `chart`, `anonymiser`,
//! and `scenario_index`.

/// Shared error type for toolchain operations.
#[derive(Debug)]
pub enum ToolchainError {
    InvalidInput(String),
    NotFound(String),
}

impl std::fmt::Display for ToolchainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
        }
    }
}
