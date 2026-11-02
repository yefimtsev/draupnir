use crate::style;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Skills shipped with this binary, embedded at build time so an installed `draupnir`
/// always writes the skills that match its own flags.
pub const SKILLS: &[(&str, &str)] = &[
    (
        "draupnir-capture",
        include_str!("../../skills/draupnir-capture/SKILL.md"),
    ),
    (
        "draupnir-compress",
        include_str!("../../skills/draupnir-compress/SKILL.md"),
    ),
    (
        "draupnir-doctor",
        include_str!("../../skills/draupnir-doctor/SKILL.md"),
    ),
    (
        "draupnir-resume",
        include_str!("../../skills/draupnir-resume/SKILL.md"),
    ),
    (
        "instinct-promote",
        include_str!("../../skills/instinct-promote/SKILL.md"),
    ),
    (
        "instinct-status",
        include_str!("../../skills/instinct-status/SKILL.md"),
    ),
];

/// Default target: `$CLAUDE_CONFIG_DIR/skills`, else `~/.claude/skills`.
pub fn default_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("CLAUDE_CONFIG_DIR") {
        return Ok(PathBuf::from(dir).join("skills"));
    }
    let home = crate::vault::home_dir()?;
    Ok(home.join(".claude").join("skills"))
}

#[derive(Debug, PartialEq)]
enum Outcome {
    Installed,
    Updated,
    Unchanged,
    /// Local copy differs; left alone without --force.
    Modified,
    /// A symlink (dev checkout via `make link-skills`); never written through.
    Linked,
}

pub fn install(dir: Option<&Path>, force: bool) -> Result<()> {
    let dir = match dir {
        Some(d) => d.to_path_buf(),
        None => default_dir()?,
    };
    let mut skipped = 0;
    for (name, content) in SKILLS {
        let outcome = install_one(&dir, name, content, force)?;
        let (icon, note) = match outcome {
            Outcome::Installed => (style::icon_ok(), "installed"),
            Outcome::Updated => (style::icon_ok(), "updated"),
            Outcome::Unchanged => (style::icon_ok(), "unchanged"),
            Outcome::Linked => (style::icon_ok(), "symlink, left alone"),
            Outcome::Modified => {
                skipped += 1;
                (
                    style::icon_warn(),
                    "locally modified, skipped (use --force)",
                )
            }
        };
        println!("{icon} {name} {}", style::dim(note));
    }
    println!("{}", style::dim(&dir.display().to_string()));
    if skipped > 0 {
        anyhow::bail!("{skipped} skill(s) skipped because they were modified locally");
    }
    Ok(())
}

fn install_one(dir: &Path, name: &str, content: &str, force: bool) -> Result<Outcome> {
    let skill_dir = dir.join(name);
    if skill_dir.is_symlink() {
        return Ok(Outcome::Linked);
    }
    let file = skill_dir.join("SKILL.md");
    let outcome = match std::fs::read_to_string(&file) {
        Ok(existing) if existing == content => return Ok(Outcome::Unchanged),
        Ok(_) if !force => return Ok(Outcome::Modified),
        Ok(_) => Outcome::Updated,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Outcome::Installed,
        Err(e) => return Err(e).with_context(|| format!("Reading {}", file.display())),
    };
    std::fs::create_dir_all(&skill_dir)
        .with_context(|| format!("Creating {}", skill_dir.display()))?;
    std::fs::write(&file, content).with_context(|| format!("Writing {}", file.display()))?;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Every `draupnir ...` invocation a skill documents, placeholders filled with `x`.
    fn invocations(skill: &str) -> Vec<String> {
        let placeholder = regex::Regex::new(r"\{[^}]*\}").unwrap();
        let inline = regex::Regex::new(r"`(draupnir [^`]+)`").unwrap();
        let mut out = Vec::new();
        let mut in_bash = false;
        for line in skill.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_bash = !in_bash && trimmed == "```bash";
                continue;
            }
            if in_bash {
                if let Some(at) = trimmed.find("draupnir ") {
                    out.push(trimmed[at..].trim_end_matches("<<'EOF'").to_string());
                }
            } else {
                out.extend(inline.captures_iter(line).map(|c| c[1].to_string()));
            }
        }
        out.into_iter()
            .map(|cmd| placeholder.replace_all(&cmd, "x").into_owned())
            .collect()
    }

    /// Splits on whitespace, honouring double quotes. Enough for the commands skills show.
    fn split(cmd: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut cur = String::new();
        let mut quoted = false;
        for c in cmd.chars() {
            match c {
                '"' => quoted = !quoted,
                c if c.is_whitespace() && !quoted => {
                    if !cur.is_empty() {
                        args.push(std::mem::take(&mut cur));
                    }
                }
                c => cur.push(c),
            }
        }
        if !cur.is_empty() {
            args.push(cur);
        }
        args
    }

    #[test]
    fn every_documented_command_parses() {
        let mut checked = 0;
        for (name, content) in SKILLS {
            for cmd in invocations(content) {
                if let Err(e) = crate::Cli::try_parse_from(split(&cmd)) {
                    panic!("{name}: `{cmd}` no longer parses:\n{e}");
                }
                checked += 1;
            }
        }
        assert!(checked > 20, "extractor found only {checked} commands");
    }

    #[test]
    fn frontmatter_name_matches_dir() {
        for (name, content) in SKILLS {
            assert!(
                content.starts_with(&format!("---\nname: {name}\n")),
                "{name}: frontmatter name mismatch"
            );
        }
    }

    #[test]
    fn install_never_clobbers_edits_or_links() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        assert_eq!(
            install_one(dir, "s", "v1", false).unwrap(),
            Outcome::Installed
        );
        assert_eq!(
            install_one(dir, "s", "v1", false).unwrap(),
            Outcome::Unchanged
        );
        assert_eq!(
            install_one(dir, "s", "v2", false).unwrap(),
            Outcome::Modified
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("s/SKILL.md")).unwrap(),
            "v1"
        );
        assert_eq!(install_one(dir, "s", "v2", true).unwrap(), Outcome::Updated);
    }

    // Windows symlinks need Developer Mode or admin, so the link case is Unix-only.
    #[cfg(unix)]
    #[test]
    fn install_never_writes_through_links() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let src = dir.join("checkout");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("SKILL.md"), "dev").unwrap();
        std::os::unix::fs::symlink(&src, dir.join("l")).unwrap();
        assert_eq!(install_one(dir, "l", "v2", true).unwrap(), Outcome::Linked);
        assert_eq!(
            std::fs::read_to_string(src.join("SKILL.md")).unwrap(),
            "dev"
        );
    }
}
