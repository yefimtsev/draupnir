use crate::models::brief::BriefMeta;
use crate::models::config::Config;
use crate::models::session::SessionIndex;
use crate::style;
use crate::vault;
use anyhow::{Context, Result};
use std::path::PathBuf;

const BRIEF_FILENAME: &str = "current-brief.md";
const BRIEF_META_FILENAME: &str = "current-brief-meta.json";
const RECENT_SESSION_COUNT: usize = 10;

/// Resolve the repo string: explicit > config repos[0] > "unknown".
fn resolve_repo(config: &Config, project_key: &str, explicit: Option<&str>) -> String {
    if let Some(r) = explicit {
        return r.to_string();
    }
    if let Some(pc) = config.project(project_key)
        && let Some(first) = pc.repos.first()
    {
        return first.clone();
    }
    "unknown".to_string()
}

/// Core generate logic — returns path without printing.
pub fn generate_core(config: &Config, project_key: &str, repo: Option<&str>) -> Result<PathBuf> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);
    let index_path = gen_dir.join("session-index.json");

    if !index_path.exists() {
        anyhow::bail!(
            "No session index found at {}. Run `draupnir index rebuild` first.",
            index_path.display()
        );
    }

    let index_content = std::fs::read_to_string(&index_path)?;
    let index: SessionIndex = serde_json::from_str(&index_content)?;

    // Take the most recent N sessions
    let recent: Vec<_> = index.sessions.iter().take(RECENT_SESSION_COUNT).collect();

    if recent.is_empty() {
        anyhow::bail!(
            "No sessions found in the index for project '{}'.",
            project_key
        );
    }

    let logs_dir = vault::session_logs_dir(&vault_root, &config.session_logs_dir);

    // Build brief content
    let mut brief = String::new();
    brief.push_str(&format!("# Current Brief — {}\n\n", project_config.title));
    brief.push_str(&format!("Generated: {}\n\n", vault::now_iso()));
    brief.push_str(&format!(
        "## Recent Sessions ({} most recent)\n\n",
        recent.len()
    ));

    for session in &recent {
        brief.push_str(&format!("### {} ({})\n", session.title, session.date));
        if !session.ticket_ids.is_empty() {
            brief.push_str(&format!("Tickets: {}\n", session.ticket_ids.join(", ")));
        }
        if !session.linked_prs.is_empty() {
            brief.push_str(&format!("PRs: {}\n", session.linked_prs.join(", ")));
        }
        if !session.tags.is_empty() {
            brief.push_str(&format!("Tags: {}\n", session.tags.join(", ")));
        }

        // Try to read the session file and extract key sections
        let session_path = logs_dir.join(
            std::path::Path::new(&session.path)
                .file_name()
                .unwrap_or_default(),
        );
        if session_path.exists()
            && let Ok(content) = vault::read_text(&session_path)
        {
            for section in ["Decisions Made", "Key Learnings", "Open Questions"] {
                if let Some(extracted) = extract_section(&content, section) {
                    brief.push_str(&format!("\n**{}:**\n{}\n", section, extracted));
                }
            }
        }

        brief.push('\n');
    }

    // Write brief
    let brief_path = gen_dir.join(BRIEF_FILENAME);
    std::fs::write(&brief_path, &brief)?;

    // Write meta
    let resolved_repo = resolve_repo(config, project_key, repo);
    let meta = BriefMeta {
        project: project_key.to_string(),
        generated_at: vault::now_iso(),
        repo: resolved_repo,
        session_count: recent.len(),
        latest_session_id: recent.first().map(|s| s.id.clone()),
    };
    let meta_path = gen_dir.join(BRIEF_META_FILENAME);
    std::fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;

    Ok(brief_path)
}

/// Generate a brief from recent sessions. The brief is a markdown summary
/// that skills like /draupnir-resume consume.
pub fn generate(config: &Config, project_key: &str, repo: Option<&str>) -> Result<PathBuf> {
    let brief_path = generate_core(config, project_key, repo)?;
    // Read back session count from meta for the message
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);
    let meta_content = std::fs::read_to_string(gen_dir.join(BRIEF_META_FILENAME))?;
    let meta: BriefMeta = serde_json::from_str(&meta_content)?;
    println!(
        "{} Brief generated: {} sessions → {}",
        style::icon_ok(),
        style::success(&meta.session_count.to_string()),
        style::dim(&brief_path.display().to_string()),
    );
    Ok(brief_path)
}

/// Check brief staleness.
pub fn status(config: &Config, project_key: &str, repo: Option<&str>, json: bool) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);
    let meta_path = gen_dir.join(BRIEF_META_FILENAME);

    if !meta_path.exists() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "exists": false,
                    "stale": true,
                    "reason": "No brief found"
                })
            );
        } else {
            println!("No brief found. Run `draupnir brief generate`.");
        }
        return Ok(());
    }

    let meta_content = std::fs::read_to_string(&meta_path)?;
    let meta: BriefMeta = serde_json::from_str(&meta_content)?;

    // Check if index has newer sessions
    let index_path = gen_dir.join("session-index.json");
    let stale = if index_path.exists() {
        let index_content = std::fs::read_to_string(&index_path)?;
        let index: SessionIndex = serde_json::from_str(&index_content)?;
        match (index.sessions.first(), &meta.latest_session_id) {
            (Some(latest), Some(brief_latest)) => latest.id != *brief_latest,
            (Some(_), None) => true,
            _ => false,
        }
    } else {
        false
    };

    let resolved_repo = resolve_repo(config, project_key, repo);
    // Skip repo_matches check if repo is "unknown"
    let repo_matches = resolved_repo == "unknown" || meta.repo == resolved_repo;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "exists": true,
                "stale": stale,
                "repo_matches": repo_matches,
                "generated_at": meta.generated_at,
                "session_count": meta.session_count,
                "latest_session_id": meta.latest_session_id,
            })
        );
    } else {
        println!(
            "{} {}",
            style::label("Brief exists:"),
            style::dim(&gen_dir.join(BRIEF_FILENAME).display().to_string()),
        );
        println!(
            "{} {}",
            style::label("Generated:"),
            style::dim(&meta.generated_at)
        );
        println!("{} {}", style::label("Sessions:"), meta.session_count);
        let stale_str = if stale {
            style::warn("yes")
        } else {
            style::success("no")
        };
        println!("{} {}", style::label("Stale:"), stale_str);
        if !repo_matches {
            println!(
                "{} brief was generated for '{}', not '{}'",
                style::warn_bold("Warning:"),
                meta.repo,
                resolved_repo,
            );
        }
    }

    Ok(())
}

/// Invalidate the current brief by deleting the meta file.
pub fn invalidate(config: &Config, project_key: &str) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);

    let meta_path = gen_dir.join(BRIEF_META_FILENAME);
    let brief_path = gen_dir.join(BRIEF_FILENAME);

    let mut removed = false;
    if meta_path.exists() {
        std::fs::remove_file(&meta_path)?;
        removed = true;
    }
    if brief_path.exists() {
        std::fs::remove_file(&brief_path)?;
        removed = true;
    }

    if removed {
        println!(
            "{} Brief invalidated for project '{}'.",
            style::icon_ok(),
            style::heading(project_key),
        );
    } else {
        println!("{} No brief to invalidate.", style::dim("—"));
    }

    Ok(())
}

/// Show the current brief content (or --json for meta only).
pub fn show(config: &Config, project_key: &str, json: bool) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);
    let brief_path = gen_dir.join(BRIEF_FILENAME);

    if !brief_path.exists() {
        anyhow::bail!("No brief found. Run `draupnir brief generate`.");
    }

    if json {
        let meta_path = gen_dir.join(BRIEF_META_FILENAME);
        if meta_path.exists() {
            let meta_content = std::fs::read_to_string(&meta_path)?;
            print!("{}", meta_content);
        } else {
            anyhow::bail!(
                "Brief exists but meta is missing. Regenerate with `draupnir brief generate`."
            );
        }
    } else {
        let content = vault::read_text(&brief_path)?;
        print!("{}", content);
    }

    Ok(())
}

/// Extract a markdown section by heading name.
fn extract_section(content: &str, section_name: &str) -> Option<String> {
    let header = format!("## {}", section_name);
    let start = content.find(&header)?;
    let after_header = start + header.len();
    let rest = &content[after_header..];

    // Find the next ## heading or end of content
    let end = rest.find("\n## ").unwrap_or(rest.len());

    let section = rest[..end].trim();
    if section.is_empty() {
        None
    } else {
        Some(section.to_string())
    }
}
