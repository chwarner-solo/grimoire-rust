# ADR-004: Hexagonal Architecture with Separate Adapter Crates

**Status:** Accepted

## Context

The application needs multiple interchangeable backends: file storage today,
cloud storage later; in-memory projections today, SQLite or Redis later; CLI
today, MCP and eventually a player-facing interface later. The domain logic
must not depend on any of these infrastructure choices.

## Decision

Use **hexagonal architecture** (ports and adapters). The workspace is structured
as separate crates with a strict dependency direction:

```
cli / mcp  →  app  →  domain
adapters/* →  app  →  domain
```

| Crate | Role |
|---|---|
| `domain` | Aggregates, events, commands, traits. Zero infrastructure dependencies. |
| `app` | Use cases, command handlers, port traits. Depends only on `domain`. |
| `adapters/event-store` | File-based event persistence. Implements `app` ports. |
| `adapters/projections` | Projection store (in-memory → SQLite → Redis etc). |
| `adapters/mcp` | MCP server — serves projections to AI clients. |
| `adapters/aggregate-store` | Aggregate snapshots for efficient replay. |
| `cli` | DM tooling. Depends on `app`. |

Ports (traits) are defined in `app`. Adapters implement those traits. The
`app` layer never imports from any adapter crate.

## Consequences

- Each adapter is independently swappable without touching domain or app.
- Local development uses the file-based event store with no cloud dependency.
- The AI (via MCP) is just another external actor — same pattern as the CLI.
- Adding a Foundry VTT import, a player-facing API, or a Kafka event stream
  means adding a new adapter crate, not modifying existing ones.
- The projections facade can fan out to multiple backends simultaneously
  (e.g. write to Redis and Postgres at the same time).
