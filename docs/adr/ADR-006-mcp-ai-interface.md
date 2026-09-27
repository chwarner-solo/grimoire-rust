# ADR-006: MCP as the AI Interface Layer

**Status:** Accepted

## Context

An AI (initially Claude) will use story data to assist DMs with session prep,
NPC dialogue, world detail, and narrative suggestions. The AI needs structured,
queryable access to story state — not raw event files.

## Decision

Expose story projections to the AI via **Model Context Protocol (MCP)** through
a dedicated `adapters/mcp` crate.

The architecture has two distinct read surfaces:

| Surface | Consumer | Shape |
|---|---|---|
| `adapters/mcp` | AI (Claude, etc.) | Structured MCP resources and tools |
| `adapters/aggregate-store` | DM via CLI | Full aggregate state for command validation |

The MCP adapter serves **projections** — purpose-built read models optimised
for AI consumption, not raw aggregate state.

The AI is treated as an external actor in the hexagonal architecture — no
different from the CLI. It consumes projections through a port.

## Consequences

- The AI never reads raw event files directly.
- Projections can be shaped specifically for AI context windows — e.g. a
  `StoryContext` projection assembling characters, quests, and locations into
  a single coherent view.
- The event listener pattern (file system watcher or cloud storage events)
  keeps projections up to date asynchronously.
- The MCP adapter is one of several future-facing adapters. Adding a REST API
  or WebSocket interface follows the same pattern.
- See ADR-007 for AI write constraints.
