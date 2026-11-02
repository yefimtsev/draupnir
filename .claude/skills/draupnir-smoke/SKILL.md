---
name: draupnir-smoke
description: Build the release binary and smoke-test it against the real vault, read-only
disable-model-invocation: true
---

# Smoke Test

Build the release binary and run smoke tests to verify core commands work.

## Steps

1. Run `cargo build --release`
2. Run `make smoke` to exercise `--help` and `doctor` commands
3. Pick a project key from `./target/release/draupnir project list` (ask if there are several),
   then test these commands against the real vault with it as `<key>`:
   - `./target/release/draupnir session search --project <key> --limit 3`
   - `./target/release/draupnir instincts list --project <key>`
   - `./target/release/draupnir brief show --project <key>`
4. Report pass/fail for each command

If any command panics or exits with an unexpected error, report the full error output.
