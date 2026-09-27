# Aggregate: Encounter

An encounter is a discrete moment of direct engagement within a session — a
fight, a negotiation, a puzzle, a revelation. Encounters can be planned in
Prep or arise spontaneously during Play. Not all planned encounters are run;
those that aren't remain available for future sessions.

## Status

```
Planned → Active → Resolved
        → Abandoned
        → DeadEnded
```

| Status | Meaning |
|---|---|
| `Planned` | Created in Prep, not yet run. Available for any future session |
| `Active` | Currently being played out at the table |
| `Resolved` | Ran to completion — has an outcome written |
| `Abandoned` | DM retired it — no longer fits the story |
| `DeadEnded` | Impossible — a prerequisite no longer exists |

**Random encounters** begin `Active` — they are created at the table with
no prior Planned state.

**Available** is not a status. It is a projection: all `Planned` encounters
for a story, regardless of which session created them.

## Kind

| Kind | Meaning |
|---|---|
| `Combat` | A fight — ambush, creature, hostile faction, duel |
| `Diplomacy` | A social challenge — negotiation, interrogation, persuasion, deception |
| `Puzzle` | An obstacle requiring thought — trap, lock, ritual, mystery |
| `Story` | A narrative beat — revelation, NPC scene, dream, world event |
| `Random` | Unplanned, generated at the table — kind refined when created |

---

## Commands

### `Plan { session_id, story_id, kind, description }`

Creates a planned encounter in Prep. The `description` should give the DM
enough context to run it — the setup, the key NPC or obstacle, the stakes.
The encounter begins `Planned`.

**Emits:** `Planned`
**Errors:** none

---

### `CreateRandom { session_id, story_id, kind, description }`

Creates an encounter that arises spontaneously during Play. Begins `Active`
immediately — there is no Planned state for random encounters.

**Emits:** `Created` (with `Active` status)
**Errors:** none

---

### `Activate`

Moves a `Planned` encounter to `Active` — the DM is running it now. Only
applies to encounters in the `Planned` state.

**Emits:** `Activated`
**Errors:** `NotPlanned`

---

### `Resolve { outcome }`

Closes an `Active` encounter with an outcome — the DM's description of what
happened, how it ended, and what it means for the story.

The outcome is the most valuable field for the AI. It captures narrative
consequence, not just mechanical result.

**Emits:** `Resolved`
**Errors:** `NotActive`

---

### `Abandon`

The DM retires a `Planned` encounter. It no longer fits the story — the
timing has passed, the narrative moved on, or it would feel forced. This
is a deliberate DM choice, not a consequence of world events.

**Emits:** `Abandoned`
**Errors:** `NotPlanned` (only planned encounters can be abandoned this way;
active encounters should be resolved)

---

### `DeadEnd { reason }`

A `Planned` encounter became impossible because a prerequisite — a character,
a location, a faction — no longer exists. The `reason` briefly explains what
was lost.

> "Vasek is dead. The diplomacy encounter built around him cannot be run."

This is distinct from Abandon: the DM did not choose this — circumstances did.
Dead-ended encounters are valuable AI context: they signal how the world changed.

**Emits:** `DeadEnded`
**Errors:** `NotPlanned`

---

## Events

### `Planned { id, session_id, story_id, kind, description }`

An encounter has been prepared. It is now in the available pool.

---

### `Created { id, session_id, story_id, kind, description }`

A random encounter was created directly as Active during play.

---

### `Activated`

A planned encounter moved to Active — the DM began running it.

---

### `Resolved { outcome }`

The encounter concluded. The `outcome` is the narrative record of what
happened. This is prime AI context for understanding what the party faced
and how it shaped the story.

---

### `Abandoned`

The DM retired this encounter. It will not be run. No narrative consequence.

---

### `DeadEnded { reason }`

The encounter became impossible. The `reason` records what prerequisite
was lost. This tells the AI something meaningful changed in the world.
