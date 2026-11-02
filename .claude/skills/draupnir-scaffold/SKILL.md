---
name: draupnir-scaffold
description: Scaffold a new draupnir CLI subcommand with clap derive struct, matching the existing command pattern
disable-model-invocation: true
---

# Scaffold a New Subcommand

Create a new draupnir CLI subcommand following the established pattern.

## Usage

`/draupnir-scaffold <command-name>`

## Steps

1. Read `src/commands/capture.rs` as the template reference for the pattern
2. Create `src/commands/<name>.rs` with:
   - A `pub fn run(config: &Config, project_key: &str, ...) -> anyhow::Result<()>` function
   - Proper imports (`use crate::models::config::Config;`, `use anyhow::Result;`, etc.)
3. Add `pub mod <name>;` to `src/commands/mod.rs` (alphabetical order)
4. Add a new variant to the `Command` enum in `src/main.rs` with `#[command(flatten)]` args inline (clap structs live in `main.rs`, not the command module)
5. Wire the match arm in `main.rs` to call `commands::<name>::run(&config, &project_key, ...)`
6. Run `cargo check` to verify it compiles

Keep the generated code minimal — just the skeleton. Let the user fill in the logic.
