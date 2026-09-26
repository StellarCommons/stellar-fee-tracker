use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Copy)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub cooldown: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            cooldown: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateTransition {
    pub from: CircuitState,
    pub to: CircuitState,
    pub at: Instant,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    consecutive_failures: u32,
    consecutive_successes: u32,
    opened_at: Option<Instant>,
    last_transition: Option<StateTransition>,
    rejected_calls: u64,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config: CircuitBreakerConfig {
                failure_threshold: config.failure_threshold.max(1),
                success_threshold: config.success_threshold.max(1),
                cooldown: config.cooldown,
            },
            state: CircuitState::Closed,
            consecutive_failures: 0,
            consecutive_successes: 0,
            opened_at: None,
            last_transition: None,
            rejected_calls: 0,
        }
    }

    pub fn config(&self) -> &CircuitBreakerConfig {
        &self.config
    }

    pub fn state(&self) -> CircuitState {
        self.state
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    pub fn consecutive_successes(&self) -> u32 {
        self.consecutive_successes
    }

    pub fn rejected_calls(&self) -> u64 {
        self.rejected_calls
    }

    pub fn last_transition(&self) -> Option<StateTransition> {
        self.last_transition
    }

    pub fn is_call_permitted(&mut self) -> bool {
        self.is_call_permitted_at(Instant::now())
    }

    pub fn is_call_permitted_at(&mut self, now: Instant) -> bool {
        match self.state {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => {
                if self.cooldown_elapsed_at(now) {
                    self.transition_to(CircuitState::HalfOpen, now);
                    true
                } else {
                    self.rejected_calls += 1;
                    false
                }
            }
        }
    }

    pub fn record_success(&mut self) {
        self.record_success_at(Instant::now());
    }

    pub fn record_success_at(&mut self, now: Instant) {
        match self.state {
            CircuitState::HalfOpen => {
                self.consecutive_successes += 1;
                if self.consecutive_successes >= self.config.success_threshold {
                    self.consecutive_failures = 0;
                    self.consecutive_successes = 0;
                    self.opened_at = None;
                    self.transition_to(CircuitState::Closed, now);
                }
            }
            CircuitState::Closed => {
                self.consecutive_failures = 0;
            }
            CircuitState::Open => {}
        }
    }

    pub fn record_failure(&mut self) {
        self.record_failure_at(Instant::now());
    }

    pub fn record_failure_at(&mut self, now: Instant) {
        match self.state {
            CircuitState::Open => {}
            CircuitState::Closed | CircuitState::HalfOpen => {
                self.consecutive_failures += 1;
                self.consecutive_successes = 0;
                let should_open = self.state == CircuitState::HalfOpen
                    || self.consecutive_failures >= self.config.failure_threshold;
                if should_open {
                    self.opened_at = Some(now);
                    self.consecutive_successes = 0;
                    self.transition_to(CircuitState::Open, now);
                }
            }
        }
    }

    pub fn force_open(&mut self, now: Instant) {
        self.opened_at = Some(now);
        self.consecutive_failures = self.config.failure_threshold;
        self.consecutive_successes = 0;
        self.transition_to(CircuitState::Open, now);
    }

    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.consecutive_failures = 0;
        self.consecutive_successes = 0;
        self.opened_at = None;
        self.last_transition = None;
    }

    pub fn cooldown_elapsed_at(&self, now: Instant) -> bool {
        match self.opened_at {
            Some(opened_at) => now.saturating_duration_since(opened_at) >= self.config.cooldown,
            None => true,
        }
    }

    fn transition_to(&mut self, to: CircuitState, now: Instant) {
        let from = self.state;
        self.state = to;
        self.last_transition = Some(StateTransition { from, to, at: now });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(
        failure_threshold: u32,
        success_threshold: u32,
        cooldown_ms: u64,
    ) -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold,
            success_threshold,
            cooldown: Duration::from_millis(cooldown_ms),
        }
    }

    fn breaker() -> CircuitBreaker {
        CircuitBreaker::new(config(3, 2, 1_000))
    }

    #[test]
    fn a_new_breaker_starts_closed() {
        let breaker = breaker();
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert_eq!(breaker.consecutive_failures(), 0);
        assert!(breaker.last_transition().is_none());
    }

    #[test]
    fn a_closed_breaker_permits_calls() {
        let mut breaker = breaker();
        assert!(breaker.is_call_permitted());
        assert_eq!(breaker.rejected_calls(), 0);
    }

    #[test]
    fn failures_below_the_threshold_keep_it_closed() {
        let mut breaker = breaker();
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert_eq!(breaker.consecutive_failures(), 2);
    }

    #[test]
    fn reaching_the_failure_threshold_opens_it() {
        let mut breaker = breaker();
        for _ in 0..3 {
            breaker.record_failure();
        }
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn a_success_resets_the_failure_streak() {
        let mut breaker = breaker();
        breaker.record_failure();
        breaker.record_failure();
        breaker.record_success();
        assert_eq!(breaker.consecutive_failures(), 0);
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn an_open_breaker_rejects_calls() {
        let mut breaker = breaker();
        for _ in 0..3 {
            breaker.record_failure();
        }
        let start = Instant::now();
        assert!(!breaker.is_call_permitted_at(start));
        assert_eq!(breaker.rejected_calls(), 1);
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn the_cooldown_moves_it_to_half_open() {
        let mut breaker = breaker();
        let start = Instant::now();
        for _ in 0..3 {
            breaker.record_failure_at(start);
        }
        assert!(!breaker.is_call_permitted_at(start + Duration::from_millis(999)));
        assert!(breaker.is_call_permitted_at(start + Duration::from_millis(1_000)));
        assert_eq!(breaker.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn half_open_closes_after_enough_successes() {
        let mut breaker = breaker();
        let start = Instant::now();
        for _ in 0..3 {
            breaker.record_failure_at(start);
        }
        assert!(breaker.is_call_permitted_at(start + Duration::from_millis(1_000)));
        breaker.record_success_at(start + Duration::from_millis(1_100));
        assert_eq!(breaker.state(), CircuitState::HalfOpen);
        breaker.record_success_at(start + Duration::from_millis(1_200));
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn a_single_half_open_failure_reopens_it() {
        let mut breaker = breaker();
        let start = Instant::now();
        for _ in 0..3 {
            breaker.record_failure_at(start);
        }
        assert!(breaker.is_call_permitted_at(start + Duration::from_millis(1_000)));
        breaker.record_failure_at(start + Duration::from_millis(1_100));
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn the_cooldown_restarts_after_reopening() {
        let mut breaker = breaker();
        let start = Instant::now();
        for _ in 0..3 {
            breaker.record_failure_at(start);
        }
        assert!(breaker.is_call_permitted_at(start + Duration::from_millis(1_000)));
        breaker.record_failure_at(start + Duration::from_millis(1_100));
        assert!(!breaker.is_call_permitted_at(start + Duration::from_millis(2_099)));
        assert!(breaker.is_call_permitted_at(start + Duration::from_millis(2_100)));
    }

    #[test]
    fn transitions_are_recorded() {
        let mut breaker = breaker();
        let start = Instant::now();
        for _ in 0..3 {
            breaker.record_failure_at(start);
        }
        let transition = breaker.last_transition().expect("transition");
        assert_eq!(transition.from, CircuitState::Closed);
        assert_eq!(transition.to, CircuitState::Open);
    }

    #[test]
    fn forcing_open_short_circuits_the_threshold() {
        let mut breaker = breaker();
        let start = Instant::now();
        breaker.force_open(start);
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.is_call_permitted_at(start));
    }

    #[test]
    fn resetting_returns_it_to_closed() {
        let mut breaker = breaker();
        for _ in 0..3 {
            breaker.record_failure();
        }
        breaker.reset();
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert_eq!(breaker.consecutive_failures(), 0);
        assert!(breaker.last_transition().is_none());
    }

    #[test]
    fn a_zero_threshold_is_clamped() {
        let breaker = CircuitBreaker::new(config(0, 0, 1_000));
        assert_eq!(breaker.config().failure_threshold, 1);
        assert_eq!(breaker.config().success_threshold, 1);
    }

    #[test]
    fn failures_while_open_are_ignored() {
        let mut breaker = breaker();
        let start = Instant::now();
        for _ in 0..3 {
            breaker.record_failure_at(start);
        }
        let failures = breaker.consecutive_failures();
        breaker.record_failure_at(start);
        assert_eq!(breaker.consecutive_failures(), failures);
    }
}
