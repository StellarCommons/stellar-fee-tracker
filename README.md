# stellar-fee-tracker

Stellar Fee Tracker provides real-time insights into Stellar network transaction fees. It helps developers and users optimize fee settings by tracking base fees, average paid fees, and network congestion trends.

## Modules

### Fee Insights (Analysis Module)
The `insights` module analyzes raw blockchain fee data to provide actionable intelligence:

#### Key Features
- **Rolling Averages**: Computes 5-minute, hourly, and daily rolling averages of fees.
- **Extremes Tracking**: Identifies historical maximum and minimum fees.
- **Congestion Detection**: Flags periods of network congestion based on fee spikes.
- **Trend Analysis**: Uses linear regression to predict fee trends (stable, rising, falling).

#### Core Components
```rust
insights/
├── engine.rs          # Main analysis engine coordinating all components
├── calculator.rs      # Rolling averages and statistical calculations
├── tracker.rs         # Real-time extremes tracking
├── detector.rs        # Congestion detection logic
├── types.rs           # Public data types (FeeInsight, CongestionLevel)
└── error.rs           # Insights-specific error handling
```

#### Usage Example
```rust
use stellar_fee_tracker_core::insights::{FeeInsightsEngine, InsightsConfig};

let config = InsightsConfig {
    horizon_url: "https://horizon.stellar.org".to_string(),
    window_size: 5 * 60,  // 5-minute rolling windows
};

let engine = FeeInsightsEngine::new(config);
let insights = engine.compute_current_insights().await?;
println!("Current congestion: {:?}", insights.congestion_level);
```

#### Configuration
| Parameter               | Description                                      | Default Value                     |
|-------------------------|--------------------------------------------------|-----------------------------------|
| `horizon_url`           | Stellar Horizon endpoint URL                     | `https://horizon.stellar.org`    |
| `window_size`           | Rolling window size in seconds                   | `300` (5 minutes)                |
| `sample_interval`       | Frequency of sampling in milliseconds           | `10_000` (10 seconds)            |
| `congestion_threshold`  | Fee multiplier to trigger congestion warnings   | `3.0`                            |

#### Output Data
```rust
pub struct FeeInsight {
    pub timestamp: DateTime<Utc>,
    pub rolling_averages: RollingAverages,
    pub extremes: FeeExtremes,
    pub congestion_level: CongestionLevel,
    pub trend_direction: TrendDirection,
}

pub enum CongestionLevel {
    Low,
    Medium,
    High,
}
```

#### Error Handling
All errors implement `std::error::Error`. Common variants:
- `InsightsError::HorizonConnection` - Horizon network issues
- `InsightsError::InvalidData` - Malformed fee samples
- `InsightsError::WindowTooSmall` - Config validation error

## CLI Usage

The CLI provides real-time fee monitoring and insights:

```bash
stellar-fee-tracker [OPTIONS]
```

### Options

| Option                   | Description                                      | Default Value                     |
|--------------------------|--------------------------------------------------|-----------------------------------|
| `--network <NETWORK>`   | Stellar network (`mainnet` or `testnet`)        | `mainnet`                        |
| `--horizon-url <URL>`    | Custom Horizon API URL                          | `https://horizon.stellar.org`    |
| `--poll-interval <SECS>` | Polling interval in seconds                      | `10`                             |

### Examples

```bash
# Default mainnet monitoring
stellar-fee-tracker

# Testnet with custom polling interval
stellar-fee-tracker --network testnet --poll-interval 5

# Custom Horizon URL
stellar-fee-tracker --horizon-url "https://horizon-testnet.stellar.org"
```

### Output Format

```
Fees: base=100 stroops, avg=120 stroops, max=300 stroops, congestion=low
Trend: stable, Samples: 45, Next update: 10s
```

---