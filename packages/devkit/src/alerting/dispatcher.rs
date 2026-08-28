//! Alert dispatchers (#632 stdout, #633 webhook, #634 file).
//!
//! A dispatcher delivers a fired [`AlertEvent`] to a destination. All
//! dispatchers implement the async [`AlertDispatcher`] trait so heterogeneous
//! sinks (including the network-bound webhook) can be held behind one type.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;

use super::rule::{AlertEvent, AlertSeverity};
use crate::error::DevkitError;

/// A destination that alert events are delivered to.
#[async_trait]
pub trait AlertDispatcher: Send + Sync {
    /// Deliver a single event. Implementations should be idempotent-friendly and
    /// return an error only when delivery could not be completed.
    async fn dispatch(&self, event: &AlertEvent) -> Result<(), DevkitError>;
}

fn severity_label(s: AlertSeverity) -> &'static str {
    match s {
        AlertSeverity::Info => "INFO",
        AlertSeverity::Warning => "WARNING",
        AlertSeverity::Critical => "CRITICAL",
    }
}

/// Format a `u64` with thousands separators (e.g. `219192` → `219,192`).
fn group_thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut parts: Vec<String> = digits
        .as_bytes()
        .rchunks(3)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect();
    parts.reverse();
    parts.join(",")
}

/// Human-readable multi-line rendering of an event.
pub fn format_text(event: &AlertEvent) -> String {
    format!(
        "[{}] {}\nRule: {}\nValue: {} stroops (threshold: {})\nTime: {}",
        severity_label(event.severity),
        event.message,
        event.rule_name,
        group_thousands(event.current_value),
        group_thousands(event.threshold),
        event.triggered_at.to_rfc3339(),
    )
}

// ---------------------------------------------------------------------------
// #632 — stdout dispatcher
// ---------------------------------------------------------------------------

/// Prints events to stdout as formatted text or compact JSON.
#[derive(Debug, Clone, Default)]
pub struct StdoutDispatcher {
    json: bool,
}

impl StdoutDispatcher {
    /// Text-mode dispatcher.
    pub fn new() -> Self {
        Self { json: false }
    }

    /// JSON-mode dispatcher.
    pub fn json() -> Self {
        Self { json: true }
    }

    /// Render an event to the configured representation (pure; testable).
    pub fn render(&self, event: &AlertEvent) -> String {
        if self.json {
            serde_json::to_string(event).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
        } else {
            format_text(event)
        }
    }
}

#[async_trait]
impl AlertDispatcher for StdoutDispatcher {
    async fn dispatch(&self, event: &AlertEvent) -> Result<(), DevkitError> {
        println!("{}", self.render(event));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// #633 — webhook dispatcher
// ---------------------------------------------------------------------------

/// POSTs each event as JSON to a configured webhook URL, retrying on failure.
#[derive(Debug, Clone)]
pub struct WebhookDispatcher {
    url: String,
    client: reqwest::Client,
    auth_header: Option<String>,
    max_retries: u32,
}

impl WebhookDispatcher {
    /// Build a dispatcher with a request `timeout` and optional `Authorization`
    /// header value. Retries up to 3 times on delivery failure.
    pub fn new(
        url: impl Into<String>,
        timeout: Duration,
        auth_header: Option<String>,
    ) -> Result<Self, DevkitError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| DevkitError::Protocol(format!("webhook client build failed: {e}")))?;
        Ok(Self {
            url: url.into(),
            client,
            auth_header,
            max_retries: 3,
        })
    }
}

#[async_trait]
impl AlertDispatcher for WebhookDispatcher {
    async fn dispatch(&self, event: &AlertEvent) -> Result<(), DevkitError> {
        let mut last_err = String::new();
        for attempt in 1..=self.max_retries {
            let mut req = self.client.post(&self.url).json(event);
            if let Some(header) = &self.auth_header {
                req = req.header("Authorization", header);
            }
            match req.send().await {
                Ok(resp) if resp.status().is_success() => {
                    eprintln!(
                        "[alerting] webhook delivered {} (attempt {attempt})",
                        event.rule_id
                    );
                    return Ok(());
                }
                Ok(resp) => last_err = format!("HTTP {}", resp.status()),
                Err(e) => last_err = e.to_string(),
            }
            eprintln!(
                "[alerting] webhook delivery failed for {} (attempt {attempt}/{}): {last_err}",
                event.rule_id, self.max_retries
            );
        }
        Err(DevkitError::Protocol(format!(
            "webhook delivery failed after {} attempts: {last_err}",
            self.max_retries
        )))
    }
}

// ---------------------------------------------------------------------------
// #634 — file dispatcher
// ---------------------------------------------------------------------------

/// Appends events to a JSONL file, rotating when the file exceeds `max_bytes`.
#[derive(Debug, Clone)]
pub struct FileDispatcher {
    path: PathBuf,
    max_bytes: u64,
}

impl FileDispatcher {
    /// Create a dispatcher writing to `path`, rotating past `max_bytes`.
    pub fn new(path: impl Into<PathBuf>, max_bytes: u64) -> Self {
        Self {
            path: path.into(),
            max_bytes,
        }
    }

    fn rotate_if_needed(&self) -> Result<(), DevkitError> {
        if self.max_bytes == 0 {
            return Ok(());
        }
        if let Ok(meta) = fs::metadata(&self.path) {
            if meta.len() >= self.max_bytes {
                let rotated = self.path.with_extension(format!(
                    "{}.{}",
                    self.path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("log"),
                    chrono::Utc::now().timestamp_millis()
                ));
                fs::rename(&self.path, rotated)?;
            }
        }
        Ok(())
    }

    /// Append a single JSON line (exposed for testing).
    pub fn append(&self, event: &AlertEvent) -> Result<(), DevkitError> {
        self.rotate_if_needed()?;
        let line = serde_json::to_string(event)
            .map_err(|e| DevkitError::Storage(format!("file dispatcher serialise: {e}")))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }
}

#[async_trait]
impl AlertDispatcher for FileDispatcher {
    async fn dispatch(&self, event: &AlertEvent) -> Result<(), DevkitError> {
        self.append(event)
    }
}
