# ADR-001: Event Sourcing as the Persistence Model

**Status:** Accepted

## Context

Grimoire needs to persist the state of a TTRPG campaign — characters, quests,
locations, sessions, and encounters. The data evolves over time as the campaign
progresses, and the history of how things changed is as valuable as the current
state. An AI will eventually use this data to assist DMs and players.

## Decision

Use **event sourcing** as the primary persistence model. State is never stored
directly. Instead, domain events are the source of truth. Current state is
derived by replaying events in order.

Each aggregate (Character, Quest, Location, Session, Encounter) maintains its
own event stream. State is reconstructed by folding events over an initial
default value:

```rust
let state = events.into_iter().fold(Aggregate::default(), Aggregate::apply);
```

## Consequences

- The full history of every aggregate is preserved — nothing is lost when state
  changes.
- Corrections are made through compensating events, not by editing history.
- The event log is the authoritative record; projections are derived from it.
- Replay must be efficient — aggregate store snapshots address long event streams.
- All domain logic splits into two sides: `handle` (validate commands, emit
  events) and `apply` (reconstruct state from events).
