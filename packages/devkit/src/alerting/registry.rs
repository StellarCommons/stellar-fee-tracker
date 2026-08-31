//! Alert rule registry — runtime CRUD (#636).
//!
//! [`AlertRegistry`] holds the active set of [`AlertRule`]s keyed by id and
//! supports add / update / remove / enable / disable at runtime, plus lookups
//! used by the engine to fetch the currently-enabled rules.

use std::collections::HashMap;

use super::rule::AlertRule;

/// In-memory store of alert rules keyed by [`AlertRule::id`].
#[derive(Debug, Default, Clone)]
pub struct AlertRegistry {
    rules: HashMap<String, AlertRule>,
}

impl AlertRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a new rule. Returns `false` (and does not overwrite) if a rule
    /// with the same id already exists — use [`AlertRegistry::update`] for that.
    pub fn add(&mut self, rule: AlertRule) -> bool {
        if self.rules.contains_key(&rule.id) {
            return false;
        }
        self.rules.insert(rule.id.clone(), rule);
        true
    }

    /// Replace an existing rule. Returns `false` if no rule with that id exists.
    pub fn update(&mut self, rule: AlertRule) -> bool {
        if !self.rules.contains_key(&rule.id) {
            return false;
        }
        self.rules.insert(rule.id.clone(), rule);
        true
    }

    /// Insert or replace a rule unconditionally.
    pub fn upsert(&mut self, rule: AlertRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Remove a rule, returning it if present.
    pub fn remove(&mut self, id: &str) -> Option<AlertRule> {
        self.rules.remove(id)
    }

    /// Enable a rule. Returns `false` if the id is unknown.
    pub fn enable(&mut self, id: &str) -> bool {
        self.set_enabled(id, true)
    }

    /// Disable a rule. Returns `false` if the id is unknown.
    pub fn disable(&mut self, id: &str) -> bool {
        self.set_enabled(id, false)
    }

    fn set_enabled(&mut self, id: &str, enabled: bool) -> bool {
        match self.rules.get_mut(id) {
            Some(rule) => {
                rule.enabled = enabled;
                true
            }
            None => false,
        }
    }

    /// Fetch a rule by id.
    pub fn get(&self, id: &str) -> Option<&AlertRule> {
        self.rules.get(id)
    }

    /// All rules (unspecified order).
    pub fn list(&self) -> Vec<AlertRule> {
        self.rules.values().cloned().collect()
    }

    /// Only the currently-enabled rules — the set the engine evaluates.
    pub fn enabled_rules(&self) -> Vec<AlertRule> {
        self.rules.values().filter(|r| r.enabled).cloned().collect()
    }

    /// Number of registered rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the registry holds no rules.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}
