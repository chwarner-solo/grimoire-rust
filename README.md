# Grimoire

An event-sourced TTRPG campaign manager. Grimoire stores the history of a
tabletop campaign as an immutable event log, and exposes the current story
state to an AI via MCP — so the AI can help DMs with session prep, NPC
dialogue, and narrative continuity.

---

## What It Does

A DM creates a **Story** and populates it with **Characters**, **Quests**,
and **Locations**. Each **Session** of play records what happened — encounters
run, quests completed, characters lost. The AI reads the current state of the
story and can assist with what comes next.

Nothing is ever overwritten. Every change is an event. The full history of
every campaign is preserved and replayable.

---

## Current Status

| Layer | Status |
|---|---|
| `domain` | Built — all five aggregates with events, commands, and tests |
| `app` | Stub — ports and use cases not yet implemented |
| `adapters/*` | Stub — event store, projections, MCP, aggregate store |
| `cli` | Stub — entry point only |

The domain is the right place to start reading. Everything else is planned
architecture waiting to be built.

---

## Architecture

Grimoire uses **hexagonal architecture** and **event sourcing**.

```
┌─────────────┐     ┌─────────────┐
│     cli     │     │  adapters/  │
│    (DM)     │     │     mcp     │  ← AI via MCP
└──────┬──────┘     └──────┬──────┘
       │                   │
       └─────────┬─────────┘
                 ▼
          ┌────────────┐
          │    app     │  ← ports (traits) + use cases
          └──────┬─────┘
                 ▼
          ┌────────────┐
          │   domain   │  ← aggregates, events, commands
          └────────────┘
                 ▲
       ┌─────────┼──────────┐
       │         │          │
  event-store  projections  aggregate-store
```

**`domain`** — pure Rust, no infrastructure dependencies. Five aggregates:
Story, Character, Quest, Location, Session, and Encounter. Each has its own
event and command enums.

**`app`** — defines port traits (`EventStore`, `ProjectionStore`, etc.) and
command handlers. Depends only on `domain`.

**`adapters/*`** — implement the ports. Each is an independent crate:

| Adapter | Role |
|---|---|
| `event-store` | Writes/reads JSON event files from disk |
| `projections` | Materialised views built from replayed events |
| `mcp` | MCP server — serves projections to AI clients |
| `aggregate-store` | Aggregate snapshots for efficient replay |

See [ADR-004](docs/adr/ADR-004-hexagonal-architecture.md) for the full
rationale.

---

## Domain Model

Five aggregates, all scoped to a Story:

| Aggregate | What it models | Key states |
|---|---|---|
| `Story` | The campaign root — title, premise, ownership | Active / Inactive |
| `Character` | PCs and NPCs | Active / Departed / Deceased |
| `Quest` | Story lines — main arc, side quests, personal arcs | Active / Completed / Failed / Abandoned |
| `Location` | Places in the world | Known / Visited / Destroyed |
| `Session` | One sitting of play | Planned / Active / Closed |
| `Encounter` | A discrete moment of engagement | Planned / Active / Resolved / Abandoned / DeadEnded |

Each aggregate processes commands and emits events:

```rust
// Validate a command against current state → produce events
fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>

// Reconstruct state from an event — always infallible
fn apply(self, event: Self::Event) -> Self

// Replay: reconstruct current state from the full event stream
let state = events.into_iter().fold(Aggregate::default(), Aggregate::apply);
```

See [docs/domain/](docs/domain/) for command and event reference for each aggregate.

---

## Event Storage

Events are stored as individual JSON files. The file path encodes identity:

```
events/
  {owner_id}/
    {story_id}/
      {aggregate_type}/
        {aggregate_id}/
          {uuid_v7}.json     ← UUID v7 = time-sortable, replay order is ls | sort
```

Each file contains one event:
```json
{
  "type": "CharacterDeparted",
  "at": "2026-09-27T14:32:00Z",
  "data": { "session_id": "01j4z..." }
}
```

See [ADR-002](docs/adr/ADR-002-ndjson-event-format.md) and
[ADR-003](docs/adr/ADR-003-event-file-structure.md).

---

## Getting Started

**Requirements:** Rust (edition 2024, stable toolchain)

```bash
# Build everything
cargo build

# Run all tests
cargo test

# Run domain tests only
cargo test -p domain
```

All meaningful tests currently live in the `domain` crate alongside the
code they test. There are 24 tests covering the aggregate lifecycle rules.

---

## Documentation

| Location | Contents |
|---|---|
| [`docs/domain/`](docs/domain/) | Command and event reference for each aggregate |
| [`docs/journeys/`](docs/journeys/) | User journey narratives — story creation, sessions, encounters |
| [`docs/adr/`](docs/adr/) | Architecture Decision Records — why things are the way they are |

**Start here if you're new:**
1. [`docs/domain/overview.md`](docs/domain/overview.md) — the domain model in one page
2. [`docs/journeys/story-creation.md`](docs/journeys/story-creation.md) — how a campaign gets started
3. [`docs/adr/ADR-001-event-sourcing.md`](docs/adr/ADR-001-event-sourcing.md) — why event sourcing

---

## Key Decisions

A short list of the most important architectural choices. Each links to the
full ADR with context and rationale.

| Decision | ADR |
|---|---|
| Event sourcing as persistence | [ADR-001](docs/adr/ADR-001-event-sourcing.md) |
| JSON over Protobuf/Avro | [ADR-002](docs/adr/ADR-002-ndjson-event-format.md) |
| Path-based event identity | [ADR-003](docs/adr/ADR-003-event-file-structure.md) |
| Hexagonal architecture | [ADR-004](docs/adr/ADR-004-hexagonal-architecture.md) |
| Apply + Aggregate traits | [ADR-005](docs/adr/ADR-005-aggregate-traits.md) |
| MCP as AI interface | [ADR-006](docs/adr/ADR-006-mcp-ai-interface.md) |
| Immutable log + AI constraints | [ADR-007](docs/adr/ADR-007-immutable-event-log.md) |
