# ADR-008: Session ID Stamping on State-Change Events

**Status:** Accepted

## Context

The system needs to answer "what happened in session N?" — both for DM review
and for AI context. State-change events (character departures, quest completions,
location discoveries) must be traceable back to the session they occurred in.

## Decision

All state-change events on Character, Quest, Location, and Encounter aggregates
carry an optional `session_id` field. When an event occurs during an active
session, the session ID is stamped on it at the command level.

```rust
CharacterCommand::Depart { session_id: Option<SessionId> }
QuestCommand::Complete { session_id: Option<SessionId> }
LocationCommand::Visit { session_id: Option<SessionId> }
```

Events that occur outside a session (e.g. during Story Creation before session
one) carry `session_id: None`.

## Rationale

Considered **Option B** (Session records its own change log as events), but
rejected it because:
- It requires the Session aggregate to know about Character, Quest, and Location
  types — a coupling violation.
- Answering "what happened in session 3?" via Option B requires reading one
  aggregate. But answering "what is Kira's full history?" requires reading every
  session — worse for aggregate-level queries.

Option A (stamp at source) keeps aggregates independent while enabling both
queries:
- "What happened in session 3?" → scan all aggregates, filter by `session_id`
- "What is Kira's full history?" → replay Character aggregate directly

## Consequences

- `session_id` is added to state-change commands and propagated into events.
- The projection layer handles the cross-aggregate "session timeline" query.
- Events created during Story Creation (before any session exists) have
  `session_id: None` — this is valid and expected.
