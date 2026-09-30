# ADR-014: Structured Logging with tracing

**Status:** Accepted

## Context

The application needs observability at two levels:
1. Visibility when things break — errors surfaced quickly with context.
2. Traceability — the ability to reconstruct what happened for a given
   request or entity after the fact.

The target deployment environments (AWS CloudWatch, GCP Cloud Logging)
expect JSON-formatted, structured log lines rather than free-form text.
Local development benefits from human-readable output.

## Decision

Use the **`tracing`** crate ecosystem throughout:

**Spans at the service layer** — every service method is annotated with
`#[tracing::instrument]`, capturing the user ID and entity IDs as span
fields. This produces a named span for every command execution:

```
INFO app::story: create  owner=usr_… story_id=abc123
INFO app::story: story created  story_id=abc123
```

**Levelled events at the API boundary:**
- `WARN` — domain rule violations, validation failures, 404s, auth
  rejections. Expected errors; no alert needed.
- `ERROR` — infrastructure failures (storage errors, unexpected panics).
  These warrant attention.
- HTTP request/response details are handled by `tower_http::TraceLayer`
  at `DEBUG` level.

**Externally configurable format** via `LOG_FORMAT` environment variable:
- `pretty` (default) — human-readable, coloured output for local dev
- `json` — newline-delimited JSON for cloud log ingestion

The Docker image defaults to `LOG_FORMAT=json`. Log verbosity is
controlled separately via `RUST_LOG` (standard `tracing-subscriber`
`EnvFilter` syntax).

## Consequences

- Every failed command carries the entity ID and user ID in its log line —
  no need to correlate across separate log entries.
- CloudWatch Logs Insights and GCP Log Explorer can filter and query on
  any structured field (`story_id`, `level`, `target`, etc.).
- Switching to `tracing-stackdriver` for native GCP severity mapping is
  a one-function change in `cli/main.rs` — nothing else changes.
- The `tracing` crate is a workspace dependency; all crates that need
  observability (`app`, `adapters`, `api`) declare it directly.
