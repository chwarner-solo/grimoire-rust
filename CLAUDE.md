# CLAUDE.md

This file is read by Claude Code at the start of every session. It provides
the project context and working conventions needed to contribute effectively.

---

## What This Project Is

Grimoire is an event-sourced TTRPG campaign manager. A DM records the history
of a tabletop campaign — characters, quests, locations, sessions, encounters —
as an immutable event log. An AI (via MCP) reads the current story state to
assist with session prep, NPC dialogue, and narrative continuity.

Key facts to always keep in mind:
- The event log is **immutable**. Corrections are compensating events.
- The AI can only write **state-change commands**, never free-form edits.
- Story creation is **incremental** — no completion required before play begins.
- **Five aggregates**: Story, Character, Quest, Location, Session, Encounter.

See `README.md` for the full picture. See `docs/` for decisions and domain detail.

---

## Architecture Rules

These are hard constraints. Do not violate them.

```
cli / mcp  →  app  →  domain
adapters/* →  app  →  domain
```

| Rule | Why |
|---|---|
| `domain` has zero infrastructure dependencies | Domain logic must be testable in isolation |
| `app` imports only from `domain` | Keeps use cases infrastructure-agnostic |
| Adapters never import from other adapters | Each adapter is independently swappable |
| Port traits live in `app`, not `domain` | Ports are application concerns |

When adding new code, the first question is always: **which layer does this belong to?**
See `docs/data-flow.md` for the definitive answer.

---

## Domain Code Conventions

### Aggregates require two impl blocks

```rust
impl Apply<FooEvent> for Foo { ... }   // state reconstruction — infallible
impl Aggregate for Foo { ... }         // command handling — can return Err
```

### `apply` is always infallible

`apply(self, event) -> Self` — never `Result`. Events are facts that already
happened. If applying an event can fail, the `handle` validation was wrong.

### `Default` uses `Uuid::nil()` for IDs

```rust
impl Default for FooId {
    fn default() -> Self { Self(Uuid::nil()) }  // nil = unset, not a real ID
}
```

Not `Self::new()` — default is a fold seed, not a real entity.

### State-change commands carry `Option<SessionId>`

Any command that changes aggregate state should carry `session_id: Option<SessionId>`
so the event can be stamped with which session it occurred in.

### Tests use the fold pattern

```rust
fn make_foo() -> Foo {
    let events = Foo::default()
        .handle(FooCommand::Create { ... })
        .unwrap();
    events.into_iter().fold(Foo::default(), Foo::apply)
}
```

Never test by constructing aggregate state directly. Always go through
`handle` + `apply` — this tests both sides of the pattern.

### Match arms are exhaustive by design

When adding a new event variant, the compiler will force you to handle it in
every `apply` and `match`. This is intentional. Do not use `_ =>` catch-alls
in domain code — handle every variant explicitly.

---

## Documentation Conventions

| Type of change | Documentation required |
|---|---|
| New architectural decision | New ADR in `docs/adr/` |
| New aggregate or major domain change | Update `docs/domain/` reference |
| New user-facing feature or workflow | Update or add `docs/journeys/` |
| Infrastructure or adapter choice | New or updated ADR |

Before starting significant new work, check `docs/` to see if the decision
has already been made. The ADRs are the source of truth for *why* things are
the way they are.

---

## How We Work Together

### Discuss before implementing

For anything non-trivial — a new aggregate, a new adapter, a structural
change — discuss the approach first. Describe the plan in a sentence or two
and confirm before writing code. Do not implement speculatively.

### Domain first, then app, then adapters

When building a new feature, the sequence is always:
1. Define the domain events and commands
2. Implement and test the aggregate
3. Define the port traits in `app`
4. Implement the adapters

Do not reach into `app` or adapter concerns while domain work is in progress.

### Write the journey or ADR before writing code for significant features

If a feature involves a new user workflow or an architectural decision,
write the document first. The document clarifies thinking and surfaces gaps
before they become bugs. Code is easier to write once the domain is clear.

### Keep it minimal

Do not add features, abstractions, or error handling beyond what the current
task requires. Grimoire has a clear roadmap — things not on it can wait.
Three similar lines of code is better than a premature abstraction.

### When in doubt, ask

If a requirement is ambiguous or a design decision has multiple valid
approaches, surface the question rather than making a silent choice.
A two-sentence clarification is faster than refactoring after the fact.

---

## Current Implementation Status

| Crate | Status | Notes |
|---|---|---|
| `domain` | Built | All 5 aggregates. Session and Encounter not yet coded. |
| `app` | Stub | Port traits and use cases not yet written |
| `adapters/*` | Stub | No adapters implemented yet |
| `cli` | Stub | Entry point only |

**Next up in domain:** Session aggregate, then Encounter aggregate.
Session needs `QuestKind`, `character_id` on Quest, and `session_id` on
state-change commands across all aggregates.

**After domain:** Define port traits in `app`, then implement `adapters/event-store`.

---

## Running the Project

```bash
cargo build          # build everything
cargo test           # run all tests
cargo test -p domain # domain tests only (24 tests, all should pass)
```

If tests fail on a clean checkout, something is wrong — the domain is the
stable layer and its tests should always be green.
