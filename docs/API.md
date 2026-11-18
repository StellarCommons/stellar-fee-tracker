# Stellar Fee Tracker — API Reference

Base URL: `http://localhost:8080` (configurable via `API_PORT`)

All endpoints return JSON unless otherwise noted. Successful responses are always `200 OK` with a JSON body. Error responses use standard HTTP status codes and the shape `{"error": "<message>"}`.

Responses include the following caching headers where applicable:

- `Cache-Control: public, max-age=<n>, s-maxage=<m>` — indicates cacheability
- `ETag` — opaque fingerprint of the response body
- `Last-Modified` — RFC 7231 timestamp of the last data update

Clients should send `If-None-Match: <etag>` on subsequent requests; if the data has not changed the server returns `304 Not Modified` with no body.

Responses include the following caching headers where applicable:

- `Cache-Control: public, max-age=<n>, s-maxage=<m>` — indicates cacheability
- `ETag` — opaque fingerprint of the response body
- `Last-Modified` — RFC 7231 timestamp of the last data update

Clients should send `If-None-Match: <etag>` on subsequent requests; if the data has not changed the server returns `304 Not Modified` with no body.


---

## `GET /health`

Service liveness check. Returns `ok` as plain text.

**Request**

```bash
curl http://localhost:8080/health
```

**Response** — `200 OK`

```
ok
```

**Headers**

```
Cache-Control: no-store
```

---

## `GET /fees/current`

Returns the most recent fee stats fetched from Horizon.

**Request**

```bash
curl http://localhost:8080/fees/current
```

**Response** — `200 OK`

```json
{
  "base_fee": "100",
  "min_fee": "100",
  "max_fee": "10000",
  "avg_fee": "250",
  "percentiles": {
    "p10": "100",
    "p20": "100",
    "p30": "100",
    "p40": "150",
    "p50": "200",
    "p60": "250",
    "p70": "300",
    "p80": "500",
    "p90": "1000",
    "p95": "3000",
    "p99": "8000"
  }
}
```

**Error responses**

| Status | Body | Cause |
|---|---|---|
| `502 Bad Gateway` | `{"error": "<reason>"}` | Horizon could not be reached or returned an error (`AppError::Network`) |
| `422 Unprocessable Entity` | `{"error": "<reason>"}` | Horizon's response could not be parsed |
| `500 Internal Server Error` | `{"error": "<reason>"}` | The fee stats provider is missing or another unexpected error occurred |

Responses are cached for a few seconds and carry `ETag`, `Last-Modified` and `Cache-Control`; send `If-None-Match` to receive `304 Not Modified`.

---

## `GET /fees/history`

Returns fee data points held in the in-memory store for a recent window, with a summary.

**Query parameters**

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `window` | `string` | No | `1h` | One of `1h`, `6h`, `24h` |

**Request**

```bash
curl "http://localhost:8080/fees/history?window=6h"
```

**Response** — `200 OK`

```json
{
  "window": "6h",
  "from": "2024-01-15T04:30:00Z",
  "to": "2024-01-15T10:30:00Z",
  "data_points": 2,
  "fees": [
    {
      "fee_amount": 100,
      "timestamp": "2024-01-15T10:29:00Z",
      "transaction_hash": "…",
      "ledger_sequence": 48000001
    }
  ],
  "summary": { "min": 100, "max": 5000, "avg": 300.0, "p50": 100, "p95": 5000 }
}
```

`from` and `to` are the window bounds computed when the request is handled, and `summary` uses nearest-rank percentiles (all zeros when there are no data points). The `fees` items are the tracker's fee data points; the exact fields are those of `FeeDataPoint` in `packages/core/src/insights/types.rs`.

The response carries `ETag`, `Last-Modified` and `Cache-Control` headers and honours `If-None-Match`.

**Error responses**

| Status | Body | Cause |
|---|---|---|
| `400 Bad Request` | `{"error": "Unsupported window value: <value>"}` | `window` is not `1h`, `6h` or `24h` |
| `500 Internal Server Error` | `{"error": "Failed to serialize fee history: <reason>"}` | Serialization failed |

---

## `GET /fees/trend`

Returns the current congestion trend and how recent fee averages compare with longer baselines. It takes **no query parameters**.

**Request**

```bash
curl http://localhost:8080/fees/trend
```

**Response** — `200 OK`

```json
{
  "status": "Rising",
  "trend_strength": "Moderate",
  "changes": {
    "5m_pct": 12.4,
    "1h_pct": 3.1,
    "6h_pct": 3.1,
    "24h_pct": null
  },
  "recent_spike_count": 1,
  "predicted_congestion_minutes": null,
  "last_updated": "2024-01-15T10:30:00Z"
}
```

| Field | Type | Description |
|---|---|---|
| `status` | `string` | `Normal`, `Rising`, `Congested` or `Declining` |
| `trend_strength` | `string` | `Weak`, `Moderate` or `Strong` |
| `changes.5m_pct` | `f64 \| null` | Percent change of the 5-minute average against the 1-hour average |
| `changes.1h_pct` | `f64 \| null` | Percent change of the 1-hour average against the 24-hour average |
| `changes.6h_pct` | `f64 \| null` | Currently computed exactly like `1h_pct` (a known issue: it is not a separate 6-hour figure) |
| `changes.24h_pct` | `null` | Always `null`: there is no longer baseline to compare against |
| `recent_spike_count` | `u32` | Spikes detected recently |
| `predicted_congestion_minutes` | `i64 \| null` | Predicted remaining congestion duration, when available |
| `last_updated` | `string` | ISO 8601 timestamp of the insights snapshot |

A change is `null` when its baseline window is only partially filled or is not positive.

**Error responses**

| Status | Body | Cause |
|---|---|---|
| `500 Internal Server Error` | `{"error": "Insights engine missing from fees state"}` | Insights engine is not configured |

---

## `GET /fees/account/:account_id`

Returns fee statistics for an account's 100 most recent transactions, fetched live from Horizon.

**Path parameters:** `account_id` — a 56-character Stellar address starting with `G`.

```bash
curl http://localhost:8080/fees/account/GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF
```

**Response** — `200 OK`

```json
{
  "account_id": "G…",
  "transaction_count": 100,
  "total_fees": 12000,
  "avg_fee": 120.0,
  "min_fee": 100,
  "max_fee": 500,
  "transactions": [
    {
      "hash": "…",
      "fee_charged": 100,
      "ledger": 48000001,
      "created_at": "2024-01-15T10:29:00+00:00",
      "successful": true
    }
  ]
}
```

| Status | Body | Cause |
|---|---|---|
| `400 Bad Request` | `{"error": "…"}` | `account_id` is not a valid Stellar address |
| `404 Not Found` | `{"error": "No transactions found for account: <id>"}` | The account has no transactions |
| `502 Bad Gateway` | `{"error": "…"}` | Horizon could not be reached |
| `422 Unprocessable Entity` | `{"error": "…"}` | Horizon's response could not be parsed |
| `500 Internal Server Error` | `{"error": "Horizon client missing from fees state"}` | Horizon client is not configured |

---

## `GET /fees/transaction/:hash`

Looks up a transaction's fee in the tracker's in-memory store (it does not query Horizon).

**Path parameters:** `hash` — 64 hexadecimal characters.

```bash
curl http://localhost:8080/fees/transaction/<64-hex-hash>
```

**Response** — `200 OK`

```json
{
  "transaction_hash": "…",
  "fee_amount": 100,
  "ledger_sequence": 48000001,
  "timestamp": "2024-01-15T10:29:00Z"
}
```

| Status | Body | Cause |
|---|---|---|
| `400 Bad Request` | `{"error": "transaction hash must be 64 hexadecimal characters"}` | Malformed hash |
| `404 Not Found` | `{"error": "Transaction not found: <hash>"}` | The hash is not in the in-memory store |

---

## `GET /fees/wait-advisory`

Advises whether to submit now or wait, by comparing the current fee with the stored history.

**Query parameters**

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `window_secs` | `u64` | No | `86400` | Comparison window in seconds, `1` to `604800` (7 days) |

```bash
curl "http://localhost:8080/fees/wait-advisory?window_secs=3600"
```

**Response** — `200 OK`

```json
{
  "recommendation": "consider_waiting",
  "reason": "…",
  "congestion_level": "moderate",
  "congestion_score": 0.54,
  "current_fee_percentile": 82.0,
  "capacity_usage_available": true,
  "history_sample_count": 340,
  "actual_window_secs": 3600,
  "computed_at": "2024-01-15T10:30:00Z"
}
```

| Field | Description |
|---|---|
| `recommendation` | `submit_now`, `consider_waiting` or `wait` |
| `congestion_level` | `low`, `moderate`, `high` or `critical` |
| `congestion_score` | Weighted score from `0.0` to `1.0` |
| `current_fee_percentile` | Where the current fee sits in the window's distribution (0–100) |
| `capacity_usage_available` | Whether the latest `fee_stats` snapshot's ledger capacity usage was available (it lowers confidence when `false`) |
| `history_sample_count` | Data points in the comparison window |
| `actual_window_secs` | Window actually used (shorter than requested when history is short) |

| Status | Body | Cause |
|---|---|---|
| `400 Bad Request` | `{"error": "window_secs must be between 1 and 604800, got <n>"}` | `window_secs` out of range |

---

## `GET /fees/recommend`

A quick recommendation for inclusion within about 2 ledgers at medium urgency, based on the 100 most recent observed fees.

```bash
curl http://localhost:8080/fees/recommend
```

**Response** — `200 OK`

```json
{
  "recommended_fee": 200,
  "congestion": "MEDIUM",
  "basis": "p50",
  "min_fee": 100,
  "avg_fee": 250,
  "timestamp": "2024-01-15T10:30:00+00:00"
}
```

`congestion` is derived from `avg_fee / min_fee`: below `1.5` is `LOW` (basis `p20`), up to `3.0` is `MEDIUM` (`p50`), above is `HIGH` (`p70`). `min_fee` is the lowest of the recent observed fees, not the network base fee.

| Status | Body | Cause |
|---|---|---|
| `502 Bad Gateway` | `{"error": "No fee data available yet. Wait for the first poll cycle."}` | The store is empty (this is reported as an upstream error today) |
| other | `{"error": "…"}` | Errors from the recommendation engine |

---

## `POST /fees/recommend`

A configurable recommendation.

**Request body**

| Field | Type | Required | Description |
|---|---|---|---|
| `target_ledgers` | `u32` | No | Ledgers within which inclusion is wanted, `1` to `100` (default from `REC_DEFAULT_LEDGERS`) |
| `urgency` | `string` | No | `Low`, `Medium` (default), `High` or `Urgent` |
| `max_fee` | `string` | No | Upper bound in stroops, an integer string up to `100000000` |

```bash
curl -X POST http://localhost:8080/fees/recommend \
  -H 'Content-Type: application/json' \
  -d '{"target_ledgers": 3, "urgency": "High", "max_fee": "5000"}'
```

**Response** — `200 OK`

```json
{
  "recommended_fee": "300",
  "fee_in_stroops": 300,
  "estimated_wait_ledgers": 3,
  "confidence": 0.9,
  "network_condition": "normal",
  "alternatives": [
    { "fee": "200", "estimated_wait_ledgers": 5, "confidence": 0.8, "label": "economy" }
  ],
  "cold_start": false,
  "data_quality": null,
  "explanation": null
}
```

`network_condition` is `normal`, `rising`, `congested`, `declining` or `unknown`. `cold_start` is `true` when there is too little history; the response then uses a fallback fee and `network_condition` may be `unknown`. `data_quality` and `explanation` are objects when available and `null` otherwise; the values above are illustrative.

| Status | Body | Cause |
|---|---|---|
| `400 Bad Request` | `{"error": "…"}` | `target_ledgers` or `max_fee` is out of range or malformed |
| `422 Unprocessable Entity` | plain text from the JSON extractor | The body is not valid JSON for this schema (see issue "Return JSON errors for extractor rejections") |

---

## `GET /fees/recommend/history`

The 50 most recent recommendations that were persisted, newest first.

```bash
curl http://localhost:8080/fees/recommend/history
```

**Response** — `200 OK`

```json
{
  "entries": [
    {
      "id": 12,
      "requested_at": "2024-01-15T10:30:00Z",
      "target_ledgers": 2,
      "percentile_basis": "p50",
      "recommended_fee": 200
    }
  ]
}
```

`entries` is empty when no repository is configured. The urgency of the original request is not stored. Rows with an unparseable timestamp are skipped.

| Status | Body | Cause |
|---|---|---|
| `500 Internal Server Error` | `{"error": "Failed to query recommendations: <reason>"}` | Database query failed |

---

## `GET /insights`

Returns a full insights snapshot including rolling averages, extremes, and congestion data.

**Request**

```bash
curl http://localhost:8080/insights
```

**Response** — `200 OK`

```json
{
  "rolling_averages": {
    "avg_1h": 250.5,
    "avg_6h": 210.3,
    "avg_24h": 195.8
  },
  "extremes": {
    "all_time_min": 100,
    "all_time_max": 50000,
    "window_min": 100,
    "window_max": 8000
  },
  "congestion": {
    "score": 0.42,
    "label": "Moderate",
    "trend": "Increasing"
  },
  "last_updated": "2024-01-15T10:30:00Z"
}
```

---

## `GET /insights/averages`

Returns rolling average fees computed over multiple time windows.

**Request**

```bash
curl http://localhost:8080/insights/averages
```

**Response** — `200 OK`

```json
{
  "avg_1h": 250.5,
  "avg_6h": 210.3,
  "avg_24h": 195.8,
  "avg_7d": 180.2
}
```

---

## `GET /insights/extremes`

Returns all-time and windowed minimum/maximum fee values.

**Request**

```bash
curl http://localhost:8080/insights/extremes
```

**Response** — `200 OK`

```json
{
  "all_time_min": 100,
  "all_time_max": 50000,
  "window_min": 100,
  "window_max": 8000,
  "window_hours": 24
}
```

---

## `GET /insights/congestion`

Returns the current congestion score, label, and trend direction.

**Request**

```bash
curl http://localhost:8080/insights/congestion
```

**Response** — `200 OK`

```json
{
  "score": 0.42,
  "label": "Moderate",
  "trend": "Increasing",
  "spike_count_1h": 3,
  "capacity_usage": 0.65
}
```

Fields:

| Field | Type | Description |
|---|---|---|
| `score` | `f64` | Congestion score from 0.0 (none) to 1.0 (maximum) |
| `label` | `string` | `Low`, `Moderate`, `High`, or `Critical` |
| `trend` | `string` | `Increasing`, `Decreasing`, or `Stable` |
| `spike_count_1h` | `u32` | Number of fee spikes detected in the last hour |
| `capacity_usage` | `f64` | Estimated network capacity utilisation (0.0–1.0) |

---

## `GET /insights/health`

Returns the health status of the insights engine.

**Request**

```bash
curl http://localhost:8080/insights/health
```

**Response** — `200 OK`

```json
{
  "status": "healthy",
  "last_update": "2024-01-15T10:30:00Z",
  "config": {
    "polling_interval_seconds": 10,
    "time_windows": 4,
    "spike_threshold": 3.0
  }
}
```

---

Alert configs and events carry an `alert_type` of `spike`, `recovery`,
`good_window`, or `stale_data` (Issue #556). `spike` fires when a fee
spike crosses the configured `threshold`; `recovery` fires when an
active spike condition clears, correlated back to the spike it resolves
via `correlation_id`; `good_window` fires when congestion is declining
(a good time to submit); `stale_data` fires when the poll pipeline's
data freshness exceeds the configured staleness threshold. All four
share the same webhook delivery mechanism (SSRF-guarded HTTPS-only
URLs, retry-once on non-2xx). The webhook payload's `event` field
identifies which kind fired (`fee_spike_detected`,
`fee_spike_recovered`, `good_submission_window`, `data_pipeline_stale`);
fields that don't apply to a given type (e.g. `peak_fee` for a
`good_window` event) are omitted from the JSON body entirely rather than
sent as `null`.

## `POST /alerts/config`

Registers a new alert webhook target. Each call creates a new,
independent config row — it does not replace or merge with existing
ones.

**Request body**

```json
{
  "webhook_url": "https://hooks.slack.com/services/xxx",
  "threshold": "Major",
  "alert_type": "spike"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `webhook_url` | `string` | Yes | HTTPS URL with a public hostname (SSRF-guarded: loopback/private/link-local hosts are rejected) |
| `threshold` | `string` | No | Alert severity: `Minor`, `Moderate`, `Major`, `Critical` (default `Major`). Only meaningful for `spike`/`recovery` alert types |
| `alert_type` | `string` | No | `spike`, `recovery`, `good_window`, or `stale_data` (default `spike`) |

**Request**

```bash
curl -X POST http://localhost:8080/alerts/config \
  -H "Content-Type: application/json" \
  -d '{"webhook_url": "https://hooks.slack.com/services/xxx", "threshold": "Moderate", "alert_type": "stale_data"}'
```

**Response** — `201 Created`

```json
{ "id": 1 }
```

**Error responses**

| Status | Body | Cause |
|---|---|---|
| `400 Bad Request` | `{"error": "Invalid threshold '<value>'. Must be one of: ..."}` | Unrecognised threshold value |
| `400 Bad Request` | `{"error": "Invalid alert_type '<value>'. Must be one of: ..."}` | Unrecognised alert_type value |
| `400 Bad Request` | `{"error": "Invalid webhook_url: must be an HTTPS URL with a public hostname"}` | URL is not HTTPS, or resolves to a private/loopback/link-local host |

---

## `GET /alerts/config`

Lists every registered alert webhook config, including disabled
(soft-deleted) ones.

**Request**

```bash
curl http://localhost:8080/alerts/config
```

**Response** — `200 OK`

```json
[
  {
    "id": 1,
    "webhook_url": "https://hooks.slack.com/services/xxx",
    "threshold": "Major",
    "alert_type": "spike",
    "enabled": true,
    "created_at": "2024-01-15T08:00:00Z"
  }
]
```

---

## `PATCH /alerts/config/:id`

Partially updates a config. Any field omitted from the request body
keeps its current value.

**Request body**

```json
{
  "threshold": "Critical",
  "enabled": false,
  "alert_type": "good_window"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `threshold` | `string` | No | New severity threshold |
| `enabled` | `bool` | No | New enabled state |
| `alert_type` | `string` | No | New alert type: `spike`, `recovery`, `good_window`, or `stale_data` |

**Request**

```bash
curl -X PATCH http://localhost:8080/alerts/config/1 \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'
```

**Response** — `204 No Content`

**Error responses**

| Status | Body | Cause |
|---|---|---|
| `400 Bad Request` | `{"error": "Invalid threshold '<value>'. Must be one of: ..."}` | Unrecognised threshold value |
| `400 Bad Request` | `{"error": "Invalid alert_type '<value>'. Must be one of: ..."}` | Unrecognised alert_type value |
| `404 Not Found` | `{"error": "Alert config not found"}` | No config with that id |

---

## `DELETE /alerts/config/:id`

Soft-deletes a config by setting `enabled` to `false`. The row still
appears in `GET /alerts/config`.

**Request**

```bash
curl -X DELETE http://localhost:8080/alerts/config/1
```

**Response** — `204 No Content`

**Error responses**

| Status | Body | Cause |
|---|---|---|
| `404 Not Found` | `{"error": "Alert config not found"}` | No config with that id |

---

## `GET /alerts/history`

Returns a paginated list of triggered alert events.

**Query parameters**

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `limit` | `i64` | No | `20` | Number of events to return (clamped to `100`) |
| `offset` | `i64` | No | `0` | Number of events to skip, newest first (negative values are treated as `0`) |
| `severity` | `string` | No | — | Filter by severity: `Minor`, `Moderate`, `Major`, `Critical` |
| `delivered` | `bool` | No | — | Filter by webhook delivery success |
| `alert_type` | `string` | No | — | Filter by `spike`, `recovery`, `good_window`, or `stale_data` |

**Request**

```bash
curl "http://localhost:8080/alerts/history?limit=10&alert_type=stale_data"
```

**Response** — `200 OK`

```json
{
  "total": 1,
  "items": [
    {
      "id": 42,
      "config_id": 1,
      "alert_type": "stale_data",
      "severity": "Major",
      "peak_fee": 0,
      "baseline_fee": 0.0,
      "spike_ratio": 0.0,
      "webhook_url": "https://hooks.slack.com/services/xxx",
      "delivered": true,
      "triggered_at": "2024-01-15T09:15:00Z",
      "correlation_id": null
    }
  ]
}
```

`peak_fee`, `baseline_fee`, and `spike_ratio` only carry meaningful
values for `spike` events; they default to `0`/`0.0` for the other
three alert types. `correlation_id` is set only on `recovery` events,
identifying the spike they resolve.

**Error responses**

| Status | Body | Cause |
|---|---|---|
| `400 Bad Request` | `{"error": "Invalid severity '<value>'. Must be one of: ..."}` | Unrecognised severity value |
| `400 Bad Request` | `{"error": "Invalid alert_type '<value>'. Must be one of: ..."}` | Unrecognised alert_type value |

---

## `GET /metrics`

Returns Prometheus-format metrics for scraping.

**Request**

```bash
curl http://localhost:8080/metrics
```

**Response** — `200 OK`, `Content-Type: text/plain; version=0.0.4`

```
# HELP stellar_fee_polls_total Total number of Horizon fee polls attempted
# TYPE stellar_fee_polls_total counter
stellar_fee_polls_total 1234

# HELP stellar_fee_poll_errors_total Total number of failed Horizon fee polls
# TYPE stellar_fee_poll_errors_total counter
stellar_fee_poll_errors_total 2

# HELP stellar_current_base_fee Current base fee in stroops from the last successful poll
# TYPE stellar_current_base_fee gauge
stellar_current_base_fee 100

# HELP stellar_api_requests_total Total number of REST API requests by path and status
# TYPE stellar_api_requests_total counter
stellar_api_requests_total{path="/fees/current",status="200"} 890
stellar_api_requests_total{path="/insights",status="200"} 341
```

---

## Error Response Format

All error responses use this consistent shape:

```json
{
  "error": "human-readable error message"
}
```

Common HTTP status codes:

| Status | Meaning |
|---|---|
| `400` | Bad request — malformed query parameters or request body |
| `404` | Not found — resource does not exist |
| `422` | Unprocessable entity — valid JSON but semantically invalid |
| `429` | Too many requests — rate limit exceeded |
| `500` | Internal server error — unexpected failure |
| `503` | Service unavailable — Horizon unreachable |
