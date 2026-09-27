# ADR-005: Apply and Aggregate Traits for Domain Aggregates

**Status:** Accepted

## Context

Each domain aggregate (Character, Quest, Location, Session, Encounter) needs
a consistent pattern for processing commands and reconstructing state from
events. The pattern must be reusable across both aggregates (which handle
commands) and projections (which only need state reconstruction).

## Decision

Two traits defined in `domain::traits`:

```rust
pub trait Apply<Event, Output = Self>: Sized {
    fn apply(self, event: Event) -> Output;
}

pub trait Aggregate: Sized + Apply<Self::Event> {
    type Command;
    type Event;
    type Error;

    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}
```

**`Apply<Event, Output = Self>`** — pure state reconstruction. Consumes `self`,
returns new state. Infallible — events are facts, applying them cannot fail.

**`Aggregate`** — extends `Apply` with the command side. `handle` borrows
`self` (reads state to validate), returns events or an error.

Each aggregate has associated types binding it to its specific command, event,
and error enums. Replay is a fold:

```rust
let state = events.into_iter().fold(Aggregate::default(), Aggregate::apply);
```

`Apply<Event, Output = Self>` with a differing `Output` covers projections that
fold multiple event types into a single read model.

## Consequences

- Command validation (`handle`) and state reconstruction (`apply`) are
  explicitly separated. `apply` is always infallible.
- Two `impl` blocks are required per aggregate: one for `Apply`, one for
  `Aggregate`. This is intentional — projections implement only `Apply`.
- The compiler enforces exhaustive event handling in every `match`. Adding
  a new event variant without handling it in `apply` is a compile error.
- `Default` is required on all aggregates to provide the fold seed.
  Default state uses `Uuid::nil()` for IDs — clearly "unset", not a real ID.
