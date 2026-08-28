## Developer Toolchain

Utilities for working with fee scenario data during development:

- **Generator** — builds valid Horizon mock scenario JSON files from a spec.
- **Linter** — validates a scenario JSON file against the Horizon
  `fee_stats` schema and reports field-level errors.
- **Differ** — compares two fee sequences and summarizes additions,
  removals, and changed records.
- **Reporter** — renders a fee sequence as a Markdown or self-contained
  HTML report, including summary statistics and a chart.
- **Chart** — produces a dependency-free SVG line chart from a fee
  sequence, marking spike events.
- **Anonymiser** — strips or hashes `transaction_hash` fields from fee
  records so they can be shared safely.

See `packages/devkit/src/toolchain/` for the individual submodules.
