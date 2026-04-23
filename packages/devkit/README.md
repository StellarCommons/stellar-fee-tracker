# DevKit

Development toolkit for local testing and debugging.

## Installation

```bash
pip install -e .
```

## Usage

```python
from devkit import FeeTracker

tracker = FeeTracker(network="testnet")
tracker.connect()
```

## Environment Variables

- `HORIZON_URL`: Custom Horizon API endpoint
- `NETWORK`: Network type (`testnet` or `mainnet`)
