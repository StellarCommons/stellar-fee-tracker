/// Mock implementation of the Horizon API server for use in tests.
pub struct HorizonMock {
    /// Canned JSON response to serve at `GET /fee_stats`.
    pub fee_stats_response: String,
}

impl HorizonMock {
    pub fn new(fee_stats_response: impl Into<String>) -> Self {
        Self {
            fee_stats_response: fee_stats_response.into(),
        }
    }

    /// Returns the canned response body for `GET /fee_stats`.
    pub fn fee_stats_payload(&self) -> &str {
        &self.fee_stats_response
    }
}
