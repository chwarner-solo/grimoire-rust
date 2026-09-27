# Aggregate: Character

A character is anyone who inhabits the story world — a Player Character
controlled by a player, or a Non-Player Character controlled by the DM.
Characters have a name, a backstory, a kind, and a status that reflects
their current standing in the story.

## Status

```
Active → Departed
       → Deceased
```

| Status | Meaning |
|---|---|
| `Active` | Present in the story, available to act |
| `Departed` | Left the story — moved away, retired, captured. Still alive but gone from active play |
| `Deceased` | Dead. Permanently removed from active play |

Note: a `Departed` character can be killed (`Deceased`) but a `Deceased`
character cannot be further changed. Both are terminal relative to normal play.

## Kind

| Kind | Meaning |
|---|---|
| `PlayerCharacter` | Controlled by a player. Created from player-provided information |
| `NonPlayer` | Controlled by the DM. NPCs, antagonists, allies, bystanders |

---

## Commands

### `Create { story_id, owner, name, backstory, kind }`

Introduces a character to the story. The `name` must be non-empty. The
`backstory` gives the character narrative grounding — even a sentence is enough
to start. Characters begin `Active`.

**Emits:** `Created`
**Errors:** `NameError::Empty` if name is blank

---

### `Depart`

Removes a character from active play — they leave the party, are captured, go
into hiding, or otherwise exit the story without dying. A departed character's
history remains intact and they may reappear.

Only `Active` characters can depart.

**Emits:** `Departed`
**Errors:** `NotActive` if character is not currently active

---

### `Kill`

Marks a character as deceased. Permanent. A character can be killed from
either `Active` or `Departed` status — death can reach those who have already
left the story.

**Emits:** `Deceased`
**Errors:** `AlreadyDeceased` if character is already dead

---

### `Rename { name }`

Changes a character's name. Handles retcons, aliases revealed as true names,
or corrections. No status restrictions.

**Emits:** `Renamed`
**Errors:** `NameError::Empty` if name is blank

---

### `UpdateBackstory { backstory }`

Rewrites or extends a character's backstory. Use this as the DM learns more
about who a character is, or as their history is revealed through play.

**Emits:** `BackstoryUpdated`
**Errors:** none

---

## Events

### `Created { id, story_id, owner, name, backstory, kind }`

A character joins the world. The `id` is generated at command time and carried
in the event for stable replay identity.

---

### `Departed`

The character left active play. They still exist in the world — just not at
the table. The story_id and all other fields are unchanged.

---

### `Deceased`

The character died. This is a terminal state. Their history, backstory, and
the circumstances of their death are part of the story record.

---

### `Renamed { name }`

The character now has a different name. Prior events still reference the old
name — the rename does not retroactively alter history.

---

### `BackstoryUpdated { backstory }`

The character's backstory was revised. The new backstory replaces the previous
one in the current projection. The event log preserves all prior versions.
