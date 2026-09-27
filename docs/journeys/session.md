# Journey: Session

## Overview

A session represents one sitting of play — the unit of time where the story
actually moves forward. Sessions have three phases: **Prep**, **Play**, and
**Close**. Each phase has a distinct purpose and produces different kinds of
story data.

The session is the primary mechanism by which the story evolves. All state
changes to characters, quests, and locations are stamped with the session they
occurred in, giving the story a timeline the AI and DM can navigate.

---

## Actor

**The DM** drives all three phases. Players participate in the Play phase but
their actions are recorded by the DM (or eventually by the players themselves
once the permissions model supports it).

---

## Session Status

```
Planned → Active → Closed
```

| Status | Phase | Description |
|---|---|---|
| `Planned` | Prep | Session is scheduled, DM is preparing |
| `Active` | Play | The table is live, events are being recorded |
| `Closed` | Close | Session is sealed, summary is written |

---

## Phase 1: Prep

The DM opens a new session before the players arrive. Prep is where the DM
reviews the current state of the story and sets the stage for what's coming.

### What the DM does in Prep

**Review current story state** — the DM consults the current projections:
- Which characters are active?
- Which quests are open? What is the main quest's current state?
- Which locations are known but not yet visited?

**Add anticipated content** — the DM may create quests, NPCs, or locations
in advance of the session:
- A new NPC the party is about to meet
- A location they're travelling toward
- A side quest that will be offered this session

**Prepare encounters** — the DM plans the specific situations the party may
face this session. Encounters are the moments of direct engagement — where the
story either advances, the world pushes back, or a problem must be solved.

Encounter kinds:

| Kind | Description |
|---|---|
| `Combat` | A fight — planned ambush, random creature, hostile faction |
| `Diplomacy` | A negotiation, interrogation, social challenge |
| `Puzzle` | An obstacle requiring thought — trap, lock, ritual |
| `Story` | A narrative beat — reveal, NPC scene, world event |
| `Random` | Unplanned, generated at the table during play |

Encounters prepared in Prep have a `Planned` status. They may or may not occur
— the DM may skip them, or new encounters may arise spontaneously during Play.

**Note: nothing in Prep is required.** The DM can open a session and go
straight to Play with no preparation recorded in the system.

**Commands issued:**
```
SessionCommand::Create { story_id, owner, number, date, notes }
EncounterCommand::Create { session_id, story_id, kind, description }
```

Supporting commands (optional):
```
CharacterCommand::Create    ← new NPC being introduced
QuestCommand::Create        ← quest being planted this session
LocationCommand::Create     ← location being revealed this session
```

---

## Phase 2: Play

The DM opens the session — the table is live. Everything that happens from
this point is stamped with the current `session_id`. This is the record of
what actually occurred.

### What gets recorded during Play

**Character events** — things that happen to characters:
- A new character joins the story (PC introduced, NPC appears)
- A character departs (leaves the party, goes their own way)
- A character dies
- A character visits a location

**Quest events** — story lines that advance:
- A quest is completed, failed, or abandoned
- A new quest is discovered and added
- A character is assigned to or removed from a quest

**Encounter events** — what the party faced:
- A planned encounter is activated (the DM runs it)
- A random encounter is created on the spot and immediately activated
- An encounter is resolved with an outcome
- An encounter is skipped (planned but never ran)

**Location events** — the world being explored:
- A location is visited for the first time
- A location is destroyed or otherwise changed permanently

All of these events carry `session_id` so the full history of a session can
be reconstructed from the event store.

**Commands issued (all stamped with session_id):**
```
SessionCommand::Open

CharacterCommand::Depart    ← with session_id
CharacterCommand::Kill      ← with session_id
CharacterCommand::LocationVisited { location_id, session_id }

QuestCommand::Complete      ← with session_id
QuestCommand::Fail          ← with session_id
QuestCommand::Abandon       ← with session_id
QuestCommand::CharacterAssigned { character_id, session_id }

LocationCommand::Visit      ← with session_id
LocationCommand::Destroy    ← with session_id

EncounterCommand::Create    ← random encounter, created during play
EncounterCommand::Activate  ← planned encounter begins
EncounterCommand::Resolve { outcome }  ← what happened, how it ended
EncounterCommand::Abandon   ← DM explicitly retired it, no longer fits
EncounterCommand::DeadEnd { reason }  ← prerequisite no longer exists
```

---

## Phase 3: Close

After the table clears, the DM writes the session summary — the narrative
account of what happened. This is the most valuable piece of context the AI
will have for next session prep.

The summary does not need to be exhaustive. It needs to capture:
- What the party accomplished (or failed to accomplish)
- Any significant character moments
- How the world changed
- What threads were left open heading into the next session

Once the summary is written, the session is **Closed** — no further events
can be tagged to it. It becomes a sealed, immutable record.

**Commands issued:**
```
SessionCommand::Summarize { summary }
SessionCommand::Close
```

---

## What the AI Gets From a Closed Session

A closed session gives the AI:

1. **The summary** — the DM's narrative account ("previously on...")
2. **The event record** — every state change that occurred, in order
3. **The delta** — what changed from the previous session to this one

This is the richest AI context in the system. When the DM asks the AI to help
prep the next session, the AI can reason about:
- What threads were left unresolved
- Which characters are at risk or in interesting positions
- What the natural next beat of the Main Quest might be

---

## Session and Story Creation

Per the Story Creation journey, a session can begin before the story is
complete. A DM can open Session 1 with only a title and a premise — characters
and quests can be created during Prep or even during Play.

This means the first session often doubles as the completion of Story Creation.

---

## Definition of Done

A session is complete when:

1. It has a `Closed` status
2. It has a summary written by the DM
3. All state changes from play are recorded and stamped with the session id

A session with `Planned` or `Active` status is considered in-progress. Only
`Closed` sessions are served to the AI as historical context.

---

## Decisions

### The Event Log is Immutable

Closed sessions cannot be reopened. Mistakes are corrected through
**compensating events** — a new event that reverses or supersedes the error:

> A character was incorrectly marked as deceased → `CharacterReinstated` event
> is issued in the current session, explaining the correction.

This preserves the full history, including the mistake and its correction.

### The AI Can Draft the Summary

The DM may request the AI to write the session summary. The AI drafts it based
on the session's event record — and may ask the DM clarifying questions about
details not captured in the data (how a combat ended emotionally, what the NPC's
motives turned out to be, etc.).

The DM reviews, edits if needed, and confirms. The confirmed summary is submitted
as `SessionCommand::Summarize` — DM-initiated, AI-assisted. The AI never writes
to the summary unprompted.

### Prep Notes and Summary are Separate Fields

`notes` — written during Prep, the DM's intentions and plans for the session.
`summary` — written during Close, the record of what actually happened.

They serve different purposes. Notes are planning; summary is history. The AI
uses the summary as historical context, not the notes.
