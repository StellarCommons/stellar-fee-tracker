//! Stdout sink for the streaming pipeline.
//!
//! [`StdoutSink`] accepts any [`serde::Serialize`] value and prints its JSON
//! representation to stdout, one line per event.  It is intentionally generic
//! so it can be wired up to the concrete `FeeEvent` type once issue #610
//! lands, without any changes to this file.

use crate::error::DevkitError;
use serde::Serialize;
use std::sync::{Arc, Mutex};

/// An in-memory pipeline sink that retains every emitted event.
///
/// Useful as a terminal sink in tests and benchmarks: events are cloned into a
/// thread-safe backing store that can be inspected afterwards. The handle is
/// cheaply cloneable (`Arc`-shared), so producer and assertion sites can hold
/// independent references to the same store.
///
/// # Example
///
/// ```rust
/// use stellar_devkit::streaming::sink::MemorySink;
///
/// let sink: MemorySink<u64> = MemorySink::new();
/// sink.emit(&1).unwrap();
/// sink.emit(&2).unwrap();
/// assert_eq!(sink.len(), 2);
/// assert_eq!(sink.snapshot(), vec![1, 2]);
/// ```
#[derive(Debug, Clone)]
pub struct MemorySink<T> {
    store: Arc<Mutex<Vec<T>>>,
}

impl<T> Default for MemorySink<T> {
    fn default() -> Self {
        Self {
            store: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<T: Clone> MemorySink<T> {
    /// Creates a new, empty in-memory sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a clone of `event`.
    ///
    /// # Errors
    ///
    /// Currently infallible, but returns `Result` for parity with other sinks
    /// so it can be swapped in without changing call sites.
    pub fn emit(&self, event: &T) -> Result<(), DevkitError> {
        self.store
            .lock()
            .map_err(|_| DevkitError::Simulation("memory sink mutex poisoned".into()))?
            .push(event.clone());
        Ok(())
    }

    /// Number of events retained.
    pub fn len(&self) -> usize {
        self.store.lock().map(|s| s.len()).unwrap_or(0)
    }

    /// Whether the sink holds no events.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a snapshot copy of all retained events in emission order.
    pub fn snapshot(&self) -> Vec<T> {
        self.store.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

/// A pipeline sink that serialises each event as JSON and writes it to stdout.
///
/// # Example
///
/// ```rust
/// use stellar_devkit::streaming::sink::StdoutSink;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct MyEvent { value: u64 }
///
/// let sink = StdoutSink::new();
/// sink.emit(&MyEvent { value: 42 }).unwrap();
/// // Prints: {"value":42}
/// ```
#[derive(Debug, Default)]
pub struct StdoutSink;

impl StdoutSink {
    /// Creates a new [`StdoutSink`].
    pub fn new() -> Self {
        Self
    }

    /// Serialises `event` as compact JSON and prints it to stdout followed by
    /// a newline.
    ///
    /// # Errors
    ///
    /// Returns [`DevkitError::Simulation`] if `serde_json` fails to serialise
    /// the value (e.g. a map with non-string keys).
    pub fn emit<T: Serialize>(&self, event: &T) -> Result<(), DevkitError> {
        let json = serde_json::to_string(event).map_err(|e| {
            DevkitError::Simulation(format!("stdout sink serialisation error: {e}"))
        })?;
        println!("{json}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct DummyEvent {
        kind: String,
        value: u64,
    }

    /// Verifies that `emit` succeeds and produces valid JSON for a simple struct.
    #[test]
    fn emit_serialises_struct_without_error() {
        let sink = StdoutSink::new();
        let event = DummyEvent {
            kind: "test".to_string(),
            value: 100,
        };
        // The key invariant: emit must not return an error.
        assert!(sink.emit(&event).is_ok());
    }

    /// Verifies that `emit` succeeds for an enum variant with a payload.
    #[test]
    fn emit_serialises_enum_variant() {
        #[derive(Serialize)]
        enum SimpleEvent {
            LedgerClosed(u64),
            NetworkConditionChanged(String),
        }

        let sink = StdoutSink::new();
        assert!(sink.emit(&SimpleEvent::LedgerClosed(42)).is_ok());
        assert!(sink
            .emit(&SimpleEvent::NetworkConditionChanged(
                "congested".to_string()
            ))
            .is_ok());
    }

    /// Verifies that the serialised output round-trips correctly via
    /// `serde_json::to_string` (used internally by the sink).
    #[test]
    fn serialisation_roundtrip() {
        let event = DummyEvent {
            kind: "spike_detected".to_string(),
            value: 9999,
        };
        let json = serde_json::to_string(&event).unwrap();
        let recovered: DummyEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, recovered);
    }

    /// [`StdoutSink`] is a unit struct; its `Default` is the struct itself.
    #[test]
    fn default_construction() {
        let _sink: StdoutSink = StdoutSink;
    }
}
