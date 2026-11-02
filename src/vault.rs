use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// The user's home directory. `$HOME` wins when set: `dirs` already reads it on Unix, but
/// on Windows it asks the OS profile API and ignores `HOME`, so tests and sandboxes that
/// redirect `HOME` would silently hit the real config.
pub fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .context("Could not determine home directory")
}

/// Expand ~ at the start of a path to the user's home directory.
pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    if let Some(rest) = path.strip_prefix("~/").or_else(|| path.strip_prefix("~\\")) {
        Ok(home_dir()?.join(rest))
    } else if path == "~" {
        home_dir()
    } else {
        Ok(PathBuf::from(path))
    }
}

/// Read a vault text file with line endings normalized to `\n`. Files that went through
/// git with `autocrlf` or a Windows editor arrive as CRLF, and the frontmatter and section
/// parsers match on `\n---`.
pub fn read_text(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path).map(|s| normalize_newlines(&s))
}

/// Turn CRLF into LF; used for vault files and for text piped in on stdin.
pub fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Resolve a project-relative path within the vault.
/// E.g., "Areas/Work/Projects/Acme" → ~/Documents/Draupnir/Areas/Work/Projects/Acme
#[allow(dead_code)]
pub fn resolve_vault_path(vault_root: &Path, relative: &str) -> PathBuf {
    vault_root.join(relative)
}

/// Get the session logs directory for a vault.
pub fn session_logs_dir(vault_root: &Path, session_logs_rel: &str) -> PathBuf {
    vault_root.join(session_logs_rel)
}

/// Get the _generated directory for a project.
pub fn generated_dir(vault_root: &Path, project_path: &str) -> PathBuf {
    vault_root.join(project_path).join("_generated")
}

/// Get the inbox directory.
pub fn inbox_dir(vault_root: &Path) -> PathBuf {
    vault_root.join("+Inbox")
}

/// Get the instincts file for a project.
pub fn instincts_file(vault_root: &Path, project_path: &str) -> PathBuf {
    vault_root
        .join(project_path)
        .join("Learnings")
        .join("instincts.md")
}

/// Get all instinct registry files for a project: `instincts.md` plus any
/// split category files (`instincts-*.md`) in the Learnings directory.
/// Returns an empty vec when no registry exists at all.
pub fn instincts_files(vault_root: &Path, project_path: &str) -> Vec<PathBuf> {
    let learnings = vault_root.join(project_path).join("Learnings");
    let mut files = Vec::new();
    let main = learnings.join("instincts.md");
    if main.exists() {
        files.push(main);
    }
    if let Ok(entries) = std::fs::read_dir(&learnings) {
        let mut split: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("instincts-") && n.ends_with(".md"))
            })
            .collect();
        split.sort();
        files.extend(split);
    }
    files
}

/// Get the product-scope.md file for a project.
pub fn product_scope_file(vault_root: &Path, project_path: &str) -> PathBuf {
    vault_root.join(project_path).join("product-scope.md")
}

/// Current UTC timestamp in ISO 8601 format.
pub fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Current local date as NaiveDate.
pub fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

/// Current local time as HHMM string.
pub fn now_hhmm() -> String {
    chrono::Local::now().format("%H%M").to_string()
}
