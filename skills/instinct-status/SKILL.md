---
name: instinct-status
description: View learned patterns filtered by project, category, or confidence. Use when the user says "/instinct-status", "show instincts", "what patterns have we learned", or wants to review the instinct registry.
---

# Instinct Status

Display the instinct registry for the current project, filtered by category or confidence.

Project is auto-detected by the `draupnir` CLI from `pwd` — pass `--project` only when overriding.

## Step 1: Query the Registry

```bash
draupnir instincts list --json
```

Apply filters from the user's arguments:

- Category filter (e.g. "Git", "Testing") → add `--category Git`
- Confidence filter (e.g. "high") → add `--confidence high`

Both filters compose. Example with both:

```bash
draupnir instincts list --category Testing --confidence high --json
```

If the CLI returns an empty array, report: "No instincts match these filters (or registry is empty). Patterns will be captured during `/draupnir-compress`."

## Step 2: Display

Group the returned entries by confidence level (high → medium → low):

```
## Instinct Registry

### High Confidence (promotion candidates)
- [Category] Pattern name — one-line summary (N confirmations)

### Medium Confidence
- [Category] Pattern name — one-line summary (N confirmations)

### Low Confidence
- [Category] Pattern name — one-line summary (N confirmations)

---
Total: N patterns | High: N | Medium: N | Low: N
Promotion candidates (high confidence): list names or "none"
```

## Step 3: Summarize Counts

For the totals line, also run:

```bash
draupnir instincts stats
```

Use its output for the summary line at the bottom of the display.

## Notes

- High-confidence patterns are promotion candidates — mention `/instinct-promote` if any exist
- Keep display concise — full details come from the CLI's JSON output
