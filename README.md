# draupnir

Long-term memory for Claude Code, kept as plain markdown files.  
Session logs, a per-project brief, learned patterns and a decision log live in a folder you can open in Obsidian.

Claude Code skills drive it: `/draupnir-resume` at the start of a session, `/draupnir-compress` at the end.  
The `draupnir` CLI does the file work. It runs on macOS, Linux and Windows.

## Install

### With an agent

Give your agent this link. It asks for the vault folder and the first project, then installs and checks everything.

```
https://raw.githubusercontent.com/yefimtsev/draupnir/master/docs/agent-install.md
```

### Windows

```powershell
irm https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1 | iex
```

No admin rights needed. It asks for the vault folder and sets up `PATH`, the config and the skills.  
Restart your terminals and Claude Code afterwards.

To uninstall (your vault, config and skills stay):

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1))) -Uninstall
```

### macOS and Linux

```sh
mkdir -p ~/.local/bin
curl -fsSL -o ~/.local/bin/draupnir \
  https://github.com/yefimtsev/draupnir/releases/latest/download/draupnir-aarch64-apple-darwin
chmod +x ~/.local/bin/draupnir
draupnir skills install
```

On Linux (x86-64) download `draupnir-x86_64-unknown-linux-gnu` instead.  
`~/.local/bin` has to be on your `PATH`. Then create the config, see below.

### From source

```sh
cargo install --git https://github.com/yefimtsev/draupnir
```

## Configuration

One line in `~/.config/draupnir/config.yaml` is enough to start:

```yaml
vault_root: ~/Documents/Draupnir
```

Then register a project:

```sh
draupnir project add acme --title Acme --repo ~/dev/acme
```

Inside `~/dev/acme`, every command now works on the `acme` project. `--project` overrides that.

<details>
<summary>Full config reference</summary>

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

Where the config is looked up, in order:

1. `~/.config/draupnir/config.yaml`
2. `~/Library/Application Support/draupnir/` on macOS, `%APPDATA%\draupnir\` on Windows, `$XDG_CONFIG_HOME/draupnir/` on Linux

When two projects match the current directory, the longer match wins. A tie is an error.  
Paths may use `/` on every platform, and on Windows case does not matter.

On Windows, write paths as `C:/Users/you/Vault` or `'C:\Users\you\Vault'`.  
In double quotes YAML treats `\` as an escape.

</details>

## Vault layout

Paths are relative to `vault_root`. Folders appear the first time something is written there.

```
<session_logs_dir>/                    session logs, one markdown file each
<project path>/product-scope.md        decision log, under "## Decision Log"
<project path>/Learnings/instincts.md  learned patterns
<project path>/_generated/             session index and brief, rebuilt by the CLI
+Inbox/                                quick captures
CLAUDE.md                              your own notes; /draupnir-resume reads it
```

Each log carries frontmatter: `date`, `time`, `project`, `title`, `topic`, `tags`.  
The `project:` field decides which project a log belongs to.

## Skills

| Skill | What it does |
|---|---|
| `/draupnir-resume` | Load the brief, recent sessions and patterns |
| `/draupnir-compress` | Save the conversation as a session log |
| `/draupnir-capture` | Quick note to the inbox |
| `/draupnir-doctor` | Check the vault's health |
| `/instinct-status` | List learned patterns |
| `/instinct-promote` | Turn a pattern into a rule or a skill |

`draupnir skills install` copies them into `~/.claude/skills`.  
Skills you edited are kept; `--force` overwrites them.

## Commands

| Command | Purpose |
|---|---|
| `session` | Write a session log, search logs |
| `index` | Rebuild and inspect the session index |
| `brief` | Generate and show the project brief |
| `instincts` | Add, confirm and list patterns |
| `decision` | Add to and read the decision log |
| `doctor` | Health checks |
| `project` | Add and list projects |
| `capture` | Quick note to the inbox |
| `vault` | Print the vault path |
| `skills` | Install the skills |

Each command has its own `--help`.

## Development

```sh
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

CI runs on Linux, macOS and Windows.  
The `Makefile` needs a Unix shell. The Claude Code hooks in `.claude/settings.json` need `jq`.

## License

MIT, see [LICENSE](LICENSE).
