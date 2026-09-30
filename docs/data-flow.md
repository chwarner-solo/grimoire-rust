# Data Flow

This document traces how data moves through Grimoire — from a DM issuing a
command to an AI receiving a projection. Understanding this flow tells you
where new code belongs and how the layers stay decoupled.

---

## The Four Flows

| Flow | Trigger | Path |
|---|---|---|
| [1. DM Command](#1-dm-command) | DM issues a command via REST API or CLI | API/CLI → app → domain → event-store → event-bus → projections |
| [2. Aggregate Load](#2-aggregate-load) | app needs current aggregate state | app → RAM cache → snapshot-store → event-store → domain |
| [3. Projection Update](#3-projection-update) | New events appended to event store | event-bus → EventProjection handlers |
| [4. AI Query](#4-ai-query) | AI requests story state via MCP | AI → mcp → projections |

---

## 1. DM Command

The most common flow. The DM issues an instruction — a character departs,
a quest is completed, a session closes.

```
DM input (HTTP request or CLI)
   │
   ▼
┌─────────┐
│  api    │  authenticates JWT, extracts UserId
│  or cli │  routes to the right service method
└────┬────┘
     │  e.g. character_service.depart(id, session_id)
     ▼
┌─────────────────────────────┐
│  app / CharacterService     │
│                             │
│  1. repository.load(id)  ──→ see Flow 2
│  2. character.handle(Depart)
│  3. repository.save(id, events)
└────────────────┬────────────┘
                 │  internally:
                 ▼
┌────────────────────┐
│  adapters/         │
│  FsEventStore      │  appends each event as a JSON line
│                    │  to: {data_dir}/characters/events/{id}.jsonl
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│  adapters/         │
│  FsSnapshotStore   │  writes updated aggregate state
│                    │  to: {data_dir}/characters/snapshots/{id}.json
└────────┬───────────┘
         │
         ▼
  EventBus publishes each new event
  (triggers Flow 3 — projection update)
```

**Port traits in `app`:**

```rust
// The only interface use cases see
trait AggregateRepository<A: Aggregate> {
    async fn load(&self, id: A::Id) -> Result<A, RepositoryError>;
    async fn save(&self, id: A::Id, events: Vec<A::Event>) -> Result<(), RepositoryError>;
}

// Durable append-only log
trait EventStore<E> {
    async fn append(&self, aggregate_id: Uuid, events: &[E]) -> Result<u64, EventStoreError>;
    async fn load(&self, aggregate_id: Uuid) -> Result<Vec<EventEnvelope<E>>, EventStoreError>;
    async fn load_from(&self, aggregate_id: Uuid, after_sequence: u64) -> ...;
}

// Snapshot for fast cold-start
trait SnapshotStore<A: Aggregate> {
    async fn load(&self, id: A::Id) -> Result<Option<(A, u64)>, SnapshotError>;
    async fn save(&self, id: A::Id, state: &A, sequence: u64) -> Result<(), SnapshotError>;
}
```

**Event envelope written to the JSONL file:**

```json
{"aggregate_id":"550e8400-…","sequence":5,"occurred_at":"2026-09-30T…","payload":{"Departed":{"session_id":"01j4z…"}}}
```

---

## 2. Aggregate Load

Before `app` can call `handle`, it needs the aggregate's current state.
The `FsAggregateRepository` uses a three-layer lookup. See
[ADR-012](adr/ADR-012-aggregate-repository.md).

```
repository.load(character_id)
   │
   ▼
┌──────────────┐
│  L1 RAM cache│  hit? return immediately (no I/O)
└──────┬───────┘
       │ miss
       ▼
┌──────────────────────────┐
│  L2 FsSnapshotStore      │  read {data_dir}/characters/snapshots/{id}.json
│                          │  → (Character state, last_sequence)
│                          │  or (Character::default(), 0) if no snapshot
└──────┬───────────────────┘
       │  seed + sequence N
       ▼
┌──────────────────────────┐
│  L3 FsEventStore         │  read {data_dir}/characters/events/{id}.jsonl
│                          │  skip lines where sequence ≤ N
│                          │  → Vec<CharacterEvent> from N+1 onwards
└──────┬───────────────────┘
       │
       ▼
  events.fold(seed, Character::apply)
       │
       ▼
  current Character state
  → stored in RAM cache
  → returned to use case
```

---

## 3. Projection Update

After events are appended to the event store, the `EventBus` fans them out
to registered `EventProjection` handlers. Projections maintain materialised
views for fast reads (Neo4j for search, BigQuery/Athena for analytics).

```
repository.save() completes
   │
   ▼
┌────────────────────┐
│  EventBus          │  publishes each EventEnvelope<DomainEvent>
│                    │  to all registered EventProjection handlers
└───┬────────────────┘
    │
    ├──→ Neo4j projection      (graph: characters ↔ quests ↔ sessions)
    ├──→ BigQuery/Athena        (analytics / AI context queries)
    └──→ (future) other sinks
```

**Port traits in `app`:**

```rust
// Fan-out after a successful store append
trait EventBus<E> {
    async fn publish(&self, envelope: &EventEnvelope<E>) -> Result<(), BusError>;
}

// Downstream read model handler
trait EventProjection<E> {
    async fn project(&self, envelope: &EventEnvelope<E>) -> Result<(), ProjectionError>;
}
```

The production event bus will be GCP Pub/Sub or Kafka. For local dev an
in-process bus calls projections directly without any message broker.
See [ADR-004](adr/ADR-004-hexagonal-architecture.md).

**Idempotency:**

Cloud event listeners (GCS, S3, SQS) deliver events *at least once*.
The projections adapter tracks processed event UUIDs and skips duplicates.
The UUID v7 filename is the deduplication key.

**Projection backends:**

The projections adapter is itself composable. A facade can fan out to
multiple backends simultaneously:

```
EventEnvelope
     │
     ▼
┌─────────────────────────────────────────┐
│         projections facade              │
└───┬──────────┬──────────────┬───────────┘
    ▼          ▼              ▼
 in-memory   SQLite/Postgres  Redis
 (fast read) (durable)        (hot cache)
```

---

## 4. AI Query

The AI queries the current state of a story via MCP. It never reads event
files directly — it receives shaped projections.

```
AI (Claude)
   │
   │  MCP tool call:
   │  "get active characters for story X"
   ▼
┌────────────────────┐
│  adapters/mcp      │  MCP server
│                    │  receives structured tool call
│                    │  calls projection store port
└────────┬───────────┘
         │
         ▼
┌────────────────────┐
│  adapters/         │
│  projections       │  queries materialised view
│                    │  returns shaped projection
└────────┬───────────┘
         │  StoryContext projection
         ▼
┌────────────────────┐
│  adapters/mcp      │  formats as MCP resource
└────────┬───────────┘
         │
         ▼
       AI (Claude)
```

**AI write flow (DM-confirmed):**

The AI can issue state-change commands, but only when the DM confirms.
This routes back through Flow 1 with `source: Ai` on the event envelope:

```
AI drafts command
   │
   ▼
DM confirms via CLI
   │
   ▼
CLI builds Command with CommandSource::Ai
   │
   ▼
→ Flow 1 (DM Command) from app onwards
```

The AI never writes to the event store directly. See
[ADR-007](adr/ADR-007-immutable-event-log.md).

---

## End-to-End Example

DM says: "Kira departed during session 3."

```
1. CLI parses → CharacterCommand::Depart { session_id: session_3_id }

2. app loads Character("Kira"):
   - aggregate-store returns snapshot at event 4
   - event-store returns events 5, 6
   - domain folds → Character { status: Active, ... }

3. domain: character.handle(Depart) → Ok([CharacterEvent::Departed])

4. event-store writes:
   {owner}/
     {story}/
       character/
         {kira_id}/
           01jc3...uuid_v7.json  ← { "type": "CharacterDeparted", "session_id": "..." }

5. aggregate-store saves new snapshot for Kira at event 7

6. notify fires: new file at .../character/{kira_id}/...
   projections updates: ActiveCharacters projection removes Kira
                        SessionTimeline projection records the departure

7. AI queries "active characters":
   mcp → projections → ActiveCharacters view (Kira no longer present)
```

---

## Where New Code Belongs

| Task | Crate |
|---|---|
| New aggregate or event | `domain` |
| New command validation rule | `domain` (in `handle`) |
| New use case / workflow | `app` |
| New port trait | `app` |
| New storage backend | `adapters/*` (new crate) |
| New projection shape | `adapters/projections` |
| New MCP resource or tool | `adapters/mcp` |
| New CLI command | `cli` |
