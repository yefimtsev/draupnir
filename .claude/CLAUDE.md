# CLAUDE.md — draupnir-cli

## What This Is

Rust CLI for the Draupnir vault. Handles session logs, instincts, briefs, and health checks.
Skills call this binary; this binary does not call skills.

## Build & Test

- `cargo check` — type check
- `cargo build` — debug build
- `cargo test` — run all tests
- `cargo clippy -- -D warnings` — lint (zero warnings)
- `cargo fmt -- --check` — format check

## Code Standards

- Edition 2024
- Error handling: `anyhow::Result` everywhere, no `.unwrap()` in non-test code
- Serialization: `serde` with `#[serde(rename_all = "camelCase")]` for JSON output
- CLI: `clap` derive macros
- Tests: `assert_cmd` for integration, `insta` for snapshot testing

## Bundled Skills

- `skills/` is the one source for the user-facing skills (`/draupnir-resume`, `/draupnir-compress`, ...).
  They are embedded via `include_str!` in `src/commands/skills.rs`; users get them with `draupnir skills install`.
- Dev: `make link-skills` symlinks them into `~/.claude/skills`, so edits here are live.
- A new skill goes into `skills/` and the `SKILLS` list. The `every_documented_command_parses` test fails when
  a skill shows a `draupnir ...` command the CLI no longer accepts — fix the skill in the same change.
- Skills stay generic: no personal projects, paths or skill names. Style rules come from the user's CLAUDE.md.

## Automations

- **Hooks** (`.claude/settings.json`): `cargo fmt` + `cargo clippy` run automatically after every Edit/Write of a `.rs` file.
  Edits to a path containing `Documents/Draupnir` (the default vault) are blocked. The hooks need `jq` and a POSIX shell
  (Git Bash on Windows) and fail loudly without `jq`.
- **Subagent** (`.claude/agents/test-runner.md`): Haiku-model agent runs `cargo test` + clippy.
- **Skills**:
  - `/draupnir-smoke` — build release binary and run smoke tests
  - `/draupnir-release` — bump version, build, tag, and install. Publishing the GitHub Release triggers
    `.github/workflows/release.yml`, which attaches Linux, macOS and Windows binaries.

## Windows

- CI runs on Linux, macOS and Windows. Compare paths through `norm_path` (`src/models/config.rs`), read vault
  markdown through `vault::read_text` (CRLF → LF), and resolve home through `vault::home_dir` (honours `HOME`).
- Unix-only test code goes behind `#[cfg(unix)]`. `make` is Unix-only; on Windows use `cargo` directly.
- `eval/`, when present, is a separate private repo (the public one ignores it); its make targets come from `eval/eval.mk`.

## NEVER

- NEVER use `.unwrap()` or `.expect()` outside of tests
- NEVER change the vault layout the CLI assumes without updating the "Vault layout" section of README.md
