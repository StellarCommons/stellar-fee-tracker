//! Performance profiling utilities for devkit: CPU/wall-time sampling,
//! memory profiling, and human-readable profiling reports.

pub mod report;
pub mod sampler;

pub use report::{FunctionProfileEntry, ProfilingReport, ProfilingReportBuilder};
pub use sampler::{sample_cpu, sample_mem, CpuSample, MemSample};

pub mod flamegraph;
