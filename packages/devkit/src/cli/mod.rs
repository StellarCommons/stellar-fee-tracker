pub mod benchmark;
pub mod export;
pub mod replay;

/// Arguments for the `simulate` subcommand.
pub struct SimulateArgs {
    /// Base fee floor in stroops.
    pub base_fee: u64,
    /// Probability of a fee spike on any given ledger [0.0, 1.0].
    pub spike_prob: f64,
    /// Number of ledgers to simulate.
    pub duration: u64,
}

impl Default for SimulateArgs {
    fn default() -> Self {
        Self {
            base_fee: 100,
            spike_prob: 0.05,
            duration: 1_000,
        }
    }
}

impl SimulateArgs {
    /// Run the fee simulation and stream results to stdout.
    pub fn run(&self) {
        println!(
            "Simulating {} ledgers | base_fee={} stroops | spike_prob={:.2}",
            self.duration, self.base_fee, self.spike_prob
        );
    }
}
