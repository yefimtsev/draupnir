use crate::models::config::Config;
use crate::models::decision::{extract_decision_section, parse_decision_rows};
use crate::style;
use crate::vault;
use anyhow::{Context, Result};

/// Append a decision row to the Decision Log table in product-scope.md.
pub fn add(
    config: &Config,
    project_key: &str,
    decision: &str,
    why: &str,
    date: Option<&str>,
) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let scope_path = vault::product_scope_file(&vault_root, &project_config.path);

    let content = match vault::read_text(&scope_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("Reading {}", scope_path.display())),
    };
    // A fresh project has no scope file yet; `add` is an explicit write, so start one.
    let content = ensure_decision_log(content, &project_config.title);
    if let Some(dir) = scope_path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("Creating {}", dir.display()))?;
    }

    let section_start = content
        .find("## Decision Log")
        .context("No '## Decision Log' section found in product-scope.md")?;

    // Find the end of the Decision Log section (next ## heading or EOF)
    let after_header = &content[section_start..];
    let section_end = after_header[1..]
        .find("\n## ")
        .map(|pos| section_start + pos + 1)
        .unwrap_or(content.len());

    let date_str = date.unwrap_or_else(|| {
        // Leak is fine — this runs once per CLI invocation
        Box::leak(
            vault::today()
                .format("%Y-%m-%d")
                .to_string()
                .into_boxed_str(),
        )
    });

    let new_row = format!("| {} | {} | {} |\n", date_str, decision, why);

    // Insert the new row at the end of the section (before the next heading or at EOF)
    let mut result = String::with_capacity(content.len() + new_row.len());
    let insert_point = find_table_end(&content[section_start..section_end]);
    let absolute_insert = section_start + insert_point;

    result.push_str(&content[..absolute_insert]);
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result.push_str(&new_row);
    result.push_str(&content[absolute_insert..]);

    std::fs::write(&scope_path, result)
        .with_context(|| format!("Writing {}", scope_path.display()))?;

    println!("{}", style::dim(&scope_path.display().to_string()));
    Ok(())
}

/// List decisions for a project.
pub fn list(config: &Config, project_key: &str, since: Option<&str>, json: bool) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let scope_path = vault::product_scope_file(&vault_root, &project_config.path);

    if !scope_path.exists() {
        if json {
            println!("[]");
        } else {
            println!("No product-scope.md found for project '{}'.", project_key);
        }
        return Ok(());
    }

    let content = vault::read_text(&scope_path)
        .with_context(|| format!("Reading {}", scope_path.display()))?;

    let section = match extract_decision_section(&content) {
        Some(s) => s,
        None => {
            if json {
                println!("[]");
            } else {
                println!("No Decision Log section found in product-scope.md.");
            }
            return Ok(());
        }
    };

    let mut decisions = parse_decision_rows(section);

    if let Some(since_date) = since {
        decisions.retain(|d| d.date.as_str() >= since_date);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&decisions)?);
    } else if decisions.is_empty() {
        println!("No decisions found.");
    } else {
        for d in &decisions {
            println!(
                "{} {} — {}",
                style::dim(&d.date),
                style::heading(&d.decision),
                d.why,
            );
        }
    }

    Ok(())
}

/// Output the raw Decision Log section for piping to other tools.
pub fn read(config: &Config, project_key: &str) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project")?;
    let scope_path = vault::product_scope_file(&vault_root, &project_config.path);

    if !scope_path.exists() {
        anyhow::bail!("product-scope.md not found at {}", scope_path.display());
    }

    let content = vault::read_text(&scope_path)
        .with_context(|| format!("Reading {}", scope_path.display()))?;

    let section = extract_decision_section(&content)
        .context("No '## Decision Log' section found in product-scope.md")?;

    print!("{}", section);
    Ok(())
}

/// Find the byte offset (relative to section start) where a new row should be inserted.
/// This is the position after the last non-empty line in the table.
/// Appends an empty Decision Log table when the section is missing; the file itself may be empty.
fn ensure_decision_log(mut content: String, title: &str) -> String {
    if content.contains("## Decision Log") {
        return content;
    }
    if content.trim().is_empty() {
        content = format!("# {title} — Product Scope\n");
    } else if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n");
    content
}

fn find_table_end(section: &str) -> usize {
    let mut last_row_end = 0;
    let mut offset = 0;

    for line in section.lines() {
        let line_with_newline = offset + line.len() + 1; // +1 for \n
        let trimmed = line.trim();

        if trimmed.starts_with('|') {
            // This is a table row — track its end
            last_row_end = line_with_newline.min(section.len());
        } else if !trimmed.is_empty() && last_row_end > 0 {
            // Non-table, non-empty line after table rows — stop
            break;
        }

        offset = line_with_newline;
    }

    if last_row_end > 0 {
        last_row_end
    } else {
        // No table rows found — insert after the section header
        section.find('\n').map(|p| p + 1).unwrap_or(section.len())
    }
}

#[cfg(test)]
mod tests {
    use super::{ensure_decision_log, find_table_end};

    #[test]
    fn decision_log_scaffolded_once() {
        let fresh = ensure_decision_log(String::new(), "Demo");
        assert!(fresh.starts_with("# Demo — Product Scope\n\n## Decision Log\n"));
        assert_eq!(ensure_decision_log(fresh.clone(), "Demo"), fresh);

        let existing = ensure_decision_log("# Scope\nNotes".into(), "Demo");
        assert!(existing.starts_with("# Scope\nNotes\n\n## Decision Log\n\n| Date |"));
    }

    #[test]
    fn find_end_of_table() {
        let section = "## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n| 2026-05-01 | X | Y |\n| 2026-05-02 | A | B |\n";
        let end = find_table_end(section);
        // Should point after the last row
        assert_eq!(
            &section[..end],
            "## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n| 2026-05-01 | X | Y |\n| 2026-05-02 | A | B |\n"
        );
    }

    #[test]
    fn find_end_with_trailing_empty_lines() {
        let section = "## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n| 2026-05-01 | X | Y |\n\n";
        let end = find_table_end(section);
        assert!(section[..end].contains("2026-05-01"));
    }

    #[test]
    fn find_end_no_rows() {
        let section = "## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n";
        let end = find_table_end(section);
        // Should place after the separator row
        assert_eq!(
            &section[..end],
            "## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n"
        );
    }
}
