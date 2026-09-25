//! Fixed-size sliding window of recent fee values.
//!
//! Closes #769.

use std::collections::VecDeque;

pub struct RollingWindow {
    capacity: usize,
    values: VecDeque<f64>,
}

impl RollingWindow {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, values: VecDeque::with_capacity(capacity) }
    }

    pub fn push(&mut self, value: f64) {
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn snapshot(&self) -> Vec<f64> {
        self.values.iter().copied().collect()
    }
}
