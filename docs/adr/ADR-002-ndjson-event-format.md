# ADR-002: JSON as the Event Serialization Format

**Status:** Accepted

## Context

Events must be serialized to disk. Candidates considered: Protocol Buffers,
Apache Avro, and JSON. The primary eventual consumer of this data is an AI
(via MCP), and the system is file-based rather than a message queue.

## Decision

Use **JSON** (one event per file) as the serialization format.

Each event file contains:
```json
{
  "type": "CharacterDeparted",
  "at": "2026-09-27T14:32:00Z",
  "data": { ... }
}
```

## Rationale

- **AI-native format.** LLMs work with text natively. JSON requires no
  transformation to feed into a prompt or MCP context window.
- **Human-readable.** Events can be inspected and debugged without tooling.
- **Rust ecosystem.** `serde_json` is mature, fast, and first-class.
- **No code generation.** Protobuf and Avro require `.proto` files and build
  scripts. JSON schema is enforced by Rust's type system at write time.
- **One file per event.** No append locking, no NDJSON stream management.
  Write and close.

## Consequences

- Schema enforcement is handled by Rust's type system, not the wire format.
- File sizes are larger than binary formats — acceptable at this scale.
- The aggregate store snapshots prevent replaying large event histories.
- Future migration to a binary format (if needed at scale) would require an
  event migration tool, but the aggregate store makes this a background concern.
