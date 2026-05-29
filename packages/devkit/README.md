# stellar-devkit

Developer toolkit for the Stellar Fee Tracker. Provides utilities for testing, mocking, and simulating Stellar network behaviour without hitting live infrastructure.

## Scope

`stellar-devkit` is a standalone testing and simulation package. It must not import from `stellar-core` or any live-network crate. All functionality is self-contained and intended for use in `[dev-dependencies]` only.

## Boundary Rules

- No imports from `packages/core`
- No live Horizon API calls
- No database connections
- All external I/O must be injectable or mockable

## Modules

| Module | Description |
|---|---|
| `harness` | Mock Horizon server and pre-built fee scenario fixtures |
| `harness::scenarios` | JSON scenario files and runtime loader |
| `simulation` | Fee models, network-load generators, congestion predictors |
| `analysis` | Percentile stats, spike classification, rolling window |
| `cli` | Replay, export, and benchmark CLI stubs |
| `types` | Shared types: `FeeRecord`, `Scenario`, `SimResult` |
| `error` | `DevkitError` unified error enum |

## Simulation

The `simulation` module provides fee modelling and network-load generation without any live-network dependencies.

### `FeeModelConfig` fields

| Field | Type | Description |
|---|---|---|
| `base_fee` | `u64` | Minimum fee (stroops) used as the simulation floor |
| `surge_multiplier` | `f64` | Fee multiplier applied when the network is congested |
| `congestion_threshold` | `f64` | Load ratio (0.0–1.0) above which surge pricing activates |

### Example usage

```rust
use stellar_devkit::simulation::{FeeModel, NetworkLoad};

let load = NetworkLoad::constant(0.85);          // 85 % utilisation
let result = FeeModel::run(&load, base_fee: 100, surge_multiplier: 2.0, congestion_threshold: 0.8);
println!("recommended fee: {} stroops", result.recommended_fee);
```

### Output format (`SimResult`)

| Field | Type | Description |
|---|---|---|
| `recommended_fee` | `u64` | Suggested fee for the simulated conditions |
| `congested` | `bool` | Whether surge pricing was triggered |
| `load_ratio` | `f64` | Network utilisation at simulation time |

## CLI

The `cli` module provides command-line subcommands for replaying, exporting, simulating, and benchmarking fee data.

### `mock`

Start a mock Horizon server using pre-built fee scenario fixtures. Useful for local development and integration testing without a live network connection.

```bash
cargo run -p stellar-devkit -- mock --scenario spike
```

Flags:

| Flag | Description |
|---|---|
| `--scenario <name>` | Scenario fixture to load: `normal`, `congested`, `spike`, `recovery` |

### `replay`

Replay a recorded fee scenario against the devkit harness. Feeds historical or synthetic fee data through the analysis pipeline to verify behaviour.

```bash
cargo run -p stellar-devkit -- replay --scenario congested --speed 1.0
```

Flags:

| Flag | Description |
|---|---|
| `--scenario <name>` | Scenario to replay |
| `--speed <factor>` | Playback speed multiplier (default: 1.0) |
| `--from <timestamp>` | Start replay from a specific timestamp |
| `--to <timestamp>` | Stop replay at a specific timestamp |

### `export`

Export fee data to CSV or JSON format for external analysis and reporting.

```bash
# Export to CSV
cargo run -p stellar-devkit -- export --format csv --output fees.csv

# Export to JSON
cargo run -p stellar-devkit -- export --format json --output fees.json

# Export a specific time window
cargo run -p stellar-devkit -- export --format csv --window 1h --output recent.csv
```

Flags:

| Flag | Description |
|---|---|
| `--format <fmt>` | Output format: `csv` or `json` |
| `--output <path>` | Write output to a file instead of stdout |
| `--window <duration>` | Restrict export to a recent time window (e.g. `1h`, `24h`) |

CSV output columns: `timestamp`, `fee`, `ledger`, `is_spike`.

### `simulate`

Run a fee simulation using the built-in fee model and network load generator. Predicts recommended fees and congestion status without hitting live infrastructure.

```bash
cargo run -p stellar-devkit -- simulate --base-fee 100 --surge 2.0 --threshold 0.8
```

Flags:

| Flag | Description |
|---|---|
| `--base-fee <stroops>` | Minimum fee floor in stroops |
| `--surge <multiplier>` | Surge pricing multiplier when congested |
| `--threshold <ratio>` | Congestion threshold (0.0–1.0) |

### `benchmark`

Run benchmarks comparing SMA, EMA, and WMA rolling-window algorithms on spike fee data. Outputs a comparison table to stdout.

```bash
cargo run -p stellar-devkit -- benchmark --window 20 --alpha 0.3
```

Flags:

| Flag | Description |
|---|---|
| `--window <size>` | Rolling window size for SMA/WMA (default: 20) |
| `--alpha <value>` | Smoothing factor for EMA (0.0–1.0, default: 0.3) |

Output example:

```
idx          SMA          EMA          WMA
19       102.3456       98.7654      101.2345
20       103.4567       99.1234      102.3456
...
```

## Running

```bash
# Run all devkit tests
cargo test -p stellar-devkit

# Run a specific test file
cargo test -p stellar-devkit --test harness_congested
```

## Mock Horizon Server

The harness exposes canned `GET /fee_stats` payloads through `HorizonMock` and the JSON fixtures in `src/harness/scenarios/`.

```bash
# Start with the baseline fixture
cargo test -p stellar-devkit --test harness_normal -- --nocapture

# Swap to a higher-pressure fixture
cargo test -p stellar-devkit --test harness_congested -- --nocapture
```

Scenario flags map directly to the fixture you load in your test setup:

- `normal` for a low-fee baseline
- `congested` for sustained high-fee demand
- `spike` for a sudden short-lived fee jump
- `recovery` for a return from congestion toward baseline

```rust
use std::path::Path;

use stellar_devkit::harness::{
    horizon_mock::HorizonMock,
    scenarios::load_from_file,
};

let payload = load_from_file(Path::new("src/harness/scenarios/spike.json"))?;
let mock = HorizonMock::new(payload);
assert!(mock.fee_stats_payload().contains("\"scenario\": \"spike\""));
```

## Adding to Your Crate

```toml
[dev-dependencies]
stellar-devkit = { path = "../devkit" }
```
