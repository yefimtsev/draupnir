---
name: draupnir-capture
description: Quick-capture an idea or note to the Draupnir vault inbox. Use when the user says "capture", "quick note", "idea", "jot down", or wants to save a thought without a full session log.
---

# Quick Capture

Zero-friction capture to the Draupnir inbox.

Project is auto-detected by the `draupnir` CLI from `pwd` — pass `--project` only when overriding.

## Workflow

```bash
draupnir capture "{user's content}"
```

The CLI creates a timestamped file in `+Inbox/` and prints the path. Confirm in one line: `Captured to {printed path}`

## Notes

- No menus, no questions — just capture and confirm
- The user's content comes from their message arguments or the conversation context
- If no content is provided, ask once: "What do you want to capture?"
