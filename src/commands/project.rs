use crate::config;
use crate::models::config::{Config, ProjectConfig};
use crate::style;
use crate::vault;
use anyhow::{Context, Result};

/// Add a new project to the config file and scaffold vault directories.
pub fn add(
    key: &str,
    title: &str,
    prefix: Option<&str>,
    path: Option<&str>,
    repos: &[String],
    detect_patterns: &[String],
) -> Result<()> {
    let config_path = config::config_path()?;
    let mut config = config::load_config()?;

    if config.projects.contains_key(key) {
        anyhow::bail!("Project '{}' already exists in config", key);
    }

    // Default vault path: Areas/Work/Projects/<Title>
    let vault_path = path
        .map(String::from)
        .unwrap_or_else(|| format!("Areas/Work/Projects/{}", title));

    let project = ProjectConfig {
        title: title.to_string(),
        prefix: prefix.map(String::from),
        path: vault_path.clone(),
        repos: repos.to_vec(),
        detect_patterns: detect_patterns.to_vec(),
    };

    config.projects.insert(key.to_string(), project);

    // Write config back
    write_config(&config_path, &config)?;

    // Scaffold vault directories
    let vault_root = config.resolved_vault_root()?;
    scaffold_project_dirs(&vault_root, &vault_path)?;

    eprintln!(
        "{} Added project '{}' ({})",
        style::icon_ok(),
        style::heading(key),
        title
    );
    eprintln!(
        "  {} {}",
        style::label("vault path:"),
        style::dim(&vault_path)
    );
    eprintln!(
        "  {} {}",
        style::label("config:"),
        style::dim(&config_path.display().to_string())
    );

    Ok(())
}

/// List all configured projects.
pub fn list(config: &Config, json: bool) -> Result<()> {
    if json {
        let output = serde_json::to_string_pretty(&config.projects)?;
        println!("{output}");
        return Ok(());
    }

    if config.projects.is_empty() {
        eprintln!("No projects configured.");
        return Ok(());
    }

    let mut keys: Vec<_> = config.projects.keys().collect();
    keys.sort();

    for key in keys {
        let p = &config.projects[key];
        println!("{}", style::heading(key));
        println!("  {}    {}", style::label("title:"), p.title);
        if let Some(ref pfx) = p.prefix {
            println!("  {}   {}", style::label("prefix:"), pfx);
        }
        println!("  {}     {}", style::label("path:"), style::dim(&p.path));
        if !p.repos.is_empty() {
            println!("  {}    {}", style::label("repos:"), p.repos.join(", "));
        }
        if !p.detect_patterns.is_empty() {
            println!(
                "  {}   {}",
                style::label("detect:"),
                p.detect_patterns.join(", ")
            );
        }
        println!();
    }

    Ok(())
}

/// Write the full config back to YAML, preserving structure via serde.
fn write_config(path: &std::path::Path, config: &Config) -> Result<()> {
    let yaml = serde_yaml::to_string(config).context("Serializing config to YAML")?;
    std::fs::write(path, yaml).with_context(|| format!("Writing {}", path.display()))?;
    Ok(())
}

/// Create the standard project directory structure in the vault.
fn scaffold_project_dirs(vault_root: &std::path::Path, project_path: &str) -> Result<()> {
    let base = vault::resolve_vault_path(vault_root, project_path);
    let dirs = [base.join("_generated"), base.join("Learnings")];

    for dir in &dirs {
        if !dir.exists() {
            std::fs::create_dir_all(dir).with_context(|| format!("Creating {}", dir.display()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_list_empty_projects() {
        let config = Config {
            vault_root: "/tmp/vault".to_string(),
            session_logs_dir: "Areas/Work/Session-Logs".to_string(),
            projects: HashMap::new(),
        };
        // Should not error
        list(&config, false).unwrap();
    }

    #[test]
    fn test_list_json_output() {
        let mut projects = HashMap::new();
        projects.insert(
            "test".to_string(),
            ProjectConfig {
                title: "Test Project".to_string(),
                prefix: Some("TST".to_string()),
                path: "Areas/Work/Projects/Test".to_string(),
                repos: vec![],
                detect_patterns: vec!["test-repo".to_string()],
            },
        );
        let config = Config {
            vault_root: "/tmp/vault".to_string(),
            session_logs_dir: "Areas/Work/Session-Logs".to_string(),
            projects,
        };
        list(&config, true).unwrap();
    }
}
