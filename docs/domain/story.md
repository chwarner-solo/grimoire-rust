# Aggregate: Story

The Story is the root of the domain. It holds the campaign's identity,
ownership, premise, and status. It does not contain characters, quests, or
locations — those are independent aggregates that reference the Story by ID.

## Status

```
Active → Inactive
```

| Status | Meaning |
|---|---|
| `Active` | The campaign is running |
| `Inactive` | The campaign has been closed or is not yet started |

---

## Commands

### `Create { owner, title, prose }`

Opens a new story. The `title` must be non-empty. The `prose` is the premise —
the one or two sentence hook that captures what the campaign is about. Neither
needs to be final; both can be updated later.

**Emits:** `Created`
**Errors:** `TitleError::Empty` if title is blank

---

### `UpdateTitle { title }`

Changes the story's title. The new title must be non-empty.

**Emits:** `TitleUpdated`
**Errors:** `TitleError::Empty` if title is blank

---

### `UpdateProse { prose }`

Updates the campaign premise. No validation — prose can be empty or long.
Use this when the DM refines the story's central idea over time.

**Emits:** `ProseUpdated`
**Errors:** none

---

### `Close`

Marks the story as finished. A closed story is inactive — it can still be read
and its history is preserved, but no further commands should be issued against
its aggregates.

**Emits:** `Closed`
**Errors:** none

---

## Events

### `Created { id, owner, title, prose }`

A new story exists. The `id` is generated at command time and carried in the
event to ensure stable identity across replays.

---

### `TitleUpdated { title }`

The story's title changed. All other fields are unchanged.

---

### `ProseUpdated { prose }`

The campaign premise was rewritten or refined.

---

### `Closed`

The campaign has ended. No further play is expected.
