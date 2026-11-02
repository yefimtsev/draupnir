use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Top-level config loaded from ~/.config/draupnir/config.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub vault_root: String,
    #[serde(default = "default_session_logs_dir")]
    pub session_logs_dir: String,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    pub path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repos: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detect_patterns: Vec<String>,
}

/// A path as a comparable string: `/` separators everywhere, so `Acme/rocket` in the config
/// matches `C:\dev\Acme\rocket`; lowercased on Windows, where paths are case-insensitive and
/// shells hand out `c:\` and `C:\` for the same directory.
fn norm_path(p: &str) -> String {
    let p = p.replace('\\', "/");
    if cfg!(windows) { p.to_lowercase() } else { p }
}

fn default_session_logs_dir() -> String {
    "Areas/Work/Session-Logs".to_string()
}

impl Config {
    /// Detect project from a directory. The most specific match wins: a detect pattern
    /// contained in `cwd`, or a repo path `cwd` is inside (worktrees included), scored by
    /// the length of what matched. Two projects tied for the top score is an error —
    /// `projects` is a HashMap, so "first match" used to be a per-process coin flip.
    pub fn detect_project(&self, cwd: &Path) -> anyhow::Result<Option<String>> {
        let cwd_str = cwd.to_string_lossy();
        let cwd_norm = norm_path(&cwd_str);
        let mut scored: Vec<(usize, &String)> = Vec::new();
        for (key, project) in &self.projects {
            let mut score = project
                .detect_patterns
                .iter()
                .map(|p| norm_path(p))
                .filter(|p| !p.is_empty() && cwd_norm.contains(p.as_str()))
                .map(|p| p.len())
                .max()
                .unwrap_or(0);
            for repo in &project.repos {
                let repo = norm_path(&crate::vault::expand_tilde(repo)?.to_string_lossy());
                let repo = repo.trim_end_matches('/');
                // Component-wise prefix: `/dev/app` must not claim `/dev/app-other`.
                let inside = cwd_norm
                    .strip_prefix(repo)
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'));
                if !repo.is_empty() && inside {
                    score = score.max(repo.len());
                }
            }
            if score > 0 {
                scored.push((score, key));
            }
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
        match scored.as_slice() {
            [] => Ok(None),
            [(top, a), (next, b), ..] if top == next => anyhow::bail!(
                "Ambiguous project for '{}': '{}' and '{}' match equally. Use --project.",
                cwd_str,
                a,
                b
            ),
            [(_, key), ..] => Ok(Some((*key).clone())),
        }
    }

    /// Get a project config by key, or None.
    pub fn project(&self, key: &str) -> Option<&ProjectConfig> {
        self.projects.get(key)
    }

    /// Resolve the vault root path, expanding ~ to home dir.
    pub fn resolved_vault_root(&self) -> anyhow::Result<std::path::PathBuf> {
        crate::vault::expand_tilde(&self.vault_root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(detect: &[&str], repos: &[&str]) -> ProjectConfig {
        ProjectConfig {
            title: String::new(),
            prefix: None,
            path: String::new(),
            repos: repos.iter().map(|s| s.to_string()).collect(),
            detect_patterns: detect.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn config(projects: Vec<(&str, ProjectConfig)>) -> Config {
        Config {
            vault_root: String::new(),
            session_logs_dir: default_session_logs_dir(),
            projects: projects
                .into_iter()
                .map(|(k, p)| (k.to_string(), p))
                .collect(),
        }
    }

    fn acme_rocket() -> Config {
        config(vec![
            ("acme", project(&["Acme"], &["/dev/Acme/to-be-named"])),
            (
                "rocket",
                project(&["Acme/rocket", "rocket"], &["/dev/Acme/rocket"]),
            ),
        ])
    }

    fn detect(c: &Config, cwd: &str) -> Option<String> {
        c.detect_project(Path::new(cwd)).unwrap()
    }

    #[test]
    fn most_specific_match_wins_every_time() {
        // Each new HashMap gets fresh hash keys (a clone would not), so a first-match
        // bug shows up within a few iterations.
        for _ in 0..50 {
            let c = acme_rocket();
            assert_eq!(detect(&c, "/dev/Acme/rocket").as_deref(), Some("rocket"));
        }
        let c = acme_rocket();
        assert_eq!(
            detect(&c, "/dev/Acme/rocket/.claude/worktrees/f1/src").as_deref(),
            Some("rocket")
        );
        assert_eq!(detect(&c, "/dev/Acme/to-be-named").as_deref(), Some("acme"));
        assert_eq!(detect(&c, "/dev/Acme").as_deref(), Some("acme"));
        assert_eq!(detect(&c, "/elsewhere"), None);
    }

    #[test]
    fn windows_separators_match_slash_patterns() {
        let c = acme_rocket();
        assert_eq!(
            detect(&c, r"C:\dev\Acme\rocket\src").as_deref(),
            Some("rocket")
        );
        let c = config(vec![("a", project(&[], &[r"C:\dev\app\"]))]);
        assert_eq!(detect(&c, "C:/dev/app/x").as_deref(), Some("a"));
    }

    #[test]
    fn repo_prefix_is_component_wise() {
        let c = config(vec![("a", project(&[], &["/dev/app"]))]);
        assert_eq!(detect(&c, "/dev/app-other"), None);
        assert_eq!(detect(&c, "/dev/app/x").as_deref(), Some("a"));
    }

    #[test]
    fn equal_specificity_is_an_error_naming_both() {
        let c = config(vec![
            ("alpha", project(&["shared"], &[])),
            ("beta", project(&["shared"], &[])),
        ]);
        let err = c
            .detect_project(Path::new("/x/shared"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("alpha") && err.contains("beta"), "{err}");
    }
}
