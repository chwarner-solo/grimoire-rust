# ADR-010: REST API with axum for the DM Interface

**Status:** Accepted

## Context

Grimoire's first user-facing client is a PWA. The PWA needs an API to issue
commands and read aggregate state. Two options were considered: REST and
GraphQL.

GraphQL was appealing because the domain is naturally graph-shaped and the
AI query use case benefits from flexible field selection. However, GraphQL
mutations are awkward on an event-sourced backend — the write and read paths
are separated (CQRS) and state after a command is eventually consistent, which
does not fit GraphQL's request/response mutation model well.

The query flexibility of GraphQL is also already addressed by the planned
Neo4j projection, which handles cross-aggregate graph traversal in the
database layer. A GraphQL API would duplicate that capability rather than
composing with it.

## Decision

Use a **REST API** built with **axum** as the primary DM interface. Routes
are organised by actor, not by aggregate:

```
/dm/...      DM routes — full read/write access to all aggregates
/player/...  Player routes — own character only
/health      Public health check
```

Creates use parent-scoped paths for context (`POST /dm/stories/:id/characters`).
Mutations use direct ID paths (`PATCH /dm/characters/:id/name`).

A GraphQL endpoint may be added later as a second interface for the AI/MCP
query use case, sitting in front of Neo4j reads without touching the REST
command API.

## Consequences

- Command routes map directly to service methods — handlers are thin.
- The PWA makes standard HTTP requests; no GraphQL client needed.
- Authorization is per-route-tree: DM token → `/dm/`, player token → `/player/`.
- Adding a player-facing PWA or mobile client later is additive.
- See `docs/openapi.yaml` for the full route surface.
