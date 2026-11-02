# Installing draupnir: instructions for an AI agent

You are installing draupnir for the user you are working with.  
draupnir is a CLI that keeps a markdown vault as long-term memory for Claude Code, plus a set of Claude Code skills that call it.  
Follow the steps in order. Do not skip the questions: the vault path and the project names are the user's to choose.

## 1. Look before installing

Run `draupnir --version` (Windows: also `where.exe draupnir`; elsewhere `command -v draupnir`).  
On Windows also check the installer's location directly, because a Claude Code session started before an install does not see it on `PATH`:  
`Test-Path "$env:LOCALAPPDATA\Programs\draupnir\draupnir.exe"` (PowerShell) or `ls "$LOCALAPPDATA/Programs/draupnir/draupnir.exe"` (Git Bash).

- **Not found:** a fresh install. Go on.
- **Found:** draupnir is already installed. Note where. If it is not in the location step 3 installs to, two copies would end up on `PATH` and the older one may win. Tell the user and ask whether to remove the old copy first.

Also run `draupnir vault root` if the binary exists. If it prints a path, a config exists: keep it, and do not ask the vault question below.

## 2. Ask the user

1. **Where should the vault live?** Suggest `~/Documents/Draupnir`. An existing Obsidian vault works too.  
   `~` is fine on every platform, so you do not need the user's account name.
2. **Which project should be registered first?** You need a short lowercase key (e.g. `acme`), a title (e.g. `Acme`) and the path of its repo. Offer the current repo if you are in one.

## 3. Install the binary

**Windows** (x86-64). The installer is a short PowerShell script; read it before running it if you want to know what it does:  
https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1

From Claude Code's Bash tool (Git Bash):

```sh
DRAUPNIR_VAULT_ROOT='~/Documents/Draupnir' powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1 | iex"
```

From PowerShell:

```powershell
$env:DRAUPNIR_VAULT_ROOT = '~/Documents/Draupnir'; irm https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1 | iex; Remove-Item Env:DRAUPNIR_VAULT_ROOT
```

Put the user's answer from step 2 in place of `~/Documents/Draupnir`.  
The script installs to `%LOCALAPPDATA%\Programs\draupnir`, adds that folder to the user `PATH`, writes the config if none exists and installs the skills. It warns if another `draupnir` is on `PATH`.  
Steps 4 and 5 are then done; go to step 6.

**PATH on Windows:** a running program keeps the `PATH` it started with, and Claude Code passes its own to every shell it opens. So `draupnir` will not resolve in this session, even in a new shell, until the user restarts Claude Code.  
Until then, call it by full path:

- Git Bash: `"$LOCALAPPDATA/Programs/draupnir/draupnir.exe"`
- PowerShell: `& "$env:LOCALAPPDATA\Programs\draupnir\draupnir.exe"`

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

## 4. Create the config (macOS and Linux)

Run `draupnir vault root`.

- It prints a path: a config exists. Leave it alone.
- It fails with `Config file not found at <path>`: create that file with one line, using the vault path from step 2:

```yaml
vault_root: '~/Documents/Draupnir'
```

When you write the file by hand, use single quotes: inside double quotes YAML reads `\` as an escape.  
`draupnir project add` rewrites the file later in its own style (no quotes, paths as given). That is fine.

## 5. Install the skills (macOS and Linux)

```sh
draupnir skills install
```

It copies the skills into `~/.claude/skills` (or `$CLAUDE_CONFIG_DIR/skills`).  
If it reports a skill as "locally modified, skipped", the user has edited that skill. Tell them, and pass `--force` only if they agree.

## 6. Register the project

From step 2's answers:

```sh
draupnir project add <key> --title "<Title>" --repo <path to the repo>
```

`--repo` is how draupnir recognises the project later: any directory inside that path resolves to this key.  
On Windows `C:\dev\acme`, `C:/dev/acme` and `~/dev/acme` all work; the comparison ignores slashes and case.  
Add `--detect <text>` for a pattern that should match too, for example a folder name shared by several clones.

## 7. Check

Run these from inside the project's repo:

```sh
draupnir doctor --fix
draupnir doctor
```

The second run should report `0 error(s)`.  
On a fresh vault it also warns that the instincts file and the inbox do not exist yet (older versions add a third warning about the brief). These appear as the user works; they are not problems.  
`--fix` builds the empty session index, so the first run may show `[FIXED]`. Any error after `--fix` is a real problem: show it to the user.

If `doctor` says `Could not detect project from cwd`, the `--repo` path does not match the current directory. Compare the two (symlinked folders are a common cause) and fix it with the user.

## 8. Tell the user

- Where the binary and the config are, the vault path, and the version (`draupnir --version`).
- The skills: `/draupnir-resume` at the start of a session, `/draupnir-compress` at the end, `/draupnir-capture` for quick notes.
- Restart Claude Code: skills load at startup, and on Windows that also picks up the new `PATH`.

## Uninstall

Only when the user asks.  
First run `draupnir vault root` and note the path: once the binary or the config is gone, nothing can tell you where the vault is.  
Then remove what they want gone:

| What | Windows | macOS / Linux |
|---|---|---|
| Binary and PATH entry | `& ([scriptblock]::Create((irm https://raw.githubusercontent.com/yefimtsev/draupnir/master/install.ps1))) -Uninstall` | `rm ~/.local/bin/draupnir` |
| Config | `~\.config\draupnir\config.yaml` or `%APPDATA%\draupnir\config.yaml` | `~/.config/draupnir/config.yaml` |
| Skills | `~\.claude\skills\draupnir-*` and `instinct-*` | `~/.claude/skills/draupnir-*` and `instinct-*` |
| Vault | the folder `draupnir vault root` prints | same |

The vault holds the user's notes: confirm before deleting it, and prefer moving it to the trash.
