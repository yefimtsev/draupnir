---
name: draupnir-release
description: Bump version, build release, tag, and install draupnir binary
disable-model-invocation: true
---

# Release Draupnir CLI

Cut a new release of the draupnir CLI. Accepts a version bump argument: `patch`, `minor`, or `major`.

## Usage

`/draupnir-release patch` (or `minor` or `major`)

## Steps

1. **Pre-flight checks**:
   - Ensure working tree is clean (`git status --porcelain` is empty)
   - Ensure on `master` branch
   - Run `make ci` (fmt-check + clippy + test) — abort if anything fails

2. **Version bump**:
   - Read current version from `Cargo.toml`
   - Compute new version based on the argument (patch/minor/major)
   - Update `version` in `Cargo.toml`
   - Run `cargo check` to update `Cargo.lock`

3. **Build & install**:
   - Run `make build` (release build)
   - Run `make install` (symlinks ~/.local/bin/draupnir to target/release/draupnir, so any
     later `cargo build --release` changes the installed binary too)

4. **Tag & commit**:
   - Ask user to confirm the version bump before committing
   - `git add Cargo.toml Cargo.lock`
   - `git commit -m "release: draupnir-cli vX.Y.Z"`
   - `git tag -m "draupnir-cli vX.Y.Z — <one-line summary>" vX.Y.Z` (the repo has
     `tag.gpgsign=true`, so a bare `git tag` fails with "no tag message")

5. **Report**:
   - Print old version -> new version
   - Print binary size
   - Print install path
   - Remind user to `git push && git push origin vX.Y.Z` if they want to publish (not
     `--tags`, which also pushes any stray local tags)
   - After a push, wait for CI on the release commit and report its result

6. **GitHub Release** (after the tag is pushed):
   - Draft notes from `git log --no-merges <previous tag>..vX.Y.Z` in the format of the earlier
     releases (`gh release view v0.6.0`): a `##` title, then sections such as New Commands / Fixes /
     Housekeeping. Write the notes to a file and show them before publishing.
   - `gh release create vX.Y.Z --verify-tag --title vX.Y.Z --notes-file <file> --latest`
   - Check `gh release list` against `git tag -l`: every tag needs a release.
   - Publishing the release triggers `.github/workflows/release.yml`. Wait for it
     (`gh run list --workflow release.yml --limit 1`) and check that `gh release view vX.Y.Z`
     lists three assets: `draupnir-x86_64-unknown-linux-gnu`, `draupnir-aarch64-apple-darwin`
     and `draupnir-x86_64-pc-windows-msvc.exe`.
