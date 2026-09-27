# Journey: Story Creation

## Overview

The Story Creation journey is the foundation of everything. Before any session can
be played, before any character takes action, the DM establishes the world the story
lives in. A story does not need to be complete to begin — it needs to be *playable*.

The goal of this journey is to get the story tight enough that:
- The DM knows what they're trying to tell
- The players know who they are and why they're together
- The world has enough shape to feel real on session one

---

## Actor

**The DM (Dungeon Master / Story Writer)** — the person who owns and drives the
narrative. They create the story, define the world, control NPCs, and set the
direction of the campaign.

---

## Minimum Viable Story

A story is playable when it has:

| Element | Why it matters |
|---|---|
| A title and premise | Anchors the tone and direction |
| At least one Main Quest | Gives the players a reason to act |
| At least one Player Character | Someone for the players to inhabit |
| At least one starting Location | Somewhere for the story to begin |

Everything else — side quests, NPCs, deeper world detail — can be added as the
campaign develops. The DM should not feel blocked from starting until the world
is complete.

---

## The Journey

### 1. Establish the Story

The DM opens a new story with a **title** and a **premise** — the one or two
sentence hook that captures what the campaign is about.

> *"The Long Road — A merchant empire has collapsed overnight. Its former
> enforcers are now scattered, hunted, or free for the first time. The players
> are three of them."*

The premise is not the full story. It is the seed.

**Commands issued:**
```
StoryCommand::Create { owner, title, prose: premise }
```

---

### 2. Define the Main Idea

The DM identifies the **Main Quest** — the spine of the campaign. This is what
the story is ultimately about, even if the players never know it at session one.

The Main Quest does not need to be fully detailed. It needs a name and enough
of a description to keep the DM pointed in the right direction.

> *Main Quest: "Dismantle the remnants of the Vareth Trading Company before
> they reconsolidate power under new leadership."*

**Commands issued:**
```
QuestCommand::Create { story_id, owner, name, description, kind: QuestKind::Main }
```

---

### 3. Establish Who's in Control

The DM defines the **power structure** — who is driving events in the world that
the players will react to. This is typically captured in the story prose and
through NPCs created as NonPlayer characters.

Key questions:
- Who is the antagonist, or the force the players are pushing against?
- Who are the allies or factions the players might work with?
- Are any of these characters present from session one?

These figures are created as **NonPlayer Characters** with backstories that
reflect their role in the world.

**Commands issued:**
```
CharacterCommand::Create { story_id, owner, name, backstory, kind: CharacterKind::NonPlayer }
```

---

### 4. Establish the Player Characters

The DM (or players, with the DM recording) defines the **Player Characters** —
who is actually at the table. Each PC needs at minimum a name and a backstory
hook that connects them to the premise.

> *"Kira — former enforcer, now wanted. She knows where the bodies are buried."*

**Commands issued:**
```
CharacterCommand::Create { story_id, owner, name, backstory, kind: CharacterKind::PlayerCharacter }
```

---

### 5. Establish the Starting Location

The story needs somewhere to begin. The DM creates at least one **Location** —
the place where session one opens. It does not need to be fully detailed, but it
needs enough description to be rendered by the DM and understood by the AI.

> *"Thornwall — a fortified waystation on the edge of the collapsed empire's
> former territory. Half the residents are ex-company, half are people who
> survived it."*

**Commands issued:**
```
LocationCommand::Create { story_id, owner, name, description }
```

---

## Definition of Done

The story is ready to play when the DM can answer:

1. **What is this story about?** → Story premise + Main Quest
2. **Who are the players?** → At least one Player Character per player
3. **Who is opposing them?** → At least one NPC with a defined role
4. **Where does it start?** → At least one Location
5. **What do they do first?** → Main Quest or an opening Side Quest

---

## What the AI Needs From This Journey

When the AI is consulted — for session prep, NPC dialogue, world detail — it
needs the following projection from a completed Story Creation journey:

- Story title and premise
- Main Quest name and description
- All active characters (PCs and NPCs) with their backstories
- Starting location(s) with descriptions
- Any side quests already defined

This is the **Story Context Projection** — the minimum context the AI needs to
be useful from session one.

---

## Decisions

### Story Completion is Not a Prerequisite

A session can begin at any point. The DM does not need to finish Story Creation
before opening Session 1. Story and session content are built incrementally —
characters, quests, and locations can be added at any time. The system never
blocks progress based on story completeness.

This means Story Creation is an ongoing activity, not a phase that closes.

---

### How the AI Interacts With Story Data

The AI receives **what has been completed** — the current projection of all
defined story data. It does not receive a fixed dump; it can request specific
projections if it needs more context (e.g. "give me all active quests" or
"give me the backstory for this character").

**The AI may only update data when a state change occurs.** It cannot freely
rewrite prose, backstories, or descriptions. All updates flow through the same
command → event pipeline as DM actions. This preserves the integrity of the
event log — every change is intentional and traceable.

Examples of permitted AI-driven updates:
- Marking a quest complete after the DM confirms it
- Updating a character's status after a session event
- Adding a location that was revealed during play

Examples of what the AI cannot do:
- Rewrite a character's backstory unprompted
- Edit story prose directly
- Create new aggregates without DM confirmation

---

### Who Creates Player Characters

**Initially, the DM creates all characters** — including Player Characters —
from information provided by the players. The DM is the sole author of story
data at the start.

**Eventually, players will be able to manage their own character details**
directly. This requires a future permissions model where a player's `UserId`
is bound to specific characters they own.

**Longer term: Foundry VTT import.** Character data can be imported from
Foundry VTT (a popular Virtual Tabletop platform) via a dedicated adapter port.
This is explicitly out of scope for the current build but the hexagonal
architecture accommodates it — it becomes one more adapter implementing the
character creation port.
