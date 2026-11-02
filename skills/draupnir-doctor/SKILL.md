---
name: draupnir-doctor
description: Audit Draupnir vault health — check staleness, rebuild indexes, verify project index, assess instinct registry size. Use when the user says "doctor", "vault health", "check staleness", or wants to maintain vault infrastructure.
---

# Draupnir Doctor

Audit and repair vault health. Run periodically or when things feel stale.

Project is auto-detected by the `draupnir` CLI from `pwd` — pass `--project` only when overriding.

## Step 1: Run Health Audit

```bash
draupnir doctor --json
```

Parse the JSON array of checks. Each has `name`, `status` (ok/warn/error), and `message`.
Present the report card from the JSON output.

## Step 2: Present Report Card

```
## Draupnir Health — {Project}

| Artifact | Status | Details |
|----------|--------|---------|
| Session Index | {status} | {count} indexed, {N} unindexed |
| Current Brief | {status} | Generated {date}, TTL {status} |
| Project Index | {status} | {details} |
| Instinct Registry | {status} | {lines} lines, {patterns} patterns |
| Inbox | {status} | {count} items pending |
```

## Step 3: Offer Repairs

For each non-OK status, offer to fix. Alternatively, `draupnir doctor --fix` performs all safe auto-repairs (rebuild index, regenerate brief) in one call.

- **Session Index STALE/MISSING** → "Rebuild session index?"
  - If yes:
    ```bash
    draupnir index rebuild
    ```
- **Current Brief STALE/MISSING** → "Regenerate current brief?"
  - If yes:
    ```bash
    draupnir brief generate
    ```
- **Project Index MISSING** → "Create project index?"
  - If yes: scan vault project directory, generate `_index.md` with folder contents and links (manual — v0.2)
- **Instinct Registry SPLIT SOON/NOW** → "Split instinct registry by category?"
  - If yes: create category files (`instincts-git.md`, `instincts-workflow.md`, etc.), move patterns, update main file as dispatch index (manual — v0.2)
- **Inbox NEEDS TRIAGE** → "Review inbox items?"
  - If yes: show each item, ask: promote to durable doc / turn into a ticket in the user's tracker (only on an explicit yes) / delete (manual — v0.2)

Use `AskUserQuestion` with repair options. Only repair what the user approves.

## Step 4: Summary

Report what was repaired, what was skipped, and when to run doctor again (suggest: weekly, or after heavy session activity).

## Notes

- Doctor is read-heavy, write-only-with-approval
- Never auto-repair without asking — this is an audit, not a cron job
- Keep the report concise — the user wants signal, not a wall of text
