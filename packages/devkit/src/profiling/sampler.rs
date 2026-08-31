use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// A single CPU-time measurement taken around a closure invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSample {
    pub label: String,
    pub wall_time: Duration,
    pub cpu_time: Duration,
    pub cpu_pct: f64,
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
}

impl CpuSample {
    pub fn new(
        label: impl Into<String>,
        wall_time: Duration,
        cpu_time: Duration,
        cpu_pct: f64,
    ) -> Self {
        Self {
            label: label.into(),
            wall_time,
            cpu_time,
            cpu_pct,
            timestamp: Utc::now(),
        }
    }

    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// A single memory measurement taken around a closure invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemSample {
    pub label: String,
    pub peak_bytes: u64,
    pub allocated_bytes: u64,
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
}

impl MemSample {
    pub fn new(label: impl Into<String>, peak_bytes: u64, allocated_bytes: u64) -> Self {
        Self {
            label: label.into(),
            peak_bytes,
            allocated_bytes,
            timestamp: Utc::now(),
        }
    }

    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Measures the wall-clock time consumed by `f` and approximates CPU time
/// as equal to wall time on platforms without a dedicated CPU-time clock.
/// Callers needing precise per-thread CPU time should wire in a
/// platform-specific clock (e.g. the `cpu-time` crate) behind this API.
pub fn sample_cpu<T, F: FnOnce() -> T>(label: &str, f: F) -> (T, CpuSample) {
    let start_time = Utc::now();
    let start = Instant::now();
    let result = f();
    let wall_time = start.elapsed();
    let sample = CpuSample {
        label: label.to_string(),
        wall_time,
        cpu_time: wall_time,
        cpu_pct: 100.0,
        timestamp: start_time,
    };
    (result, sample)
}

/// Measures approximate memory usage for closure `f`.
pub fn sample_mem<T, F: FnOnce() -> T>(
    label: &str,
    peak_bytes: u64,
    allocated_bytes: u64,
    f: F,
) -> (T, MemSample) {
    let start_time = Utc::now();
    let result = f();
    let sample = MemSample {
        label: label.to_string(),
        peak_bytes,
        allocated_bytes,
        timestamp: start_time,
    };
    (result, sample)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sample_cpu_custom_closure() {
        let (res, sample) = sample_cpu("custom_closure", || {
            let mut sum = 0;
            for i in 0..1000 { sum += i; }
            sum
        });
        assert_eq!(res, 499500);
        assert_eq!(sample.label, "custom_closure");
    }
}
