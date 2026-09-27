# Aggregate: Location

A location is a place in the story world — a city, a dungeon, a crossroads,
a tavern. Locations anchor the narrative geographically and give the AI
context for where events are happening. They progress from known (heard of)
to visited (been there) and may eventually be destroyed.

## Status

```
Known → Visited
      → Destroyed
Visited → Destroyed
```

| Status | Meaning |
|---|---|
| `Known` | The party knows this place exists but has not been there |
| `Visited` | The party has been here — it is part of their lived experience |
| `Destroyed` | The location no longer exists in its prior form — burned, collapsed, sunk |

A location can be destroyed from either `Known` or `Visited` status.
A destroyed location cannot be visited or destroyed again.

---

## Commands

### `Create { story_id, owner, name, description }`

Introduces a location to the story. The `name` must be non-empty. The
`description` should give enough detail for the DM to render it and the AI
to reference it meaningfully. New locations begin as `Known`.

**Emits:** `Created`
**Errors:** `NameError::Empty` if name is blank

---

### `Visit`

Records that the party has been to this location. Can be applied from
`Known` or again from `Visited` (visiting a second time is valid and
the state remains `Visited`). Cannot be applied to a destroyed location.

**Emits:** `Visited`
**Errors:** `AlreadyDestroyed`

---

### `Destroy`

The location is permanently altered or eliminated — burned to the ground,
swallowed by the earth, sunk into the sea. Destroyed locations cannot be
visited or destroyed again.

**Emits:** `Destroyed`
**Errors:** `AlreadyDestroyed`

---

### `Rename { name }`

Changes the location's name. Use when the party learns a place's true name,
or when a location is renamed by its new occupants.

**Emits:** `Renamed`
**Errors:** `NameError::Empty`

---

### `UpdateDescription { description }`

Revises the location's description. Use as the DM adds detail after the party
has explored, or as the location changes over time short of full destruction.

**Emits:** `DescriptionUpdated`
**Errors:** none

---

## Events

### `Created { id, story_id, owner, name, description }`

A new place enters the world. The party may not know it exists yet, but the
DM has defined it.

---

### `Visited`

The party has been here. This is the moment the location becomes part of
their lived story, not just background knowledge.

---

### `Destroyed`

The location is gone. Depending on the story, this may be a tragedy, a
victory, or simply the consequence of events. Its history is preserved.

---

### `Renamed { name }`

The location is now known by a different name.

---

### `DescriptionUpdated { description }`

The location's description was revised or expanded.
