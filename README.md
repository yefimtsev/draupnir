# draupnir

A CLI that keeps a markdown vault (Obsidian-friendly, plain files) as long-term memory for Claude Code.
It writes session logs, builds a per-project brief from the recent ones, tracks learned patterns ("instincts") and a decision log, and checks the vault's health.

The binary also carries a set of Claude Code skills (`/draupnir-resume`, `/draupnir-compress`, ...) that call it.
The skills decide what to remember; the CLI does the file work.

Runs on macOS, Linux and Windows.

## Install

**Prebuilt binary.** Download the file for your platform from [Releases](https://github.com/yefimtsev/draupnir/releases) and put it on your `PATH` as `draupnir` (`draupnir.exe` on Windows).
The binaries are not signed.
On macOS and Linux run `chmod +x draupnir`; on macOS also `xattr -d com.apple.quarantine draupnir`, or Gatekeeper refuses to start it.
On Windows, SmartScreen may warn on the first run.

**With Cargo** (needs a Rust toolchain; on Windows also the MSVC build tools):

```sh
cargo install --git https://github.com/yefimtsev/draupnir
```

**From a checkout:** `cargo install --path .`, or `make install` on macOS/Linux (symlinks the release build into `~/.local/bin`).

## Configuration

The config is looked up in this order:

1. `~/.config/draupnir/config.yaml` (on Windows: `C:\Users\<you>\.config\draupnir\config.yaml`)
2. The platform config dir: `~/Library/Application Support/draupnir/config.yaml` on macOS, `%APPDATA%\draupnir\config.yaml` on Windows, `$XDG_CONFIG_HOME/draupnir/config.yaml` on Linux

The minimum is one line:

```yaml
vault_root: ~/Documents/Draupnir
```

Then register projects with `draupnir project add`, or by hand:

```yaml
vault_root: ~/Documents/Draupnir
session_logs_dir: Areas/Work/Session-Logs   # default, relative to vault_root
projects:
  acme:
    title: Acme
    path: Areas/Work/Projects/Acme          # project folder inside the vault
    prefix: ACME                            # optional: ticket IDs like ACME-14 are picked out of logs
    repos: [~/dev/acme, ~/dev/acme-api]     # cwd inside one of these → this project
    detect_patterns: [acme]                 # or: cwd contains this text
```

The project is detected from the current directory unless you pass `--project`.
The most specific match wins (the longest matching repo path or pattern), and a tie between two projects is an error rather than a guess.
Patterns and repos may use `/` on every platform; on Windows the comparison also ignores case.

**Windows paths in YAML:** write `C:/Users/you/Vault` or `'C:\Users\you\Vault'` (single quotes).
Inside double quotes a backslash starts an escape sequence, so `"C:\Users\..."` breaks.

## Vault layout

Paths are relative to `vault_root`; folders are created when first written.

```
<session_logs_dir>/                    session logs, one markdown file each, with YAML frontmatter
<project path>/product-scope.md        decision log under a "## Decision Log" heading
<project path>/Learnings/instincts.md  instinct registry (may be split into instincts-*.md)
<project path>/_generated/             session index and brief, rebuilt by the CLI
+Inbox/                                quick captures
CLAUDE.md                              your own notes; the resume skill reads it
```

A log's `project:` frontmatter decides which project it belongs to; the filename is only a fallback for old logs.
Files with Windows (CRLF) line endings are read the same as LF.

## Claude Code skills

```sh
draupnir skills install
```

This copies the bundled skills into `~/.claude/skills` (or `$CLAUDE_CONFIG_DIR/skills`).
Skills you edited locally are left alone unless you pass `--force`.

| Skill | What it does |
|---|---|
| `/draupnir-resume` | Load the brief, recent sessions and instincts at session start |
| `/draupnir-compress` | Save the conversation as a session log |
| `/draupnir-capture` | Quick-capture a note to the inbox |
| `/draupnir-doctor` | Vault health: staleness, indexes, registry size |
| `/instinct-status` | List learned patterns |
| `/instinct-promote` | Turn a high-confidence pattern into a rule or a skill |

The skills run `draupnir` through Claude Code's Bash tool (heredocs included).
On Windows Claude Code runs that tool in Git Bash, so they work there unchanged.

## Commands

`draupnir --help` lists them; each has its own `--help`.

| Command | Purpose |
|---|---|
| `session` | Write a session log from stdin, search logs |
| `index` | Rebuild, append to and inspect a project's session index |
| `brief` | Generate, show, check and invalidate the project brief |
| `instincts` | Add, confirm, list and count instinct patterns |
| `decision` | Add to, list and read the decision log |
| `doctor` | Health checks for the vault and a project |
| `project` | Add and list configured projects |
| `capture` | Quick note to the inbox |
| `vault` | Print the resolved vault root |
| `skills` | Install the bundled skills |

## Development

```sh
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

CI runs these on Linux, macOS and Windows.
The `Makefile` assumes a Unix shell; on Windows call `cargo` directly.

The Claude Code hooks in `.claude/settings.json` (format and lint after each edit) need `jq` on `PATH` and a POSIX shell, which on Windows means Git Bash.

## License

MIT, see [LICENSE](LICENSE).
