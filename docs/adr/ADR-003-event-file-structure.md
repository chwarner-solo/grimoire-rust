# ADR-003: Directory Structure as Event Stream Identity

**Status:** Accepted

## Context

Events are stored as individual files. The file path must encode enough
information to locate all events for a given aggregate without reading file
contents — enabling efficient replay and directory-traversal queries.

## Decision

Use a **nested directory structure** where the path encodes identity:

```
events/
  {owner_id}/
    {story_id}/
      {aggregate_type}/
        {aggregate_id}/
          {uuid_v7}.json
```

Example:
```
events/
  01j4z.../
    01j5a.../
      character/
        01j6b.../
          01j9x....json   ← CharacterCreated
          01jb2....json   ← CharacterDeparted
      quest/
        01j7c.../
          01ja1....json   ← QuestCreated
          01jc3....json   ← QuestCompleted
```

## Rationale

- The path **is** the identity — no metadata encoded in the filename itself.
- UUID v7 filenames are **time-sortable** — filesystem sort equals temporal
  order. Replay is `ls | sort | read each`.
- Directory traversal queries are pure filesystem operations:
  - All events for one character: `{owner}/{story}/character/{id}/`
  - All character events in a story: `{owner}/{story}/character/`
  - Everything in a story: `{owner}/{story}/`
  - Everything for a user: `{owner}/`
- Compatible with cloud object stores (GCS, S3) — the same path scheme works
  as object key prefixes. Switching storage backends requires only a new
  adapter, not a path redesign.

## Consequences

- Event type lives inside the JSON body, not in the path or filename.
- A story with many events produces a directory with many small files.
  The aggregate store snapshots mitigate full-history replay cost.
- The `notify` crate (or cloud equivalents) can watch directories for new
  files — the path gives aggregate identity before the file is opened.
