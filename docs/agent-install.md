# Installing draupnir: instructions for an AI agent

You are installing draupnir for the user you are working with.
draupnir is a CLI that keeps a markdown vault as long-term memory for Claude Code, plus a set of Claude Code skills that call it.
Follow the steps in order. Do not skip the questions: the vault path and the project names are the user's to choose.

## 1. Ask the user

Before running anything, ask:

1. **Where should the vault live?** Suggest `~/Documents/Draupnir`. An existing Obsidian vault works too.
2. **Which project should be registered first?** You need a short lowercase key (e.g. `acme`), a title (e.g. `Acme`) and the path of its repo. Offer the current repo if you are in one.

If draupnir is already installed (`draupnir vault root` prints a path), the vault question is settled: do not ask it and do not touch the config.

## 2. Install the binary

Pick the block for the user's platform.

**Windows** (x86-64). Run from Claude Code's Bash tool (Git Bash) or from PowerShell, with the vault path from step 1:

```sh
DRAUPNIR_VAULT_ROOT='C:/Users/<user>/Documents/Draupnir' powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1 | iex"
```

The script installs to `%LOCALAPPDATA%\Programs\draupnir`, adds that folder to the user `PATH`, writes the config if none exists and installs the skills.
Steps 3 and 4 are then already done; go to step 5.
The new `PATH` reaches new terminals only. Until then call the binary by its full path: `"$LOCALAPPDATA/Programs/draupnir/draupnir.exe"` in Git Bash.

**macOS on Apple silicon:**

```sh
mkdir -p ~/.local/bin
curl -fsSL -o ~/.local/bin/draupnir https://github.com/yefimtsev/draupnir/releases/latest/download/draupnir-aarch64-apple-darwin
chmod +x ~/.local/bin/draupnir
```

**Linux** (x86-64):

```sh
mkdir -p ~/.local/bin
curl -fsSL -o ~/.local/bin/draupnir https://github.com/yefimtsev/draupnir/releases/latest/download/draupnir-x86_64-unknown-linux-gnu
chmod +x ~/.local/bin/draupnir
```

**Anything else** (Intel Mac, ARM Linux): `cargo install --git https://github.com/yefimtsev/draupnir`. This needs a Rust toolchain; ask the user before installing one.

On macOS and Linux, check that `~/.local/bin` is on `PATH` (`command -v draupnir`). If it is not, tell the user which shell profile line to add rather than editing their profile yourself.

## 3. Create the config (macOS and Linux)

Run `draupnir vault root`.

- It prints a path: a config exists. Leave it alone.
- It fails with `Config file not found at <path>`: create that file with one line, using the vault path from step 1:

```yaml
vault_root: '~/Documents/Draupnir'
```

Use single quotes. On Windows paths use `/`, not `\`.

## 4. Install the skills (macOS and Linux)

```sh
draupnir skills install
```

It copies the skills into `~/.claude/skills` (or `$CLAUDE_CONFIG_DIR/skills`).
If it reports a skill as "locally modified, skipped", the user has edited that skill. Tell them, and pass `--force` only if they agree.

## 5. Register the project

From step 1's answers:

```sh
draupnir project add <key> --title "<Title>" --repo <path to the repo>
```

`--repo` is how draupnir recognises the project later: any directory inside that path resolves to this key.
Add `--detect <text>` for a pattern that should match too, for example a folder name shared by several clones.

## 6. Check

Run these from inside the project's repo:

```sh
draupnir doctor --fix
draupnir doctor
```

On a fresh vault the second run reports `0 errors, 3 warning(s)`: no brief yet, no instincts file yet, no inbox yet.
These fill in as the user works. Any error is a real problem: show it to the user.

If `doctor` says `Could not detect project from cwd`, the `--repo` path does not match the current directory. Compare the two (symlinked folders are a common cause) and fix it with the user.

## 7. Tell the user

- Where the binary and the config are, and the vault path.
- The skills: `/draupnir-resume` at the start of a session, `/draupnir-compress` at the end, `/draupnir-capture` for quick notes.
- Skills are loaded when a Claude Code session starts, so they appear in the next session.
- Uninstall on Windows: `& ([scriptblock]::Create((irm https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1))) -Uninstall` in PowerShell. Elsewhere: delete `~/.local/bin/draupnir`. The config, the vault and the skills stay in place either way.
