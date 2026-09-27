# Domain Overview

Grimoire models a TTRPG campaign as five aggregates, all scoped to a Story.
Each aggregate has its own event stream. State is reconstructed by replaying
events in order — nothing is stored directly.

## Aggregates

| Aggregate | Belongs To | Purpose |
|---|---|---|
| `Story` | Owner (DM) | The root — title, premise, ownership |
| `Character` | Story | PCs and NPCs — who inhabits the world |
| `Quest` | Story | Story lines — what drives the narrative |
| `Location` | Story | Places — where the story happens |
| `Session` | Story | A sitting of play — when the story moves |
| `Encounter` | Session + Story | A discrete moment of engagement within a session |

## Shared Concepts

**`UserId`** — identifies the owner of a story or the author of a command.
Initially the DM. Eventually players will have their own IDs.

**`StoryId`** — the shared anchor. Every aggregate carries a `StoryId` so
projections can assemble a complete story picture.

**`SessionId`** — stamped on state-change events to answer "what happened
in session N?" See ADR-008.

**`Prose`** — a free-text field with no validation constraints. Descriptions,
backstories, summaries, outcomes. The AI reads these heavily.

**`Name`** — a validated non-empty string. Used for character names, quest
names, location names.

## Lifecycle Pattern

Every aggregate follows the same pattern:

```
Command → handle() → Vec<Event>  (validate + produce)
Event   → apply()  → Self        (reconstruct state)
```

`handle` reads current state, validates the command, and returns events.
`apply` is infallible — events are facts, applying them cannot fail.

## Command Source

Every command carries a source identifying who issued it:

| Source | Meaning |
|---|---|
| `Dm` | Issued by the DM directly |
| `Player(UserId)` | Issued by a specific player |
| `Ai` | Issued by the AI on DM request |

This is stamped on the event envelope, not the domain event itself.
See ADR-007 for AI write constraints.
