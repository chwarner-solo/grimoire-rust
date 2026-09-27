# Journey: Encounter

## Overview

An encounter is a discrete moment of direct engagement within a session — the
point where the story forces a decision, a conflict, or a challenge. Encounters
are the texture of play: they can be planned carefully in Prep, or arise
spontaneously at the table.

Not all planned encounters get run. The story moves, circumstances change, and
some encounters become impossible before they happen. This history — of what was
planned and what was not — is as meaningful to the AI as what actually occurred.

---

## Actor

**The DM** creates, activates, and resolves encounters. Players participate
in the encounter itself but their actions are recorded through the DM.

---

## Encounter Status

```
Planned → Active → Resolved
        → Abandoned
        → DeadEnded
```

| Status | Description |
|---|---|
| `Planned` | Created in Prep, not yet run |
| `Active` | Currently being played out at the table |
| `Resolved` | Ran to completion — has an outcome |
| `Abandoned` | DM retired it — too stale, no longer fits the story |
| `DeadEnded` | Became impossible — a prerequisite no longer exists |

**Random encounters** skip `Planned` entirely — they are created `Active`
when they arise spontaneously during play.

---

## Encounter Kinds

| Kind | Description |
|---|---|
| `Combat` | A fight — ambush, creature, hostile faction |
| `Diplomacy` | A social challenge — negotiation, interrogation, persuasion |
| `Puzzle` | An obstacle requiring thought — trap, lock, ritual, mystery |
| `Story` | A narrative beat — revelation, NPC scene, world event |
| `Random` | Unplanned, arises spontaneously during play |

An encounter has one kind. The nuance of how it played out lives in the
outcome description.

---

## Phase 1: Prep — Creating a Planned Encounter

The DM anticipates a situation the party will likely face this session and
captures it before the table convenes. A planned encounter needs:

- A **kind** — what type of challenge is this?
- A **description** — enough context for the DM to run it
- Optionally: assigned characters (NPCs involved), a location

> *"Diplomacy encounter with Lord Vasek at his estate. He knows about Kira's
> past. He'll offer her a deal she shouldn't take."*

Encounters are created within a session but are not consumed by it. A `Planned`
encounter that doesn't get run remains **available** — it stays in the DM's
encounter pool and can be pulled into any future session's Prep. The DM does
not need to explicitly carry it forward; it is simply still there.

**Commands issued:**
```
EncounterCommand::Create {
    session_id,
    story_id,
    kind: EncounterKind::Diplomacy,
    description,
}
```

---

## Phase 2: Play — Activating and Resolving

### Activating a Planned Encounter

When the DM runs a prepared encounter, they activate it. This marks the moment
it moves from preparation into the actual record of the session.

```
EncounterCommand::Activate
```

### Creating a Random Encounter

Some encounters are unplanned — they arise from a random table, an improvised
moment, or a player action that triggers something unexpected. These are created
directly as `Active`.

```
EncounterCommand::Create {
    session_id,
    story_id,
    kind: EncounterKind::Random,
    description,
}
```

### Resolving an Encounter

Once the encounter concludes, the DM writes the outcome — what happened, how
it ended, what it means for the story going forward.

The outcome is the most valuable part of an encounter for the AI. It captures
not just the result but the narrative consequence.

> *"Kira refused Vasek's deal. He smiled. The party now has a powerful enemy
> who knows their faces."*

```
EncounterCommand::Resolve { outcome: Prose }
```

---

## Phase 3: Terminal States Without Resolution

### Abandoned

The DM deliberately retires an encounter. It no longer fits the story — the
timing has passed, the narrative moved in a different direction, or the encounter
would feel forced if run now.

Abandoned encounters do not carry forward. They are not dead-ended by
circumstance — the DM made a choice.

```
EncounterCommand::Abandon
```

### DeadEnded

The encounter became **impossible** because something it depended on no longer
exists. This is not a DM choice — it is a consequence of how the story unfolded.

> A combat encounter planned around a crime boss who was killed in the previous
> session. The encounter cannot be run; the boss is gone.

> A diplomacy encounter at a location that was destroyed last session.

DeadEnded encounters require a `reason` — a brief note explaining what
prerequisite was lost. This note becomes AI context: it tells the story of how
the world's state foreclosed certain possibilities.

```
EncounterCommand::DeadEnd { reason: Prose }
```

**DeadEnded encounters are valuable history.** The AI can reason: "this was
planned but became impossible — here is why." That tells the AI something about
what changed in the world and how quickly.

---

## What the AI Gets From Encounters

Encounters give the AI the most granular view of what happens session-to-session.

| Encounter state | AI value |
|---|---|
| `Resolved` + outcome | Direct narrative history — what happened and what it meant |
| `DeadEnded` + reason | World-change signal — what no longer exists |
| `Abandoned` | Low signal — DM choice, not story-driven |
| `Planned` (current session) | DM intent — what the DM is setting up |

When the DM asks the AI to help prep the next session, the AI can:
- Reference resolved encounters as established history
- Note dead-ended encounters as indicators of world change
- Identify planned encounters that were never run and may still be relevant

---

## Encounter and Quest Relationship

Some encounters are the direct expression of a quest in play. A combat encounter
might be the climax of the Main Quest. A puzzle might be the obstacle blocking a
Side Quest's completion.

Encounters do not formally belong to a quest, but the DM's description and
outcome naturally reference quests by name. The AI uses this narrative connection
without requiring a hard foreign key between the two aggregates.

---

## Available Encounters

An encounter is **available** when it is `Planned` and has not reached a
terminal state — regardless of which session it was created in. This is a
projection, not a status.

The DM sees available encounters as a pool to draw from when prepping any
future session. An encounter remains available until:
- It is activated and resolved
- The DM abandons it
- Its prerequisites are gone and it is dead-ended

**Session close does not gate on encounters.** A session can close with
`Planned` encounters still in the pool — they simply remain available. Only
`Active` encounters need attention: an encounter in mid-run should be resolved
or abandoned before the session closes, but this is a DM discipline concern,
not a system constraint.

---

## Definition of Done

An encounter is complete when it reaches any terminal state:
`Resolved`, `Abandoned`, or `DeadEnded`.
