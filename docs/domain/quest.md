# Aggregate: Quest

A quest is a story line — a goal, challenge, or obligation that drives the
narrative forward. Quests vary in scope and purpose: some are the spine of
the entire campaign; others are personal to one character; others are
practical tasks the party takes on.

## Status

```
Active → Completed
       → Failed
       → Abandoned
```

| Status | Meaning |
|---|---|
| `Active` | In progress — the party is pursuing this |
| `Completed` | Successfully resolved — the goal was achieved |
| `Failed` | Definitively failed — the goal is no longer achievable |
| `Abandoned` | The party gave up or moved on — may still be possible but no longer pursued |

## Kind

| Kind | Meaning |
|---|---|
| `Main` | The spine of the campaign — completing this advances the primary story arc |
| `Side` | Optional enrichment — adds texture to the world, no story dependency |
| `Personal` | Tied to a specific character's arc — requires a `character_id` |
| `Crafting` | Resource gathering, item creation, building — practical party goals |

A `Personal` quest must reference the character it belongs to. All other kinds
have `character_id: None`.

---

## Commands

### `Create { story_id, owner, name, description, kind }`

Introduces a new story line. The `name` must be non-empty. The `description`
provides enough context for the DM and AI to understand what this quest is
about and why it matters. Quests begin `Active`.

For `kind: Personal`, a `character_id` must be provided.

**Emits:** `Created`
**Errors:** `NameError::Empty` if name is blank, `MissingCharacter` if kind
is Personal and no character_id provided

---

### `Complete`

The quest was successfully resolved. Only `Active` quests can be completed.

**Emits:** `Completed`
**Errors:** `AlreadyComplete` if already completed, `NotActive` for any other
terminal state

---

### `Fail`

The quest has definitively failed — the goal is no longer achievable. The
merchant is dead; the artifact is destroyed; the window has closed.

Only `Active` quests can fail.

**Emits:** `Failed`
**Errors:** `NotActive`

---

### `Abandon`

The party chose to stop pursuing this quest. Unlike `Fail`, abandon is a
decision, not a defeat — the quest may still be theoretically achievable.

Only `Active` quests can be abandoned.

**Emits:** `Abandoned`
**Errors:** `NotActive`

---

### `Rename { name }`

Changes the quest's name. Use when the DM refines how they're framing a
story line, or when the true nature of the quest shifts.

**Emits:** `Renamed`
**Errors:** `NameError::Empty`

---

### `UpdateDescription { description }`

Revises or extends the quest description. Use as the quest evolves, new
information comes to light, or the DM wants to sharpen the AI's context.

**Emits:** `DescriptionUpdated`
**Errors:** none

---

## Events

### `Created { id, story_id, owner, name, description, kind, character_id }`

A new story line enters the world. The `character_id` is `Some` only for
`Personal` quests.

---

### `Completed`

The quest was successfully resolved. The story moved forward because of it.

---

### `Failed`

The quest ended in failure. The goal was not achieved and cannot be pursued
further through this story line.

---

### `Abandoned`

The party stopped pursuing this quest. The door is not necessarily closed —
but it is no longer active.

---

### `Renamed { name }`

The quest's framing or title changed.

---

### `DescriptionUpdated { description }`

The quest's description was revised. Prior versions are preserved in the
event log.
