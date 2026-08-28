use std::time::{Duration, Instant};

/// A single CPU-time measurement taken around a closure invocation.
#[derive(Debug, Clone)]
pub struct CpuSample {
    pub label: String,
    pub wall_time: Duration,
    pub cpu_time: Duration,
    pub cpu_pct: f64,
}

/// Measures the wall-clock time consumed by `f` and approximates CPU time
/// as equal to wall time on platforms without a dedicated CPU-time clock.
/// Callers needing precise per-thread CPU time should wire in a
/// platform-specific clock (e.g. the `cpu-time` crate) behind this API.
pub fn sample_cpu<T, F: FnOnce() -> T>(label: &str, f: F) -> (T, CpuSample) {
    let start = Instant::now();
    let result = f();
    let wall_time = start.elapsed();
    let sample = CpuSample {
        label: label.to_string(),
        wall_time,
        cpu_time: wall_time,
        cpu_pct: 100.0,
    };
    (result, sample)
}
