---
name: draupnir-resume
description: Load recent session context and project memory from the Draupnir vault to resume work. Use when starting a new session, when the user says "resume", "load context", or at the beginning of work to restore previous context.
---

# Resume Context

Load recent session context filtered by current project.

Project is auto-detected by the `draupnir` CLI from `pwd` — pass `--project` only when overriding.

## Step 1: Check and Load the Brief

The brief is the canonical distilled summary of recent work. Always start here.

```bash
draupnir brief status --json
```

Parse the JSON. If `stale` is true or `exists` is false:

```bash
draupnir brief generate
```

Then load:

```bash
draupnir brief show
```

The brief output is your primary summary. Lean on it in Step 4.

## Step 2: Load Strategic Context

Read `CLAUDE.md` at the vault root (`draupnir vault root` prints it) for the user's strategic rules and cross-project context. This is the user's authored file, not CLI-managed.

## Step 3: Load Recent Session Metadata

```bash
draupnir session search --limit 3 --json
```

Parse the array. Each entry has `title`, `date`, `tags`, `ticketIds`, and `path`. Show titles and dates as "what was recently worked on" — do not open the files; the brief already distills their content.

If the user gave a topic (e.g. "resume auth"), filter with `--query`:

```bash
draupnir session search --query "auth" --limit 3 --json
```

## Step 4: Load Active Instincts

```bash
draupnir instincts list --confidence high --json
```

Parse and list all high-confidence patterns. If the user gave a topic, also include medium-confidence entries relevant to that topic:

```bash
draupnir instincts list --confidence medium --json
```

## Step 5: Summarize

Present a concise summary:

```
## Context Restored

**Brief:**
[Key points from Step 1 brief output]

**Recent Sessions:**
- {date} — {title} [{tags}]
- ...

**Active Instincts (high confidence):**
- [category] Pattern name — one-line summary

**Strategic Context:**
- [Relevant items from the vault's CLAUDE.md]

**Ready to continue.** What would you like to work on?
```

## Notes

- If `brief show` errors or the brief is empty, fall back to showing only session search results
- Never open session log files directly — the brief is the distilled form; session search gives you titles
- If no sessions match the current project, the CLI returns an empty array — report "no recent sessions for this project" and show strategic context only
