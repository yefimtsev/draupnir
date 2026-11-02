---
name: draupnir-compress
description: Save current conversation to a structured session log in the Draupnir vault. Use when ending a session, when significant work is completed, when the user says "compress", "save session", or when wrapping up to preserve important context for future sessions.
---

# Compress Session

Save the current conversation to a structured session log and optionally update permanent memory.

Project is auto-detected by the `draupnir` CLI from `pwd` — pass `--project` only when overriding.

## Step 1: Analyze Conversation

Extract key information:
- **Decisions Made**: Choices or conclusions reached
- **Tasks Completed**: What was accomplished
- **Tasks Created**: New todos that emerged
- **Key Learnings**: Important insights or discoveries
- **Code Changes**: Files modified and why
- **Open Questions**: Unresolved items for future sessions

## Step 1b: Extract Product Decisions

From the "Decisions Made" list in Step 1, identify entries that are **product or architecture decisions** — choices that shape what the app does, how features behave, what's in/out of scope, or how components are structured. Exclude session-local implementation details (e.g., "used Mutex instead of NSLock in this file").

For each product decision, draft a one-row table entry:

```
| {date} | {what was decided} | {why — one sentence} |
```

These will be appended to the Decision Log in Step 2b.

## Step 2: Write Session Log

Compose plain markdown with section headers (`## Decisions Made`, `## Tasks Completed`, etc.) from Step 1. Pipe directly to the CLI — do NOT construct frontmatter; the CLI adds it automatically when stdin does not start with `---`.

```bash
draupnir session write --topic "{topic}" --tag "{tag1}" --tag "{tag2}" <<'EOF'
## Decisions Made
...

## Tasks Completed
...

## Open Questions
...
EOF
```

The CLI prints the written file path to stdout. Capture it for Step 2a.

## Step 2a: Update Session Index

```bash
draupnir index append --file {log_path_from_step2}
```

## Step 2b: Append Product Decisions

If Step 1b produced any product decision rows, append them to the Decision Log via the CLI.

```bash
draupnir decision add --decision "{what was decided}" --why "{why — one sentence}"
```

Project is auto-detected from pwd. The CLI resolves the vault path, locates the Decision Log table, and appends a properly formatted row with today's date.

If no product decisions were made this session, skip this step.

## Step 2c: Regenerate Brief

```bash
draupnir brief generate
```

The session we just wrote changes the project context — regenerate the brief now so the next `/draupnir-resume` gets a fresh summary without a stale/missing brief gap.

## Step 2d: Extract Instincts (Optional)

After writing the session log, review Key Learnings and Decisions Made from Step 1 and identify reusable pattern candidates. Then use `AskUserQuestion` to present them:

- List the candidate patterns you found as the question text (e.g., "Found N reusable patterns: ...")
- Options: "Yes, add them" / "Skip"
- If the user picks Skip → go to Step 3

If yes:
1. Identify candidates that are **reusable patterns** (not session-specific facts)
   - Reusable: "ViewThatFits is unsafe around focusable views" (applies broadly)
   - Session-specific: "PROJ-14 PR was missing a nonce field" (one-time fact)
2. For each candidate:
   a. Determine category: reuse an existing one from `draupnir instincts stats` when it fits. Otherwise use one of Workflow | Architecture | Debugging | Testing | Git | CI | Performance. Add a new category (e.g. a language or framework) only when none of these fits.
   b. Check for similar existing entries: `draupnir instincts list --category {category} --json`. If an entry already states the same pattern, confirm it instead of adding a duplicate: `draupnir instincts confirm --title "{existing title}"`. That raises its confidence (2 confirmations → medium, 3+ → high).
   c. If new → add via CLI
3. Write each new instinct via CLI:

```bash
echo "instinct description here" | draupnir instincts add --category "{category}" --title "{title}"
```

The CLI creates the entry at Confirmations: 1, Confidence: low.

4. Report: "Added N new instinct(s). Confirmed M existing one(s)." Name any that reached high confidence and suggest `/instinct-promote`.

## Step 3: Offer Permanent Memory Update

Use `AskUserQuestion`: "Anything to remember permanently in the vault?"
- Options: "No, we're done" / "Yes, update vault memory"

If yes → ask what to remember, then update the relevant section in `CLAUDE.md` at the vault root (`draupnir vault root` prints it):
- Strategic decisions → `## Strategic Decisions`
- Cross-project learnings → `## Cross-Project Learnings`
- Project status changes → `## Active Projects`

If no → done.

## Notes

- Ask for the topic name if not provided by user
- Keep summaries concise but complete
- Before writing the log body, apply any writing-style rules from the user's CLAUDE.md (for example, a prose or style skill they ask you to load)
- Focus on actionable information for future sessions
- All logs go to the vault (centralized), never to project-local directories
