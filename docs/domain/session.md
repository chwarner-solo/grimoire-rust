# Aggregate: Session

A session represents one sitting of play. It is the unit of time by which
the story advances. Sessions have three phases — Prep, Play, and Close —
and carry two text fields: `notes` (DM's intentions before play) and
`summary` (the record of what actually happened).

All state-change events on other aggregates are stamped with the active
`session_id`, giving the story a navigable timeline.

## Status

```
Planned → Active → Closed
```

| Status | Meaning |
|---|---|
| `Planned` | The session is scheduled; DM is in Prep |
| `Active` | The table is live; events are being recorded |
| `Closed` | Play has ended; summary is written; session is sealed |

A `Closed` session is immutable. Corrections to events that occurred in a
closed session are made via compensating events in a future session.

---

## Commands

### `Create { story_id, owner, number, date, notes }`

Opens a new session in the Planned state. The `number` is the session
counter (1, 2, 3...). The `date` records when the players met. `notes`
are the DM's pre-session preparation — intentions, planned encounters,
reminders. Notes are optional.

**Emits:** `Created`
**Errors:** none

---

### `Open`

Moves the session from `Planned` to `Active`. The table is live. From
this point, all state-change commands on characters, quests, locations,
and encounters should carry this session's ID.

**Emits:** `Opened`
**Errors:** `NotPlanned` if session is already active or closed

---

### `UpdateNotes { notes }`

Revises the DM's prep notes while the session is `Planned` or `Active`.
Notes are working material — they are not served to the AI as historical
context.

**Emits:** `NotesUpdated`
**Errors:** `AlreadyClosed`

---

### `Summarize { summary }`

Records what actually happened during the session. The summary is the
DM's narrative account — what the party did, what changed, what threads
were left open. Can be written during `Active` or `Closed` status.

The DM may request the AI draft the summary; the DM reviews and confirms
before this command is issued. The AI never summarises unprompted.

**Emits:** `Summarized`
**Errors:** none

---

### `Close`

Seals the session. No further events can be stamped with this session's ID.
The session becomes immutable historical record.

**Emits:** `Closed`
**Errors:** `NotActive` if session has not been opened

---

## Events

### `Created { id, story_id, owner, number, date, notes }`

A new session exists in the Planned state.

---

### `Opened`

Play has begun. The session is now Active and accepting stamped events
from other aggregates.

---

### `NotesUpdated { notes }`

The DM's prep notes were revised.

---

### `Summarized { summary }`

The DM wrote (or confirmed) the session summary. This is the primary AI
context for "what happened last time" and "what threads are open."

---

### `Closed`

The session is sealed. History is fixed. Any errors discovered after this
point are corrected via compensating events in a future session.
