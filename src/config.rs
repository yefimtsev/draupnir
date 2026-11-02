use crate::models::config::Config;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Find the config file. Checks ~/.config/draupnir/ first (XDG convention for CLI tools),
/// then falls back to the platform config dir (~/Library/Application Support/ on macOS,
/// %APPDATA% on Windows).
pub fn config_path() -> Result<PathBuf> {
    let home = crate::vault::home_dir()?;
    let xdg_path = home.join(".config").join("draupnir").join("config.yaml");
    if xdg_path.exists() {
        return Ok(xdg_path);
    }

    if let Some(platform_dir) = dirs::config_dir() {
        let platform_path = platform_dir.join("draupnir").join("config.yaml");
        if platform_path.exists() {
            return Ok(platform_path);
        }
    }

    // Default to XDG path for error messages
    Ok(xdg_path)
}

/// Load config from the default location, or return a helpful error.
pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        anyhow::bail!(
            "Config file not found at {}\n\
             Create it with one line:\n\n  \
             vault_root: ~/Documents/Draupnir\n\n\
             then register projects with `draupnir project add`.\n\
             Format: https://github.com/yefimtsev/draupnir#configuration",
            path.display()
        );
    }
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("Reading {}", path.display()))?;
    let config: Config =
        serde_yaml::from_str(&content).with_context(|| format!("Parsing {}", path.display()))?;
    Ok(config)
}

/// Resolve the project key: explicit flag > detect from cwd > error.
pub fn resolve_project(config: &Config, explicit: Option<&str>) -> Result<String> {
    if let Some(key) = explicit {
        if config.project(key).is_some() {
            return Ok(key.to_string());
        }
        anyhow::bail!(
            "Unknown project '{}'. Known projects: {}",
            key,
            config
                .projects
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let cwd = std::env::current_dir().context("Could not determine current directory")?;

    config.detect_project(&cwd)?.with_context(|| {
        format!(
            "Could not detect project from cwd '{}'. Use --project flag.\nKnown projects: {}",
            cwd.display(),
            config
                .projects
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}
