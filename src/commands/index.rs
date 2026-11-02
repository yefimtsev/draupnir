use crate::models::config::Config;
use crate::models::session::{SessionEntry, SessionIndex};
use crate::style;
use crate::vault;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static FRONTMATTER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^---\n(.+?)\n---").unwrap());
static FIELD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\w[\w_]*)\s*:\s*(.+)$").unwrap());
static TITLE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#\s+(.+)$").unwrap());
static PR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"#(\d+)").unwrap());
// Ticket regex is now dynamic per-project via ProjectConfig.prefix

/// Core rebuild logic — returns (path, session_count) without printing.
pub fn rebuild_core(config: &Config, project_key: &str) -> Result<(PathBuf, usize)> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let logs_dir = vault::session_logs_dir(&vault_root, &config.session_logs_dir);
    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);

    if !gen_dir.exists() {
        std::fs::create_dir_all(&gen_dir)?;
    }

    // The filename glob only narrows candidates; frontmatter decides, so a misfiled
    // `acme-rocket-*` log matches both globs but lands in one index. Don't read every
    // log: most are iCloud dataless placeholders and each read forces a download.
    let pattern = format!("{}/*-{}*.md", logs_dir.display(), project_key);
    let mut entries: Vec<SessionEntry> = Vec::new();

    for path in glob::glob(&pattern)? {
        let path = path?;
        if let Ok(entry) = parse_session_file(config, &path)
            && entry.project == project_key
        {
            entries.push(entry);
        }
    }

    // Sort by date descending (newest first)
    entries.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.id.cmp(&a.id)));

    let now = vault::now_iso();
    let count = entries.len();
    let index = SessionIndex {
        generated_at: now,
        count,
        sessions: entries,
    };

    let index_path = gen_dir.join("session-index.json");
    let json = serde_json::to_string_pretty(&index)?;
    std::fs::write(&index_path, json)
        .with_context(|| format!("Writing index to {}", index_path.display()))?;

    Ok((index_path, count))
}

/// Rebuild the entire session index for a project from session log files.
pub fn rebuild(config: &Config, project_key: &str) -> Result<PathBuf> {
    let (index_path, count) = rebuild_core(config, project_key)?;
    println!(
        "{} Rebuilt index: {} sessions → {}",
        style::icon_ok(),
        style::success(&count.to_string()),
        style::dim(&index_path.display().to_string()),
    );
    Ok(index_path)
}

/// Append a single session file to the existing index (or create if missing).
pub fn append(config: &Config, project_key: &str, file_path: &Path) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);
    let index_path = gen_dir.join("session-index.json");

    let mut index = if index_path.exists() {
        let content = std::fs::read_to_string(&index_path)?;
        serde_json::from_str::<SessionIndex>(&content)?
    } else {
        if !gen_dir.exists() {
            std::fs::create_dir_all(&gen_dir)?;
        }
        SessionIndex {
            generated_at: vault::now_iso(),
            count: 0,
            sessions: Vec::new(),
        }
    };

    let entry = parse_session_file(config, file_path)
        .with_context(|| format!("Parsing session file: {}", file_path.display()))?;
    if entry.project != project_key {
        anyhow::bail!(
            "{} belongs to project '{}', not '{}'",
            file_path.display(),
            entry.project,
            project_key
        );
    }

    // Deduplicate by ID, insert, and re-sort to maintain date-descending order
    index.sessions.retain(|s| s.id != entry.id);
    index.sessions.push(entry);
    index
        .sessions
        .sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.id.cmp(&a.id)));
    index.count = index.sessions.len();
    index.generated_at = vault::now_iso();

    let json = serde_json::to_string_pretty(&index)?;
    std::fs::write(&index_path, &json)?;

    println!(
        "{} Appended to index: {} sessions",
        style::icon_ok(),
        style::success(&index.count.to_string()),
    );
    Ok(())
}

/// Print index stats.
pub fn stats(config: &Config, project_key: &str) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);
    let index_path = gen_dir.join("session-index.json");

    if !index_path.exists() {
        println!("No session index found. Run `draupnir index rebuild`.");
        return Ok(());
    }

    let content = std::fs::read_to_string(&index_path)?;
    let index: SessionIndex = serde_json::from_str(&content)?;

    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    let mut ticket_count = 0;
    let mut pr_count = 0;

    for session in &index.sessions {
        for tag in &session.tags {
            *tag_counts.entry(tag.clone()).or_default() += 1;
        }
        ticket_count += session.ticket_ids.len();
        pr_count += session.linked_prs.len();
    }

    println!(
        "{} {}",
        style::label("Sessions:"),
        style::heading(&index.count.to_string())
    );
    println!(
        "{} {}",
        style::label("Generated:"),
        style::dim(&index.generated_at)
    );
    println!("{} {}", style::label("Linked tickets:"), ticket_count);
    println!("{} {}", style::label("Linked PRs:"), pr_count);

    let mut tags: Vec<_> = tag_counts.into_iter().collect();
    tags.sort_by_key(|t| std::cmp::Reverse(t.1));
    println!("\n{}:", style::label("Top tags"));
    for (t, count) in tags.iter().take(10) {
        println!("  {}: {}", style::tag(t), count);
    }

    Ok(())
}

/// Parse a single session log file into a SessionEntry. The project comes from the log
/// itself: frontmatter `project:` (authoritative), else the longest configured key the
/// filename starts with after `YYYY-MM-DD-HHMM-`. Never from the caller's project.
fn parse_session_file(config: &Config, path: &Path) -> Result<SessionEntry> {
    let content = vault::read_text(path)?;
    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let mut date = String::new();
    let mut fm_project: Option<String> = None;
    let mut tags = Vec::new();
    let mut title = String::new();

    // Parse frontmatter
    if let Some(fm_match) = FRONTMATTER_RE.captures(&content) {
        let fm = &fm_match[1];
        for line in fm.lines() {
            if let Some(caps) = FIELD_RE.captures(line) {
                let key = &caps[1];
                let value = caps[2].trim();
                match key {
                    "date" => date = value.to_string(),
                    "project" => fm_project = Some(value.to_string()),
                    "tags" => {
                        tags = value
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .split(',')
                            .map(|t| t.trim().to_string())
                            .filter(|t| !t.is_empty())
                            .collect();
                    }
                    _ => {}
                }
            }
        }
    }

    // Find first H1 for title
    for line in content.lines() {
        if let Some(caps) = TITLE_RE.captures(line) {
            title = caps[1].to_string();
            break;
        }
    }

    if title.is_empty() {
        title = filename.to_string();
    }

    let project = match fm_project {
        Some(p) => p,
        None => {
            let slug = filename.splitn(5, '-').nth(4).unwrap_or("");
            config
                .projects
                .keys()
                .filter(|k| slug.starts_with(&format!("{}-", k)))
                .max_by_key(|k| k.len())
                .cloned()
                .with_context(|| format!("No project for {}", path.display()))?
        }
    };
    let prefix = config.project(&project).and_then(|p| p.prefix.as_deref());

    // Extract ticket IDs from content (only if project has a prefix)
    let ticket_ids: Vec<String> = if let Some(pfx) = prefix {
        let pattern = format!(r"(?i)({}[-–]\d+)", regex::escape(pfx));
        let ticket_re = Regex::new(&pattern)?;
        ticket_re
            .captures_iter(&content)
            .map(|c| c[1].to_uppercase())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    } else {
        Vec::new()
    };

    // Extract PR references from content
    let linked_prs: Vec<String> = PR_RE
        .captures_iter(&content)
        .map(|c| format!("#{}", &c[1]))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Vault-relative with `/`, the same on every platform; a path outside the vault stays absolute.
    let vault_root = config.resolved_vault_root()?;
    let rel_path = match path.strip_prefix(&vault_root) {
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => path.to_string_lossy().to_string(),
    };

    // Validate date — reject entries with empty or unparseable dates
    if date.is_empty() {
        anyhow::bail!("Missing date in frontmatter of {}", path.display());
    }
    if chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
        anyhow::bail!(
            "Invalid date '{}' in frontmatter of {}",
            date,
            path.display()
        );
    }

    Ok(SessionEntry {
        id: filename.to_string(),
        project,
        date,
        ticket_ids,
        title,
        tags,
        path: rel_path,
        linked_prs,
        generated_at: vault::now_iso(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::ProjectConfig;

    fn project(path: &str) -> ProjectConfig {
        ProjectConfig {
            title: String::new(),
            prefix: None,
            path: path.to_string(),
            repos: vec![],
            detect_patterns: vec![],
        }
    }

    #[test]
    fn index_takes_project_from_the_log_not_the_caller() {
        let vault = tempfile::tempdir().unwrap();
        let config = Config {
            vault_root: vault.path().to_string_lossy().to_string(),
            session_logs_dir: "Logs".to_string(),
            projects: [
                ("acme".to_string(), project("P/Acme")),
                ("rocket".to_string(), project("P/Rocket")),
            ]
            .into_iter()
            .collect(),
        };
        let logs = vault.path().join("Logs");
        std::fs::create_dir_all(&logs).unwrap();
        // Filename says acme, frontmatter says rocket: frontmatter wins. CRLF on purpose:
        // a Windows checkout or editor must not hide the frontmatter.
        std::fs::write(
            logs.join("2026-10-10-1150-acme-rocket-clock.md"),
            "---\r\ndate: 2026-10-10\r\nproject: rocket\r\n---\r\n# Clock\r\n",
        )
        .unwrap();
        // No frontmatter project: filename decides.
        std::fs::write(
            logs.join("2026-09-26-2110-acme-gate0.md"),
            "---\ndate: 2026-09-26\n---\n# Gate\n",
        )
        .unwrap();

        let paths = |key: &str| -> Vec<String> {
            rebuild_core(&config, key).unwrap();
            let path = vault.path().join(format!(
                "P/{}/_generated/session-index.json",
                if key == "acme" { "Acme" } else { "Rocket" }
            ));
            let index: SessionIndex =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            index.sessions.into_iter().map(|s| s.path).collect()
        };
        // Relative to the vault with `/` separators, also on Windows.
        assert_eq!(paths("acme"), ["Logs/2026-09-26-2110-acme-gate0.md"]);

        let ids = |key: &str| -> Vec<String> {
            rebuild_core(&config, key).unwrap();
            let path = vault.path().join(format!(
                "P/{}/_generated/session-index.json",
                if key == "acme" { "Acme" } else { "Rocket" }
            ));
            let index: SessionIndex =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            assert!(index.sessions.iter().all(|s| s.project == key));
            index.sessions.into_iter().map(|s| s.id).collect()
        };
        assert_eq!(ids("rocket"), ["2026-10-10-1150-acme-rocket-clock"]);
        assert_eq!(ids("acme"), ["2026-09-26-2110-acme-gate0"]);

        let err = append(
            &config,
            "acme",
            &logs.join("2026-10-10-1150-acme-rocket-clock.md"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("rocket"));
    }
}
