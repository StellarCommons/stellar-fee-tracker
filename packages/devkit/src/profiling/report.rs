use super::sampler::{CpuSample, MemSample};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

/// Summary statistics for a single profiled function/label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionProfileEntry {
    /// Function label or identifier.
    pub function_label: String,
    /// Total number of samples recorded for this function.
    pub sample_count: usize,
    /// Mean CPU / execution duration across all CPU samples.
    pub cpu_mean: Duration,
    /// 95th percentile CPU / execution duration.
    pub cpu_p95: Duration,
    /// Mean peak memory usage in bytes across memory samples.
    pub mem_mean_peak: f64,
    /// Timestamp of the slowest CPU run recorded, if available.
    pub slowest_run_timestamp: Option<DateTime<Utc>>,
}

/// A structured profiling report combining CPU and memory metrics across multiple functions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfilingReport {
    /// Function profile entries sorted by function label.
    pub entries: Vec<FunctionProfileEntry>,
}

impl ProfilingReport {
    /// Create a new empty profiling report.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Create a builder for configuring and assembling a `ProfilingReport`.
    pub fn builder() -> ProfilingReportBuilder {
        ProfilingReportBuilder::new()
    }

    /// Retrieve entry for a specific function label if it exists.
    pub fn get_entry(&self, label: &str) -> Option<&FunctionProfileEntry> {
        self.entries.iter().find(|e| e.function_label == label)
    }

    /// Format the profiling report as an aligned text table.
    pub fn render_table(&self) -> String {
        if self.entries.is_empty() {
            return "No profiling samples recorded.\n".to_string();
        }

        let headers = [
            "Function Label",
            "Samples",
            "CPU Mean",
            "CPU p95",
            "Memory Mean Peak",
            "Slowest Run Timestamp",
        ];

        // Format raw rows for measuring widths
        let rows: Vec<[String; 6]> = self
            .entries
            .iter()
            .map(|e| {
                [
                    e.function_label.clone(),
                    e.sample_count.to_string(),
                    format_duration(e.cpu_mean),
                    format_duration(e.cpu_p95),
                    format_bytes(e.mem_mean_peak),
                    e.slowest_run_timestamp
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_else(|| "N/A".to_string()),
                ]
            })
            .collect();

        // Calculate maximum column widths
        let mut col_widths = [
            headers[0].len(),
            headers[1].len(),
            headers[2].len(),
            headers[3].len(),
            headers[4].len(),
            headers[5].len(),
        ];

        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }

        let make_border = |left: char, mid: char, right: char, fill: char| -> String {
            let mut line = String::new();
            line.push(left);
            for (i, &w) in col_widths.iter().enumerate() {
                line.push_str(&fill.to_string().repeat(w + 2));
                if i < col_widths.len() - 1 {
                    line.push(mid);
                }
            }
            line.push(right);
            line.push('\n');
            line
        };

        let mut out = String::new();
        out.push_str(&make_border('+', '+', '+', '-'));

        // Header row
        out.push('|');
        for (i, header) in headers.iter().enumerate() {
            out.push_str(&format!(" {:<width$} |", header, width = col_widths[i]));
        }
        out.push('\n');
        out.push_str(&make_border('+', '+', '+', '-'));

        // Data rows
        for row in &rows {
            out.push('|');
            // Left-align label and timestamp, right-align numbers
            out.push_str(&format!(" {:<width$} |", row[0], width = col_widths[0]));
            out.push_str(&format!(" {:>width$} |", row[1], width = col_widths[1]));
            out.push_str(&format!(" {:>width$} |", row[2], width = col_widths[2]));
            out.push_str(&format!(" {:>width$} |", row[3], width = col_widths[3]));
            out.push_str(&format!(" {:>width$} |", row[4], width = col_widths[4]));
            out.push_str(&format!(" {:<width$} |", row[5], width = col_widths[5]));
            out.push('\n');
        }

        out.push_str(&make_border('+', '+', '+', '-'));
        out
    }

    /// Print the profiling report as an aligned text table to stdout.
    pub fn print_table(&self) {
        print!("{}", self.render_table());
    }
}

impl fmt::Display for ProfilingReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render_table())
    }
}

/// Builder for combining CPU and memory samples into a structured `ProfilingReport`.
#[derive(Debug, Clone, Default)]
pub struct ProfilingReportBuilder {
    cpu_samples: Vec<CpuSample>,
    mem_samples: Vec<MemSample>,
}

impl ProfilingReportBuilder {
    /// Create a new empty `ProfilingReportBuilder`.
    pub fn new() -> Self {
        Self {
            cpu_samples: Vec::new(),
            mem_samples: Vec::new(),
        }
    }

    /// Add a single `CpuSample` to the builder.
    pub fn add_cpu_sample(&mut self, sample: CpuSample) -> &mut Self {
        self.cpu_samples.push(sample);
        self
    }

    /// Add multiple `CpuSample` instances to the builder.
    pub fn add_cpu_samples(
        &mut self,
        samples: impl IntoIterator<Item = CpuSample>,
    ) -> &mut Self {
        self.cpu_samples.extend(samples);
        self
    }

    /// Add a single `MemSample` to the builder.
    pub fn add_mem_sample(&mut self, sample: MemSample) -> &mut Self {
        self.mem_samples.push(sample);
        self
    }

    /// Add multiple `MemSample` instances to the builder.
    pub fn add_mem_samples(
        &mut self,
        samples: impl IntoIterator<Item = MemSample>,
    ) -> &mut Self {
        self.mem_samples.extend(samples);
        self
    }

    /// Build the structured `ProfilingReport` by aggregating statistics per function label.
    pub fn build(&self) -> ProfilingReport {
        let mut cpu_by_label: BTreeMap<String, Vec<&CpuSample>> = BTreeMap::new();
        let mut mem_by_label: BTreeMap<String, Vec<&MemSample>> = BTreeMap::new();

        for sample in &self.cpu_samples {
            cpu_by_label
                .entry(sample.label.clone())
                .or_default()
                .push(sample);
        }

        for sample in &self.mem_samples {
            mem_by_label
                .entry(sample.label.clone())
                .or_default()
                .push(sample);
        }

        // Collect unique labels in sorted order
        let mut all_labels: Vec<String> = cpu_by_label
            .keys()
            .chain(mem_by_label.keys())
            .cloned()
            .collect();
        all_labels.sort();
        all_labels.dedup();

        let mut entries = Vec::new();

        for label in all_labels {
            let cpus = cpu_by_label.get(&label).cloned().unwrap_or_default();
            let mems = mem_by_label.get(&label).cloned().unwrap_or_default();

            let sample_count = cpus.len().max(mems.len());

            // Compute CPU mean and p95
            let (cpu_mean, cpu_p95, slowest_timestamp) = if !cpus.is_empty() {
                let total_nanos: u128 = cpus.iter().map(|s| s.wall_time.as_nanos()).sum();
                let mean_nanos = (total_nanos / cpus.len() as u128) as u64;
                let mean = Duration::from_nanos(mean_nanos);

                let mut sorted_durations: Vec<Duration> =
                    cpus.iter().map(|s| s.wall_time).collect();
                sorted_durations.sort();

                let p95_idx = ((0.95 * sorted_durations.len() as f64).ceil() as usize)
                    .saturating_sub(1)
                    .min(sorted_durations.len() - 1);
                let p95 = sorted_durations[p95_idx];

                let slowest_sample = cpus.iter().max_by_key(|s| s.wall_time);
                let slowest_time = slowest_sample.map(|s| s.timestamp);

                (mean, p95, slowest_time)
            } else {
                (Duration::ZERO, Duration::ZERO, None)
            };

            // Compute memory mean peak
            let mem_mean_peak = if !mems.is_empty() {
                let total_peak: u64 = mems.iter().map(|s| s.peak_bytes).sum();
                total_peak as f64 / mems.len() as f64
            } else {
                0.0
            };

            entries.push(FunctionProfileEntry {
                function_label: label,
                sample_count,
                cpu_mean,
                cpu_p95,
                mem_mean_peak,
                slowest_run_timestamp: slowest_timestamp,
            });
        }

        ProfilingReport { entries }
    }
}

/// Helper function to format duration in human-readable units.
fn format_duration(d: Duration) -> String {
    let nanos = d.as_nanos();
    if nanos == 0 {
        return "0.00ms".to_string();
    }
    if nanos < 1_000 {
        format!("{nanos}ns")
    } else if nanos < 1_000_000 {
        format!("{:.2}µs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", nanos as f64 / 1_000_000_000.0)
    }
}

/// Helper function to format byte counts in human-readable units.
fn format_bytes(bytes: f64) -> String {
    if bytes <= 0.0 {
        return "0 B".to_string();
    }
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes < KB {
        format!("{:.0} B", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes / KB)
    } else if bytes < GB {
        format!("{:.2} MB", bytes / MB)
    } else {
        format!("{:.2} GB", bytes / GB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_builder_empty() {
        let report = ProfilingReportBuilder::new().build();
        assert!(report.entries.is_empty());
        assert_eq!(report.render_table(), "No profiling samples recorded.\n");
    }

    #[test]
    fn test_report_builder_combined_samples() {
        let mut builder = ProfilingReportBuilder::new();

        let t1 = Utc::now();
        builder.add_cpu_sample(
            CpuSample::new("fee_estimator", Duration::from_millis(10), Duration::from_millis(10), 100.0)
                .with_timestamp(t1),
        );
        builder.add_cpu_sample(
            CpuSample::new("fee_estimator", Duration::from_millis(20), Duration::from_millis(20), 100.0)
                .with_timestamp(t1),
        );
        builder.add_mem_sample(MemSample::new("fee_estimator", 2048, 1024).with_timestamp(t1));
        builder.add_mem_sample(MemSample::new("fee_estimator", 4096, 2048).with_timestamp(t1));

        let report = builder.build();
        assert_eq!(report.entries.len(), 1);

        let entry = &report.entries[0];
        assert_eq!(entry.function_label, "fee_estimator");
        assert_eq!(entry.sample_count, 2);
        assert_eq!(entry.cpu_mean, Duration::from_millis(15));
        assert_eq!(entry.cpu_p95, Duration::from_millis(20));
        assert_eq!(entry.mem_mean_peak, 3072.0);
        assert_eq!(entry.slowest_run_timestamp, Some(t1));

        let table = report.render_table();
        assert!(table.contains("fee_estimator"));
        assert!(table.contains("15.00ms"));
        assert!(table.contains("20.00ms"));
    }
}
