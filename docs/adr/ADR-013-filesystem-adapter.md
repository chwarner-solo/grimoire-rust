# ADR-013: Filesystem Adapter as the v1 Storage Backend

**Status:** Accepted

## Context

The application needs a working storage backend to be runnable. The
production target is cloud storage (GCS or S3) for the event store and
a document database (Firestore, DynamoDB, or MongoDB) for snapshots.
However, building those adapters before the core application is working
would add infrastructure complexity before the domain is validated.

## Decision

Implement `FsAggregateRepository` as the v1 storage adapter. It satisfies
`AggregateRepository<A>` using only the local filesystem:

**Event store** (`FsEventStore<E>`):
- One JSONL file per aggregate instance: `{base_dir}/events/{id}.jsonl`
- Each line is a serialised `EventEnvelope<E>` with `aggregate_id`,
  `sequence`, `occurred_at`, and `payload`
- Appends are O(1); reads scan the file and filter by sequence

**Snapshot store** (`FsSnapshotStore<A>`):
- One JSON file per aggregate instance: `{base_dir}/snapshots/{id}.json`
- Stores `{ "state": <serialised A>, "sequence": N }`
- Written on every `save` call to keep cold-start cost bounded

**Directory layout per aggregate type:**
```
data/
  characters/
    events/{character_id}.jsonl
    snapshots/{character_id}.json
  stories/
    events/{story_id}.jsonl
    snapshots/{story_id}.json
  ...
```

All domain types derive `serde::Serialize` and `serde::Deserialize` so
they can be used directly as snapshot and event payloads.

## Consequences

- Zero infrastructure required to run locally or in a Docker container.
- Each aggregate's full history is a human-readable file — useful for
  debugging and development.
- Concurrent writes to the same aggregate from multiple processes are
  unsafe (no locking). Acceptable for a single-user v1; addressed in
  production by Kafka's partitioned log or a database transaction.
- Migrating to GCS/S3 + Firestore means implementing `EventStore` and
  `SnapshotStore` for those backends — `FsAggregateRepository` is replaced
  at the composition root in `cli/main.rs`.
- The `DATA_DIR` environment variable controls the root path, making the
  filesystem location explicit and Docker-mountable as a volume.
