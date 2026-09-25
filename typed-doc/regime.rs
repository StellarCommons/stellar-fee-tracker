use std::time::{Duration, Instant};

/// A single benchmark measurement.
#[derive(Debug, Clone)]
pub struct Measurement {
    /// Name of the benchmark.
    pub name: String,
    /// Number of iterations.
    pub iterations: u64,
    /// Total elapsed time.
    pub elapsed: Duration,
    /// Optional throughput (operations per second).
    pub throughput: Option<f64>,
    /// Optional memory delta in bytes.
    pub memory_bytes: Option<u64>,
}

impl Measurement {
    /// Average time per iteration in nanoseconds.
    pub fn avg_ns(&self) -> f64 {
        self.elapsed.as_nanos() as f64 / self.iterations as f64
    }

    /// Average time per iteration in microseconds.
    pub fn avg_us(&self) -> f64 {
        self.avg_ns() / 1000.0
    }

    /// Average time per iteration in milliseconds.
    pub fn avg_ms(&self) -> f64 {
        self.avg_us() / 1000.0
    }

    /// Format the measurement as a human-readable string.
    pub fn display(&self) -> String {
        let avg = if self.avg_ms() >= 1.0 {
            format!("{:.2} ms", self.avg_ms())
        } else if self.avg_us() >= 1.0 {
            format!("{:.2} µs", self.avg_us())
        } else {
            format!("{:.0} ns", self.avg_ns())
        };
        let throughput = self
            .throughput
            .map(|t| format!(", {:.0} ops/s", t))
            .unwrap_or_default();
        let mem = self
            .memory_bytes
            .map(|b| format!(", {} bytes", b))
            .unwrap_or_default();
        format!("{}: {} iterations in {:?}{}{}", self.name, self.iterations, self.elapsed, avg, throughput, mem)
    }

    /// Format as a JSON line.
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"name":"{}","iterations":{},"elapsed_ns":{},"avg_ns":{:.2},"throughput":{},"memory_bytes":{}}}"#,
            self.name,
            self.iterations,
            self.elapsed.as_nanos(),
            self.avg_ns(),
            self.throughput.map(|t| t.to_string()).unwrap_or_else(|| "null".into()),
            self.memory_bytes.map(|b| b.to_string()).unwrap_or_else(|| "null".into()),
        )
    }
}

/// A benchmark suite that measures monitoring overhead.
pub struct MonitoringBenchmark {
    results: Vec<Measurement>,
}

impl MonitoringBenchmark {
    pub fn new() -> Self {
        Self { results: Vec::new() }
    }

    /// Run a benchmark measuring trace context creation overhead.
    pub fn bench_trace_creation(&mut self, iterations: u64) {
        let start = Instant::now();
        for _ in 0..iterations {
            let _ctx = super::TraceContext::new_root();
        }
        let elapsed = start.elapsed();
        self.results.push(Measurement {
            name: "trace_creation".into(),
            iterations,
            elapsed,
            throughput: Some(iterations as f64 / elapsed.as_secs_f64()),
            memory_bytes: None,
        });
    }

    /// Run a benchmark measuring trace context propagation overhead.
    pub fn bench_trace_propagation(&mut self, iterations: u64) {
        let propagator = super::W3CPropagator;
        let ctx = super::TraceContext::new_root();
        let mut carrier = std::collections::HashMap::new();
        let start = Instant::now();
        for _ in 0..iterations {
            propagator.inject(&ctx, &mut carrier);
            let _extracted = propagator.extract(&carrier);
            carrier.clear();
        }
        let elapsed = start.elapsed();
        self.results.push(Measurement {
            name: "trace_propagation".into(),
            iterations,
            elapsed,
            throughput: Some(iterations as f64 / elapsed.as_secs_f64()),
            memory_bytes: None,
        });
    }

    /// Run a benchmark measuring baggage operations.
    pub fn bench_baggage_ops(&mut self, iterations: u64) {
        let start = Instant::now();
        for i in 0..iterations {
            let ctx = super::TraceContext::new_root()
                .with_baggage("key", &format!("value{}", i));
            let _val = ctx.baggage("key");
        }
        let elapsed = start.elapsed();
        self.results.push(Measurement {
            name: "baggage_ops".into(),
            iterations,
            elapsed,
            throughput: Some(iterations as f64 / elapsed.as_secs_f64()),
            memory_bytes: None,
        });
    }

    /// Run a benchmark measuring context propagation cost.
    pub fn bench_span_creation(&mut self, iterations: u64) {
        let root = super::TraceContext::new_root();
        let start = Instant::now();
        for _ in 0..iterations {
            let _child = root.child_span();
        }
        let elapsed = start.elapsed();
        self.results.push(Measurement {
            name: "span_creation".into(),
            iterations,
            elapsed,
            throughput: Some(iterations as f64 / elapsed.as_secs_f64()),
            memory_bytes: None,
        });
    }

    /// Run all standard benchmarks with a given iteration count.
    pub fn run_all(&mut self, iterations: u64) {
        self.bench_trace_creation(iterations);
        self.bench_trace_propagation(iterations);
        self.bench_baggage_ops(iterations);
        self.bench_span_creation(iterations);
    }

    /// Display all benchmark results.
    pub fn report(&self) -> String {
        let mut out = String::from("Monitoring Overhead Benchmark\n");
        out.push_str("==============================\n");
        for m in &self.results {
            out.push_str(&format!("  {}\n", m.display()));
        }
        out
    }

    /// Export all results as a JSON array.
    pub fn to_json(&self) -> String {
        let items: Vec<String> = self.results.iter().map(|m| m.to_json()).collect();
        format!("[{}]", items.join(","))
    }

    /// Clear all results.
    pub fn reset(&mut self) {
        self.results.clear();
    }

    /// Return the worst-case average latency across all benchmarks.
    pub fn worst_avg_ns(&self) -> f64 {
        self.results.iter().map(|m| m.avg_ns()).fold(0.0_f64, f64::max)
    }

    /// Return the total elapsed time across all benchmarks.
    pub fn total_elapsed(&self) -> Duration {
        self.results.iter().fold(Duration::ZERO, |acc, m| acc + m.elapsed)
    }
}

/// Arguments for the benchmark subcommand.
pub struct BenchmarkOverheadArgs {
    /// Number of iterations per benchmark.
    pub iterations: u64,
    /// Output as JSON.
    pub json: bool,
}

impl Default for BenchmarkOverheadArgs {
    fn default() -> Self {
        Self {
            iterations: 10_000,
            json: false,
        }
    }
}

impl BenchmarkOverheadArgs {
    /// Run the benchmark.
    pub fn run(&self) {
        let mut bench = MonitoringBenchmark::new();
        bench.run_all(self.iterations);
        if self.json {
            println!("{}", bench.to_json());
        } else {
            println!("{}", bench.report());
        }
    }

    /// Suggest an iteration count based on desired precision.
    pub fn suggest_iterations(precision_pct: f64) -> u64 {
        (100.0 / precision_pct).powi(2) as u64
    }

    /// Format a duration in a human-friendly way.
    pub fn format_duration(d: &Duration) -> String {
        let total_ns = d.as_nanos();
        if total_ns >= 1_000_000_000 {
            format!("{:.2}s", d.as_secs_f64())
        } else if total_ns >= 1_000_000 {
            format!("{:.2}ms", d.as_secs_f64() * 1000.0)
        } else if total_ns >= 1_000 {
            format!("{:.2}µs", d.as_secs_f64() * 1_000_000.0)
        } else {
            format!("{}ns", total_ns)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_avg_ns_is_correct() {
        let m = Measurement {
            name: "test".into(),
            iterations: 10,
            elapsed: Duration::from_nanos(1000),
            throughput: None,
            memory_bytes: None,
        };
        assert!((m.avg_ns() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn benchmark_runs_without_panic() {
        let mut bench = MonitoringBenchmark::new();
        bench.run_all(100);
        assert_eq!(bench.results.len(), 4);
    }

    #[test]
    fn benchmark_report_contains_names() {
        let mut bench = MonitoringBenchmark::new();
        bench.run_all(100);
        let report = bench.report();
        assert!(report.contains("trace_creation"));
        assert!(report.contains("span_creation"));
    }

    #[test]
    fn benchmark_json_is_array() {
        let mut bench = MonitoringBenchmark::new();
        bench.run_all(100);
        let json = bench.to_json();
        assert!(json.starts_with('['));
        assert!(json.contains("trace_creation"));
    }

    #[test]
    fn reset_clears_results() {
        let mut bench = MonitoringBenchmark::new();
        bench.run_all(100);
        bench.reset();
        assert!(bench.results.is_empty());
    }

    #[test]
    fn worst_avg_ns_returns_max() {
        let mut bench = MonitoringBenchmark::new();
        bench.run_all(100);
        assert!(bench.worst_avg_ns() > 0.0);
    }

    #[test]
    fn suggest_iterations_scales_with_precision() {
        let low = BenchmarkOverheadArgs::suggest_iterations(10.0);
        let high = BenchmarkOverheadArgs::suggest_iterations(1.0);
        assert!(high > low);
    }

    #[test]
    fn format_duration_handles_ns() {
        let s = BenchmarkOverheadArgs::format_duration(&Duration::from_nanos(500));
        assert!(s.contains("ns"));
    }

    #[test]
    fn benchmark_args_default() {
        let args = BenchmarkOverheadArgs::default();
        assert_eq!(args.iterations, 10_000);
        assert!(!args.json);
    }

    #[test]
    fn measurement_json_includes_fields() {
        let m = Measurement {
            name: "test".into(),
            iterations: 100,
            elapsed: Duration::from_micros(500),
            throughput: Some(200_000.0),
            memory_bytes: Some(1024),
        };
        let json = m.to_json();
        assert!(json.contains("throughput"));
        assert!(json.contains("memory_bytes"));
    }
}


/// Documentation metadata for the monitoring module.
///
/// This module provides observability utilities including:
/// - Trace context propagation across module boundaries
/// - Benchmarking infrastructure for measuring overhead
/// - Log rotation with configurable size and retention policies
///
/// # Module Structure
///
/// ```text
/// src/monitoring/
/// ├── mod.rs              -- Re-exports + module constants
/// ├── trace_context.rs    -- TraceId, SpanId, TraceContext, TraceRegistry
/// ├── benchmark.rs        -- Measurement, MonitoringBenchmark
/// └── log_rotation.rs     -- LogRotationConfig, RotatingLogWriter
/// ```

/// Current version of the monitoring module API.
pub const MONITORING_VERSION: &str = "1.0.0";

/// Default sampling rate for trace spans (1.0 = sample all).
pub const DEFAULT_SAMPLING_RATE: f64 = 1.0;

/// Maximum baggage key-value pairs per trace context.
pub const MAX_BAGGAGE_ENTRIES: usize = 64;

/// Maximum traceparent header length (W3C spec).
pub const TRACEPARENT_MAX_LENGTH: usize = 55;

/// Default log file size before rotation (10 MB).
pub const DEFAULT_LOG_MAX_SIZE: u64 = 10 * 1024 * 1024;

/// Default number of archived log files to retain.
pub const DEFAULT_LOG_MAX_FILES: u32 = 5;

/// Default log output directory.
pub const DEFAULT_LOG_DIR: &str = "logs";

/// Known trace propagation formats supported by the module.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropagationFormat {
    W3CTraceparent,
    ZipkinB3,
    Jaeger,
    Datadog,
}

impl PropagationFormat {
    /// Return the HTTP header name used by this format.
    pub fn header_name(&self) -> &'static str {
        match self {
            Self::W3CTraceparent => "traceparent",
            Self::ZipkinB3 => "b3",
            Self::Jaeger => "uber-trace-id",
            Self::Datadog => "x-datadog-trace-id",
        }
    }

    /// Return all supported formats.
    pub fn all() -> &'static [PropagationFormat] {
        &[
            Self::W3CTraceparent,
            Self::ZipkinB3,
            Self::Jaeger,
            Self::Datadog,
        ]
    }
}

/// Severity levels for monitoring events.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum MonitoringLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl MonitoringLevel {
    /// Parse a level from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" | "warning" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// Return the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

/// A metric label for tagging monitoring data.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MetricLabel {
    pub key: String,
    pub value: String,
}

impl MetricLabel {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

/// Monitoring configuration that can be serialized/deserialized.
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Whether trace propagation is enabled.
    pub trace_enabled: bool,
    /// Whether benchmark collection is enabled.
    pub benchmark_enabled: bool,
    /// Whether log rotation is enabled.
    pub log_rotation_enabled: bool,
    /// Sampling rate for trace spans [0.0, 1.0].
    pub sampling_rate: f64,
    /// Active propagation format.
    pub propagation_format: PropagationFormat,
    /// Active monitoring level.
    pub level: MonitoringLevel,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            trace_enabled: true,
            benchmark_enabled: false,
            log_rotation_enabled: true,
            sampling_rate: DEFAULT_SAMPLING_RATE,
            propagation_format: PropagationFormat::W3CTraceparent,
            level: MonitoringLevel::Info,
        }
    }
}

impl MonitoringConfig {
    /// Validate the configuration.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if !(0.0..=1.0).contains(&self.sampling_rate) {
            issues.push("sampling_rate must be between 0.0 and 1.0".into());
        }
        issues
    }

    /// Display the configuration as a formatted string.
    pub fn display(&self) -> String {
        format!(
            "Monitoring Configuration\n\
             ========================\n\
             trace:              {}\n\
             benchmark:          {}\n\
             log_rotation:       {}\n\
             sampling_rate:      {:.2}\n\
             propagation_format: {:?}\n\
             level:              {}\n",
            if self.trace_enabled { "enabled" } else { "disabled" },
            if self.benchmark_enabled { "enabled" } else { "disabled" },
            if self.log_rotation_enabled { "enabled" } else { "disabled" },
            self.sampling_rate,
            self.propagation_format,
            self.level.as_str(),
        )
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"trace":{},"benchmark":{},"log_rotation":{},"sampling_rate":{},"format":"{:?}","level":"{}"}}"#,
            self.trace_enabled,
            self.benchmark_enabled,
            self.log_rotation_enabled,
            self.sampling_rate,
            self.propagation_format,
            self.level.as_str(),
        )
    }
}

/// Generate a summary of the monitoring module's public API.
pub fn module_summary() -> Vec<(String, String)> {
    vec![
        ("TraceContext".into(), "W3C trace context with baggage propagation".into()),
        ("TraceRegistry".into(), "Thread-safe trace context store".into()),
        ("W3CPropagator".into(), "Inject/extract traceparent headers".into()),
        ("MonitoringBenchmark".into(), "Overhead measurement suite".into()),
        ("Measurement".into(), "Single benchmark result".into()),
        ("LogRotationConfig".into(), "Log file rotation settings".into()),
        ("RotatingLogWriter".into(), "Auto-rotating log file writer".into()),
        ("MonitoringConfig".into(), "Global monitoring settings".into()),
    ]
}

/// Arguments for the monitoring documentation subcommand.
pub struct MonitoringDocsArgs {
    /// Show the module summary.
    pub summary: bool,
    /// Show configuration documentation.
    pub config: bool,
    /// Output as JSON.
    pub json: bool,
}

impl Default for MonitoringDocsArgs {
    fn default() -> Self {
        Self {
            summary: true,
            config: false,
            json: false,
        }
    }
}

impl MonitoringDocsArgs {
    /// Run the monitoring docs subcommand.
    pub fn run(&self) {
        if self.summary {
            let items = module_summary();
            if self.json {
                let pairs: Vec<String> = items
                    .iter()
                    .map(|(n, d)| format!(r#"{{"name":"{}","desc":"{}"}}"#, n, d))
                    .collect();
                println!("[{}]", pairs.join(","));
            } else {
                println!("Monitoring Module — Public API");
                println!("===============================");
                for (name, desc) in &items {
                    println!("  {:<25} {}", name, desc);
                }
            }
        }
        if self.config {
            let cfg = MonitoringConfig::default();
            if self.json {
                println!("{}", cfg.to_json());
            } else {
                println!("\n{}", cfg.display());
            }
        }
    }

    /// Generate markdown documentation for the monitoring module.
    pub fn generate_docs() -> String {
        let mut docs = String::new();
        docs.push_str("# Monitoring Module\n\n");
        docs.push_str("## Overview\n\n");
        docs.push_str("The monitoring module provides distributed tracing, benchmarking, and log rotation for the devkit.\n\n");
        docs.push_str("## Components\n\n");
        docs.push_str("| Component | Description |\n");
        docs.push_str("|-----------|-------------|\n");
        for (name, desc) in module_summary() {
            docs.push_str(&format!("| `{}` | {} |\n", name, desc));
        }
        docs.push_str("\n## Configuration\n\n");
        docs.push_str("```rust\n");
        docs.push_str("MonitoringConfig {\n");
        docs.push_str("    trace_enabled: true,\n");
        docs.push_str("    benchmark_enabled: false,\n");
        docs.push_str("    log_rotation_enabled: true,\n");
        docs.push_str("    sampling_rate: 1.0,\n");
        docs.push_str("    propagation_format: W3CTraceparent,\n");
        docs.push_str("    level: Info,\n");
        docs.push_str("}\n```\n");
        docs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitoring_version_defined() {
        assert!(!MONITORING_VERSION.is_empty());
    }

    #[test]
    fn propagation_format_header_name() {
        assert_eq!(PropagationFormat::W3CTraceparent.header_name(), "traceparent");
        assert_eq!(PropagationFormat::ZipkinB3.header_name(), "b3");
    }

    #[test]
    fn propagation_format_all_returns_four() {
        assert_eq!(PropagationFormat::all().len(), 4);
    }

    #[test]
    fn monitoring_level_parse() {
        assert_eq!(MonitoringLevel::parse("info"), Some(MonitoringLevel::Info));
        assert_eq!(MonitoringLevel::parse("WARN"), Some(MonitoringLevel::Warn));
        assert_eq!(MonitoringLevel::parse("unknown"), None);
    }

    #[test]
    fn monitoring_level_as_str() {
        assert_eq!(MonitoringLevel::Debug.as_str(), "debug");
        assert_eq!(MonitoringLevel::Error.as_str(), "error");
    }

    #[test]
    fn config_validation_passes() {
        let cfg = MonitoringConfig::default();
        assert!(cfg.validate().is_empty());
    }

    #[test]
    fn config_validation_fails_for_bad_rate() {
        let cfg = MonitoringConfig {
            sampling_rate: 1.5,
            ..Default::default()
        };
        assert!(!cfg.validate().is_empty());
    }

    #[test]
    fn config_display_contains_fields() {
        let out = MonitoringConfig::default().display();
        assert!(out.contains("trace"));
        assert!(out.contains("benchmark"));
    }

    #[test]
    fn config_json_is_valid() {
        let json = MonitoringConfig::default().to_json();
        assert!(json.contains("sampling_rate"));
    }

    #[test]
    fn module_summary_returns_items() {
        let items = module_summary();
        assert!(items.len() >= 7);
        assert!(items.iter().any(|(n, _)| n == "TraceContext"));
    }

    #[test]
    fn generate_docs_contains_overview() {
        let docs = MonitoringDocsArgs::generate_docs();
        assert!(docs.contains("Monitoring Module"));
        assert!(docs.contains("TraceContext"));
        assert!(docs.contains("RotatingLogWriter"));
    }

    #[test]
    fn metric_label_new() {
        let label = MetricLabel::new("env", "prod");
        assert_eq!(label.key, "env");
        assert_eq!(label.value, "prod");
    }

    #[test]
    fn monitoring_docs_args_default() {
        let args = MonitoringDocsArgs::default();
        assert!(args.summary);
        assert!(!args.config);
    }

    #[test]
    fn max_baggage_entries_constant() {
        assert_eq!(MAX_BAGGAGE_ENTRIES, 64);
    }
}
