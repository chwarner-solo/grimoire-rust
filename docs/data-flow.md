# Data Flow

This document traces how data moves through Grimoire — from a DM issuing a
command to an AI receiving a projection. Understanding this flow tells you
where new code belongs and how the layers stay decoupled.

---

## The Four Flows

| Flow | Trigger | Path |
|---|---|---|
| [1. DM Command](#1-dm-command) | DM issues a command via CLI | CLI → app → domain → event-store → projections |
| [2. Aggregate Load](#2-aggregate-load) | app needs current aggregate state | app → aggregate-store + event-store → domain |
| [3. Projection Update](#3-projection-update) | New event file written to disk | event-store → notify → projections |
| [4. AI Query](#4-ai-query) | AI requests story state via MCP | AI → mcp → projections |

---

## 1. DM Command

The most common flow. The DM issues an instruction — a character departs,
a quest is completed, a session closes.

```
DM input
   │
   ▼
┌─────────┐
│   cli   │  parses input, builds Command struct
└────┬────┘
     │  CharacterCommand::Depart { session_id }
     ▼
┌─────────┐
│   app   │  command handler
│         │  1. loads current aggregate state  ──→ see Flow 2
│         │  2. calls aggregate.handle(command)
│         │  3. receives Vec<CharacterEvent>
└────┬────┘
     │  Vec<CharacterEvent>
     ▼
┌────────────────────┐
│  adapters/         │
│  event-store       │  serialises each event to JSON
│                    │  writes to:
│                    │  {owner}/{story}/character/{id}/{uuid_v7}.json
└────────────────────┘
     │  new file on disk
     ▼
  (triggers Flow 3 — projection update)
```

**What the app layer does:**

The `app` command handler is the orchestrator. It never touches the file
system directly — it calls port traits defined in `app` and implemented by
the adapters:

```rust
// port defined in app, implemented by adapters/event-store
trait EventStore {
    fn append(&self, stream: StreamId, events: Vec<SerializedEvent>);
    fn load(&self, stream: StreamId) -> Vec<SerializedEvent>;
}

// port defined in app, implemented by adapters/aggregate-store
trait AggregateStore {
    fn load_snapshot<A: Aggregate>(&self, id: AggregateId) -> Option<(A, SequenceNumber)>;
    fn save_snapshot<A: Aggregate>(&self, id: AggregateId, aggregate: A, seq: SequenceNumber);
}
```

**Command source is stamped on the event envelope:**

```json
{
  "type": "CharacterDeparted",
  "at": "2026-09-27T14:32:00Z",
  "source": "Dm",
  "session_id": "01j4z...",
  "data": {}
}
```

---

## 2. Aggregate Load

Before the `app` layer can call `handle`, it needs the aggregate's current
state. Replaying all events from the beginning every time would be expensive
for long-running campaigns. The aggregate store provides a snapshot shortcut.

```
app needs: current state of Character(id)
   │
   ▼
┌─────────────────┐
│ aggregate-store │  do we have a snapshot?
└────────┬────────┘
         │
    ┌────┴─────┐
    │          │
  yes          no
    │          │
    ▼          ▼
snapshot    Character::default()
at seq N    at seq 0
    │          │
    └────┬─────┘
         │  seed state + last known sequence number
         ▼
┌──────────────┐
│ event-store  │  load events after seq N
│              │  from: {owner}/{story}/character/{id}/
│              │  sorted by UUID v7 filename (time order)
└──────┬───────┘
       │  Vec<CharacterEvent> from seq N+1 onwards
       ▼
┌───────────────────────────────────┐
│  domain                           │
│                                   │
│  events.fold(seed, Character::apply)  │
│                                   │
│  → current Character state        │
└───────────────────────────────────┘
       │
       ▼
  app calls handle(command)
       │
       ▼
  new snapshot saved to aggregate-store
```

**Why this matters:**

The aggregate store is not just a performance optimisation — it is also
the **DM working view**. When a DM opens a character to issue a command,
the aggregate store provides the fully-reconstituted state they are
editing against. See [ADR-005](adr/ADR-005-aggregate-traits.md).

---

## 3. Projection Update

Projections are materialised views built from events. They are kept up to
date by an event listener that watches for new files on disk.

```
new file written:
{owner}/{story}/character/{id}/{uuid_v7}.json
   │
   ▼
┌────────────────────┐
│  notify watcher    │  OS-level file system event
│  (inotify / FSEvents)  path tells us aggregate type + id
│                    │  before the file is opened
└────────┬───────────┘
         │  FileCreated { path }
         ▼
┌────────────────────┐
│  adapters/         │
│  projections       │  reads + deserialises event from file
│                    │  routes to the right projection handler
│                    │  updates materialised view(s)
└────────────────────┘
```

**The event listener port:**

```rust
// port defined in app
trait EventListener {
    fn listen(&self) -> impl Stream<Item = EventEnvelope>;
}

// implemented by:
//   adapters/event-store  → notify watching local directories
//   future: adapters/gcs  → GCP Pub/Sub on bucket writes
//   future: adapters/s3   → S3 Event Notifications via SQS
```

Switching from local files to cloud storage means replacing the
`EventListener` implementation — nothing else changes.
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
