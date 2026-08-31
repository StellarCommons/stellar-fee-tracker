//! Alert engine — main loop (#637).
//!
//! [`AlertEngine`] ties the [`RuleEvaluator`], the configured
//! [`AlertDispatcher`]s, the [`AlertHistory`], and the [`AlertRegistry`]
//! together. On each incoming fee snapshot it evaluates the enabled rules,
//! dispatches every triggered [`AlertEvent`], and records them to history.

use super::dispatcher::AlertDispatcher;
use super::evaluator::{FeeSnapshot, RuleEvaluator};
use super::history::AlertHistory;
use super::registry::AlertRegistry;
use super::rule::AlertEvent;

/// The running alerting engine.
#[derive(Default)]
pub struct AlertEngine {
    registry: AlertRegistry,
    evaluator: RuleEvaluator,
    dispatchers: Vec<Box<dyn AlertDispatcher>>,
    history: AlertHistory,
}

impl AlertEngine {
    /// Create an engine with an empty registry, no dispatchers, and a
    /// default-capacity history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an engine pre-loaded with a rule registry.
    pub fn with_registry(registry: AlertRegistry) -> Self {
        Self {
            registry,
            ..Default::default()
        }
    }

    /// Register a dispatcher to receive fired events.
    pub fn add_dispatcher(&mut self, dispatcher: Box<dyn AlertDispatcher>) {
        self.dispatchers.push(dispatcher);
    }

    /// Mutable access to the rule registry for runtime CRUD.
    pub fn registry_mut(&mut self) -> &mut AlertRegistry {
        &mut self.registry
    }

    /// Read access to the rule registry.
    pub fn registry(&self) -> &AlertRegistry {
        &self.registry
    }

    /// Read access to the alert history.
    pub fn history(&self) -> &AlertHistory {
        &self.history
    }

    /// Process one fee snapshot: evaluate enabled rules, dispatch and record
    /// every triggered event, and return the events that fired.
    pub async fn process(&mut self, snapshot: &FeeSnapshot) -> Vec<AlertEvent> {
        let rules = self.registry.enabled_rules();
        let events = self.evaluator.evaluate(&rules, snapshot);
        for event in &events {
            for dispatcher in &self.dispatchers {
                if let Err(err) = dispatcher.dispatch(event).await {
                    eprintln!("[alerting] dispatch error for {}: {err}", event.rule_id);
                }
            }
            self.history.record(event.clone());
        }
        events
    }
}
