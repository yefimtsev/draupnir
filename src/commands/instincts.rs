use crate::models::config::Config;
use crate::models::instinct::{compute_stats, confirm_entry, parse_instincts};
use crate::style;
use crate::vault;
use anyhow::{Context, Result};
use std::io::Read;

/// Add a new instinct entry to the instinct registry.
/// Appends to `instincts-<category>.md` when a split category file exists,
/// otherwise to `instincts.md`. Content comes from stdin.
pub fn add(config: &Config, project_key: &str, category: &str, title: &str) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let main_path = vault::instincts_file(&vault_root, &project_config.path);
    let learnings = main_path
        .parent()
        .context("Instincts path has no parent directory")?
        .to_path_buf();
    let category_path = learnings.join(format!("instincts-{}.md", category.to_lowercase()));
    let instincts_path = if category_path.exists() {
        category_path
    } else {
        main_path
    };

    // Read description from stdin
    let mut description = String::new();
    std::io::stdin()
        .read_to_string(&mut description)
        .context("Reading instinct content from stdin")?;
    let description = vault::normalize_newlines(&description);

    if description.trim().is_empty() {
        anyhow::bail!("No content received on stdin. Pipe the instinct description.");
    }

    // Ensure the file exists
    if !instincts_path.exists() {
        std::fs::create_dir_all(learnings)?;
        std::fs::write(
            &instincts_path,
            format!(
                "---\nproject: {}\ntype: instinct-registry\n---\n\n# {} Instincts\n\n\
                 Reusable patterns discovered through session work.\n\
                 Confidence: low (1 observation), medium (2), high (3+).\n\n---\n\n\
                 <!-- Categories: Workflow, Architecture, Debugging, Testing, Git, CI, Performance -->\n\n",
                project_key, project_config.title
            ),
        )?;
    }

    let today = vault::today().format("%Y-%m-%d").to_string();

    // Build the instinct entry
    let entry = format!(
        "\n### [{}] {}\n\
         - **First seen**: {} (session: cli-added)\n\
         - **Confirmations**: 1\n\
         - **Confidence**: low\n\
         - **Context**: {}\n\
         - **Pattern**: {}\n\
         - **Evidence**: Added via CLI\n",
        category,
        title,
        today,
        description.lines().next().unwrap_or("").trim(),
        description.trim()
    );

    // Append to the file
    let mut existing = vault::read_text(&instincts_path)?;
    if !existing.ends_with('\n') {
        existing.push('\n');
    }
    existing.push_str(&entry);
    std::fs::write(&instincts_path, existing)?;

    println!(
        "{} Added instinct: [{}] {}",
        style::icon_ok(),
        style::tag(category),
        style::heading(title),
    );
    Ok(())
}

/// Add a confirmation to an existing instinct, matched by title across all registry files.
pub fn confirm(config: &Config, project_key: &str, title: &str) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let today = vault::today().format("%Y-%m-%d").to_string();

    let mut hits = Vec::new();
    for path in vault::instincts_files(&vault_root, &project_config.path) {
        let content =
            vault::read_text(&path).with_context(|| format!("Reading {}", path.display()))?;
        if let Some(hit) = confirm_entry(&content, title, &today) {
            hits.push((path, hit));
        }
    }
    let (path, (text, count)) = match hits.len() {
        0 => anyhow::bail!(
            "No instinct titled '{title}' in project '{project_key}'. \
             Titles must match exactly (case-insensitive); see `draupnir instincts list`."
        ),
        1 => hits.remove(0),
        _ => anyhow::bail!(
            "'{title}' matches entries in several files: {}",
            hits.iter()
                .map(|(p, _)| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    std::fs::write(&path, text).with_context(|| format!("Writing {}", path.display()))?;
    println!(
        "{} Confirmed: {} ({count} confirmations)",
        style::icon_ok(),
        style::heading(title)
    );
    Ok(())
}

/// List instinct entries with optional filters.
pub fn list(
    config: &Config,
    project_key: &str,
    confidence: Option<&str>,
    category: Option<&str>,
    json: bool,
) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let files = vault::instincts_files(&vault_root, &project_config.path);

    if files.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("No instincts file found for project '{}'.", project_key);
        }
        return Ok(());
    }

    let mut instincts = Vec::new();
    for path in &files {
        let content =
            vault::read_text(path).with_context(|| format!("Reading {}", path.display()))?;
        instincts.extend(parse_instincts(&content));
    }

    let filtered: Vec<_> = instincts
        .into_iter()
        .filter(|i| {
            if let Some(c) = confidence
                && !i.confidence.eq_ignore_ascii_case(c)
            {
                return false;
            }
            if let Some(cat) = category
                && !i.category.eq_ignore_ascii_case(cat)
            {
                return false;
            }
            true
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
    } else {
        if filtered.is_empty() {
            println!("No matching instincts.");
        }
        for inst in &filtered {
            let confidence_colored = match inst.confidence.as_str() {
                "high" => style::success(&inst.confidence),
                "medium" => style::warn(&inst.confidence),
                _ => style::dim(&inst.confidence),
            };
            println!(
                "[{}] {} — {} ({} confirmations)",
                style::tag(&inst.category),
                style::heading(&inst.title),
                confidence_colored,
                inst.confirmations,
            );
        }
    }

    Ok(())
}

/// Print instinct stats for a project.
pub fn stats(config: &Config, project_key: &str, json: bool) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let files = vault::instincts_files(&vault_root, &project_config.path);

    if files.is_empty() {
        println!("No instincts file found for project '{}'.", project_key);
        return Ok(());
    }

    let mut instincts = Vec::new();
    for path in &files {
        let content =
            vault::read_text(path).with_context(|| format!("Reading {}", path.display()))?;
        instincts.extend(parse_instincts(&content));
    }
    let stats = compute_stats(&instincts);

    if json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!(
            "{} {}",
            style::label("Total instincts:"),
            style::heading(&stats.total.to_string()),
        );
        println!(
            "{} {} high, {} medium, {} low",
            style::label("Confidence:"),
            style::success(&stats.by_confidence.high.to_string()),
            style::warn(&stats.by_confidence.medium.to_string()),
            style::dim(&stats.by_confidence.low.to_string()),
        );
        println!("\n{}:", style::label("By category"));
        for (cat, count) in &stats.by_category {
            println!("  {}: {}", style::tag(cat), count);
        }
    }

    Ok(())
}
