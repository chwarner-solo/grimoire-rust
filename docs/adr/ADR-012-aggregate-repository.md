# ADR-012: Three-Layer Aggregate Repository

**Status:** Accepted

## Context

Use cases need to load aggregate state before handling a command. The naive
approach — replaying all events from the event store on every command — is
correct but expensive for aggregates with long histories. A caching strategy
is needed, but the use cases should not know about it.

Additionally, the storage backend will vary by environment (filesystem for
local dev, GCS/S3 for production, Kafka for streaming). The use cases must
remain infrastructure-agnostic.

## Decision

A single `AggregateRepository<A>` port trait is the **only interface use
cases interact with**:

```rust
trait AggregateRepository<A: Aggregate> {
    async fn load(&self, id: A::Id) -> Result<A, RepositoryError>;
    async fn save(&self, id: A::Id, events: Vec<A::Event>) -> Result<(), RepositoryError>;
}
```

Internally, the concrete implementation is a three-layer decorator:

```
load(id):
  L1 RAM cache         → hit: return immediately
  L2 SnapshotStore     → hit: load snapshot + events after snapshot sequence
  L3 EventStore        → miss: replay all events from scratch

save(id, events):
  L3 EventStore        → append events (always)
  L2 SnapshotStore     → write snapshot at new sequence (always)
  L1 RAM cache         → update with new state
```

Each layer has its own port trait (`SnapshotStore<A>`, `EventStore<E>`) so
the implementations are independently swappable.

The RAM cache stores `(A, Instant)` pairs, making TTL-based eviction (or
an LRU cache via `moka`) a future drop-in replacement without changing the
interface. See ADR-013 for the filesystem implementation.

## Consequences

- Use cases are two lines: `load` → `handle` → `save`. No storage concerns.
- Cold starts require one snapshot read + incremental event replay.
- Hot requests (same aggregate loaded twice in one process lifetime) skip
  all I/O after the first load.
- The snapshot is written on every `save`, keeping cold-start cost bounded
  regardless of aggregate history length.
- Deploying a Kafka or MongoDB adapter means implementing `EventStore` and
  `SnapshotStore` — use cases and services are untouched.
