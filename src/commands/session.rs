use crate::models::config::Config;
use crate::models::session::SessionIndex;
use crate::style;
use crate::vault;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::io::Read;
use std::path::PathBuf;

/// Write a session log from stdin to the vault.
///
/// Returns the path to the written file.
pub fn write_session(
    config: &Config,
    project_key: &str,
    topic: &str,
    tags: &[String],
) -> Result<PathBuf> {
    let vault_root = config.resolved_vault_root()?;
    let logs_dir = vault::session_logs_dir(&vault_root, &config.session_logs_dir);

    if !logs_dir.exists() {
        std::fs::create_dir_all(&logs_dir)
            .with_context(|| format!("Creating session logs dir: {}", logs_dir.display()))?;
    }

    // Read content from stdin
    let mut content = String::new();
    std::io::stdin()
        .read_to_string(&mut content)
        .context("Reading session content from stdin")?;
    let content = vault::normalize_newlines(&content);

    if content.trim().is_empty() {
        anyhow::bail!("No content received on stdin. Pipe session log content.");
    }

    // Build filename: YYYY-MM-DD-HHMM-project-topic.md
    let date = vault::today();
    let time = vault::now_hhmm();
    let slug = topic_slug(topic, project_key);
    let filename = format!(
        "{}-{}-{}-{}.md",
        date.format("%Y-%m-%d"),
        time,
        project_key,
        slug
    );
    let file_path = logs_dir.join(&filename);

    let final_content = if content.starts_with("---") {
        set_frontmatter_project(&content, project_key)
    } else {
        // Wrap with frontmatter
        let tags_yaml = if tags.is_empty() {
            "[session-log]".to_string()
        } else {
            let mut all_tags = vec!["session-log".to_string()];
            all_tags.extend(tags.iter().cloned());
            format!(
                "[{}]",
                all_tags
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!(
            "---\ndate: {}\ntime: {}\nproject: {}\ntopic: {}\ntags: {}\n---\n\n{}",
            date.format("%Y-%m-%d"),
            time[..2].to_string() + ":" + &time[2..],
            project_key,
            slug,
            tags_yaml,
            content
        )
    };

    std::fs::write(&file_path, final_content)
        .with_context(|| format!("Writing session log to {}", file_path.display()))?;

    println!("{}", style::dim(&file_path.display().to_string()));
    Ok(file_path)
}

/// Slugify a topic, dropping a leading project key so filenames carry it once.
fn topic_slug(topic: &str, project_key: &str) -> String {
    let slug = topic
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();
    let key_prefix = format!("{}-", project_key.to_lowercase());
    let mut rest = slug.as_str();
    while let Some(stripped) = rest.strip_prefix(&key_prefix) {
        rest = stripped;
    }
    rest.to_string()
}

/// Force `project: <key>` into existing frontmatter. The resolved key is authoritative:
/// a skill that wrote its own frontmatter may have guessed the project differently.
fn set_frontmatter_project(content: &str, project_key: &str) -> String {
    let Some(end) = content[3..].find("\n---").map(|i| i + 3) else {
        return content.to_string();
    };
    let mut out = format!("---\nproject: {}", project_key);
    for field in content[3..end]
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("project:"))
    {
        out.push('\n');
        out.push_str(field);
    }
    out.push_str(&content[end..]);
    out
}

/// Search the session index with filters.
#[allow(clippy::too_many_arguments)]
pub fn search(
    config: &Config,
    project_key: &str,
    query: Option<&str>,
    tag: Option<&str>,
    ticket: Option<&str>,
    since: Option<&str>,
    limit: usize,
    json: bool,
) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);
    let index_path = gen_dir.join("session-index.json");

    if !index_path.exists() {
        if json {
            println!("[]");
        } else {
            println!("No session index found. Run `draupnir index rebuild`.");
        }
        return Ok(());
    }

    let content = std::fs::read_to_string(&index_path)?;
    let index: SessionIndex = serde_json::from_str(&content)?;

    // Parse --since date if provided
    let since_date = if let Some(s) = since {
        Some(
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .with_context(|| format!("Invalid --since date '{}'. Expected YYYY-MM-DD.", s))?,
        )
    } else {
        None
    };

    let results: Vec<_> = index
        .sessions
        .into_iter()
        .filter(|entry| {
            // --query: case-insensitive substring on title
            if let Some(q) = query
                && !entry.title.to_lowercase().contains(&q.to_lowercase())
            {
                return false;
            }
            // --tag: case-insensitive exact match
            if let Some(t) = tag
                && !entry.tags.iter().any(|et| et.eq_ignore_ascii_case(t))
            {
                return false;
            }
            // --ticket: case-insensitive exact match
            if let Some(tk) = ticket
                && !entry
                    .ticket_ids
                    .iter()
                    .any(|et| et.eq_ignore_ascii_case(tk))
            {
                return false;
            }
            // --since: date comparison (skip unparseable dates)
            if let Some(since_d) = since_date {
                match NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d") {
                    Ok(entry_date) => {
                        if entry_date < since_d {
                            return false;
                        }
                    }
                    Err(_) => return false,
                }
            }
            true
        })
        .take(limit)
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        if results.is_empty() {
            println!("No matching sessions.");
        }
        for entry in &results {
            let tags_str = if entry.tags.is_empty() {
                String::new()
            } else {
                format!(
                    " [{}]",
                    entry
                        .tags
                        .iter()
                        .map(|t| style::tag(t))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            println!(
                "{} {} {}{}",
                style::dim(&entry.date),
                style::dim("|"),
                style::heading(&entry.title),
                tags_str,
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_drops_repeated_project_prefix() {
        assert_eq!(topic_slug("Rocket F1 landed", "rocket"), "f1-landed");
        assert_eq!(topic_slug("rocket rocket x", "rocket"), "x");
        assert_eq!(topic_slug("Acme gate", "rocket"), "acme-gate");
    }

    #[test]
    fn frontmatter_gets_project_added_or_overwritten() {
        let added = set_frontmatter_project("---\ndate: 2026-10-10\n---\n\n# T\n", "rocket");
        assert_eq!(
            added,
            "---\nproject: rocket\ndate: 2026-10-10\n---\n\n# T\n"
        );

        let fixed = set_frontmatter_project("---\nproject: acme\ntags: [a]\n---\nbody", "rocket");
        assert_eq!(fixed, "---\nproject: rocket\ntags: [a]\n---\nbody");
    }
}
