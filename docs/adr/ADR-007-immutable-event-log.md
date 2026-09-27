# ADR-007: Immutable Event Log and AI Write Constraints

**Status:** Accepted

## Context

Two related decisions about data integrity: whether the event log can be
corrected after the fact, and whether the AI can freely modify story data.

## Decision

### The Event Log is Immutable

Events, once written, are never edited or deleted. Closed sessions cannot be
reopened. Mistakes are corrected through **compensating events** — a new event
that explicitly reverses or supersedes the error.

> A character was incorrectly marked as deceased.
> Correction: issue `CharacterReinstated` in the current session with a note
> explaining the error.

The full history — including the mistake and its correction — is preserved.

### The AI May Only Write on State Changes

The AI cannot freely edit prose, backstories, descriptions, or any narrative
content. All AI-driven writes must flow through the same command → event
pipeline as DM actions.

The AI **may**:
- Issue state-change commands when the DM confirms them (e.g. mark a quest
  complete, update a character's status)
- Draft a session summary when the DM requests it — the DM reviews and
  confirms before `SessionCommand::Summarize` is issued

The AI **may not**:
- Rewrite a character's backstory unprompted
- Edit story prose directly
- Create new aggregates without DM confirmation

## Rationale

Immutability guarantees the event log is a trustworthy timeline. Every change
is intentional, authored, and traceable. The AI constraint ensures the DM
remains the author of the story — the AI is a collaborator, not an editor.

## Consequences

- The `CommandSource` on the event envelope distinguishes DM-authored events
  from AI-assisted ones: `Dm | Player(UserId) | Ai`.
- Compensating events must be first-class domain events, not special cases.
  E.g. `CharacterReinstated`, `QuestReopened`.
- The AI's MCP write tools are limited to state-change commands, never
  free-form field updates.
