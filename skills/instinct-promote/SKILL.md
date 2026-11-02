---
name: instinct-promote
description: Promote a high-confidence instinct pattern to CLAUDE.md rules or a new skill. Use when the user says "/instinct-promote", "promote instinct", "promote pattern", or wants to codify a learned pattern into permanent rules.
---

# Instinct Promote

Promote a high-confidence pattern from the instinct registry to a permanent rule or skill.

Project is auto-detected by the `draupnir` CLI from `pwd` — pass `--project` only when overriding.

## Step 1: List Promotion Candidates

```bash
draupnir instincts list --confidence high --json
```

If the CLI returns an empty array:
"No high-confidence patterns to promote. Patterns need 3+ confirmations; `/draupnir-compress` adds one each time a session hits the same pattern."

Display candidates:
```
## Promotion Candidates

1. [Category] Pattern name — one-line summary (N confirmations)
2. [Category] Pattern name — one-line summary (N confirmations)
```

Ask the user which pattern to promote (by number or name).

## Step 2: Choose Promotion Target

Ask where to promote:

1. **Project CLAUDE.md** — Add to the project's `CLAUDE.md` rules (ALWAYS/NEVER sections)
2. **Global CLAUDE.md** — Add to `~/.claude/CLAUDE.md` (applies to all projects)
3. **Vault CLAUDE.md** — Add to `CLAUDE.md` at the vault root (strategic knowledge; `draupnir vault root` prints the path)
4. **New skill** — Create a new skill in `~/.claude/skills/`

## Step 3: Draft the Rule

Based on target:

### For CLAUDE.md (project/global/vault):
- Draft a concise rule matching the existing format
- For project CLAUDE.md: determine if it's an ALWAYS or NEVER rule
- Present the draft with the exact location where it will be inserted
- Example: `- ALWAYS wrap async client calls in a Result before mapping errors`

### For new skill:
- Draft a SKILL.md with frontmatter and step-by-step instructions
- Present for approval

## Step 4: Apply (with approval)

**Wait for explicit user approval before writing.**

1. Write the approved rule to the target file
2. Note: marking the registry entry as promoted currently requires manual edit — the CLI does not yet expose an `instincts promote` command. Append a `Promoted:` line to the matching entry in the instinct file the CLI uses.
3. Report: "Promoted '{pattern name}' to {target}."

## Notes

- Never auto-promote — always require explicit user approval
- Preserve existing formatting in target files
- If the pattern is project-specific, recommend project CLAUDE.md over global
- If the pattern applies broadly, recommend global CLAUDE.md
