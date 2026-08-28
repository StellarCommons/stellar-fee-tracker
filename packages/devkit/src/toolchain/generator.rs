/// Builds valid Horizon mock scenario JSON files from a fluent spec.
#[derive(Debug, Default, Clone)]
pub struct ScenarioGenerator {
    base_fee: u64,
    p50: u64,
    p95: u64,
    p99: u64,
}

impl ScenarioGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_fee(mut self, v: u64) -> Self {
        self.base_fee = v;
        self
    }

    pub fn p50(mut self, v: u64) -> Self {
        self.p50 = v;
        self
    }

    pub fn p95(mut self, v: u64) -> Self {
        self.p95 = v;
        self
    }

    pub fn p99(mut self, v: u64) -> Self {
        self.p99 = v;
        self
    }

    pub fn generate(&self) -> String {
        format!(
            "{{\"base_fee\":{},\"p50\":{},\"p95\":{},\"p99\":{}}}",
            self.base_fee, self.p50, self.p95, self.p99
        )
    }
}
