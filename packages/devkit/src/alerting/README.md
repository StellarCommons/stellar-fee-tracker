## Alerting Engine

Rule-based alerting over fee data:

- **AlertRule** — `id`, `name`, `condition`, `severity`, `dispatchers`,
  `cooldown`.
- **Conditions** — threshold, percentile breach, spike-count-over-window,
  and anomaly-score comparisons against a configurable value.
- **Dispatchers** — pluggable delivery targets (webhook, log, email) that
  receive a fired `AlertEvent`.
- **History** — fired alerts are persisted with their triggering sample so
  past incidents can be reviewed.
- **Engine setup** — construct an `AlertEngine` with a rule set and a
  dispatcher registry, then call `evaluate()` on each new fee sample.

See `packages/devkit/src/alerting/` for the implementation.
