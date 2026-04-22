/// Mock implementation of the Horizon API server for use in tests.
pub struct HorizonMock {
    /// Name of the currently active scenario.
    pub scenario: String,
    /// Optional simulated response delay in milliseconds.
    pub delay_ms: Option<u64>,
}

impl HorizonMock {
    pub fn new(scenario: impl Into<String>) -> Self {
        Self { scenario: scenario.into(), delay_ms: None }
    }

    /// Sets the simulated network latency delay.
    pub fn with_delay_ms(mut self, ms: u64) -> Self {
        self.delay_ms = Some(ms);
        self
    }

    /// Applies the configured delay, if any. Call before serving a response.
    pub fn apply_delay(&self) {
        if let Some(ms) = self.delay_ms {
            std::thread::sleep(std::time::Duration::from_millis(ms));
        }
    }

    /// Switches to the next scenario from the rotator and updates the active scenario.
    pub fn rotate(&mut self, rotator: &mut crate::harness::scenarios::ScenarioRotator) {
        if let Some(next) = rotator.next() {
            self.scenario = next.to_string();
        }
    }

    /// Logs a request to stdout with timestamp, method, path, and active scenario name.
    pub fn log_request(&self, method: &str, path: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        println!("[{}] {} {} scenario={}", now, method, path, self.scenario);
    }

    /// Returns the JSON body for `GET /health`.
    pub fn health_payload(&self) -> String {
        format!(r#"{{"status":"ok","scenario":"{}"}}"#, self.scenario)
    }
}
