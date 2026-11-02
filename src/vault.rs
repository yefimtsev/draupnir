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
        Ok(native(home_dir()?.join(rest)))
    } else if path == "~" {
        home_dir()
    } else {
        Ok(native(PathBuf::from(path)))
    }
}

/// Rebuild a path from its components so it uses the platform separator throughout.
/// Config values like `Areas/Work/Projects/X` joined onto `C:\Users\...` would otherwise print
/// as `C:\Users\x/Areas/Work`. A no-op on Unix.
fn native(path: PathBuf) -> PathBuf {
    path.components().collect()
}

/// Read a vault text file through `normalize_text`.
pub fn read_text(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path).map(|s| normalize_text(&s))
}

/// Drop a leading UTF-8 BOM and turn CRLF into LF; used for vault files, the config and stdin.
/// Windows hands us both: Notepad and git `autocrlf` write CRLF, PowerShell 5.1 prefixes text
/// piped to a program with a BOM. Either one hides frontmatter from the `^---\n` parsers.
pub fn normalize_text(s: &str) -> String {
    s.strip_prefix('\u{feff}')
        .unwrap_or(s)
        .replace("\r\n", "\n")
}

/// Resolve a project-relative path within the vault.
/// E.g., "Areas/Work/Projects/Acme" → ~/Documents/Draupnir/Areas/Work/Projects/Acme
pub fn resolve_vault_path(vault_root: &Path, relative: &str) -> PathBuf {
    native(vault_root.join(relative))
}

/// Get the session logs directory for a vault.
pub fn session_logs_dir(vault_root: &Path, session_logs_rel: &str) -> PathBuf {
    native(vault_root.join(session_logs_rel))
}

/// Get the _generated directory for a project.
pub fn generated_dir(vault_root: &Path, project_path: &str) -> PathBuf {
    native(vault_root.join(project_path).join("_generated"))
}

/// Get the inbox directory.
pub fn inbox_dir(vault_root: &Path) -> PathBuf {
    vault_root.join("+Inbox")
}

/// Get the instincts file for a project.
pub fn instincts_file(vault_root: &Path, project_path: &str) -> PathBuf {
    native(
        vault_root
            .join(project_path)
            .join("Learnings")
            .join("instincts.md"),
    )
}

/// Get all instinct registry files for a project: `instincts.md` plus any
/// split category files (`instincts-*.md`) in the Learnings directory.
/// Returns an empty vec when no registry exists at all.
pub fn instincts_files(vault_root: &Path, project_path: &str) -> Vec<PathBuf> {
    let learnings = native(vault_root.join(project_path).join("Learnings"));
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
    native(vault_root.join(project_path).join("product-scope.md"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_text_drops_bom_and_crlf() {
        assert_eq!(
            normalize_text("\u{feff}---\r\na: 1\r\n---\r\n"),
            "---\na: 1\n---\n"
        );
        assert_eq!(normalize_text("x\u{feff}y\n"), "x\u{feff}y\n"); // only a leading BOM
    }

    #[cfg(windows)]
    #[test]
    fn joins_use_the_platform_separator() {
        let dir = generated_dir(Path::new(r"C:\v"), "Areas/Work/Projects/P");
        assert_eq!(
            dir.to_string_lossy(),
            r"C:\v\Areas\Work\Projects\P\_generated"
        );
    }
}
