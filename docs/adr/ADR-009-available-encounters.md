# ADR-009: Available Encounters as a Projection, Not a Status

**Status:** Accepted

## Context

Encounters are created within a session's Prep phase, but not all planned
encounters are run during that session. The DM needs to see unused encounters
across session boundaries without the system forcing them to explicitly carry
each one forward.

## Decision

**"Available" is a projection query, not an encounter status.**

An encounter is available when its status is `Planned` — regardless of which
session created it or whether that session is now closed. The DM's encounter
pool is simply all `Planned` encounters for a given story.

```
Planned → Active → Resolved
        → Abandoned
        → DeadEnded { reason }
```

| Status | Meaning |
|---|---|
| `Planned` | Created, not yet run — available for any future session |
| `Active` | Currently being played out |
| `Resolved` | Ran to completion, has an outcome |
| `Abandoned` | DM explicitly retired it — no longer fits the story |
| `DeadEnded` | Impossible — a prerequisite (character, location) no longer exists |

Session close is **not gated** on encounter completion. A session may close
with `Planned` encounters still in the pool.

## Rationale

Gating session close on encounter completion would create friction for the
DM — in practice, most sessions end with unused prepared content. The
encounters carry forward naturally without any explicit action.

`DeadEnded` (with a `reason`) is distinct from `Abandoned` (a DM choice).
A dead-ended encounter tells the AI that the world changed in a way that
foreclosed a possibility — valuable historical context.

## Consequences

- The projection query `available_encounters(story_id)` filters for
  `status == Planned` across all sessions for that story.
- Encounters have a longer lifetime than a session — they belong to the story,
  not to any single session.
- The `session_id` on an encounter records where it was *created*, not where
  it was *run* (which may be a different session, or never).
