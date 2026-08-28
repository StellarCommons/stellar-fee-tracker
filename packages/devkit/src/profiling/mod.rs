//! Performance profiling utilities for devkit: CPU/wall-time sampling,
//! flamegraph generation, and human-readable profiling reports.
//!
//! This module is not yet wired into the crate root (`pub mod profiling;`
//! in `lib.rs`); it is scaffolding for the profiling submodules
//! (`sampler`, `flamegraph`, `report`) to be built out incrementally.

/// Placeholder profiling report shared by the submodules below.
#[derive(Debug, Clone, Default)]
pub struct ProfilingReport {
    pub samples: Vec<String>,
}

impl ProfilingReport {
    pub fn new() -> Self {
        Self { samples: Vec::new() }
    }

    pub fn record(&mut self, label: &str) {
        self.samples.push(label.to_string());
    }
}
