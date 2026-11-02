use crate::commands::{brief, index};
use crate::models::brief::BriefMeta;
use crate::models::config::Config;
use crate::models::instinct::parse_instincts;
use crate::models::session::SessionIndex;
use crate::style;
use crate::vault;
use anyhow::{Context, Result};
use serde::Serialize;
use std::time::SystemTime;

const INDEX_STALE_DAYS: u64 = 7;

/// Severity level for a single health check result.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Ok,
    Warn,
    Error,
}

/// What action was taken (if --fix was used).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    /// No action needed
    None,
    /// Automatically fixed
    Fixed,
    /// Requires human attention
    Manual,
}

/// A single health check result.
#[derive(Debug, Clone, Serialize)]
struct Check {
    name: String,
    status: Status,
    message: String,
    #[serde(skip_serializing_if = "is_action_none")]
    action: Action,
}

fn is_action_none(action: &Action) -> bool {
    matches!(action, Action::None)
}

impl Check {
    fn ok(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: Status::Ok,
            message: message.into(),
            action: Action::None,
        }
    }

    fn warn(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: Status::Warn,
            message: message.into(),
            action: Action::None,
        }
    }

    fn error(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: Status::Error,
            message: message.into(),
            action: Action::None,
        }
    }

    fn with_action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }
}

/// Run vault health checks for a project, optionally auto-fixing issues.
pub fn run(config: &Config, project_key: &str, fix: bool, json: bool) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let project_config = config.project(project_key).context("Unknown project key")?;

    let gen_dir = vault::generated_dir(&vault_root, &project_config.path);
    let index_path = gen_dir.join("session-index.json");
    let brief_meta_path = gen_dir.join("current-brief-meta.json");
    let instincts_path = vault::instincts_file(&vault_root, &project_config.path);
    let inbox_path = vault::inbox_dir(&vault_root);

    let mut checks: Vec<Check> = Vec::new();

    // ── Session Index ────────────────────────────────────────────────────────
    if !index_path.exists() {
        let check = if fix {
            match index::rebuild_core(config, project_key) {
                Ok((_, count)) => Check::ok(
                    "session-index",
                    format!("Rebuilt — {} sessions indexed", count),
                )
                .with_action(Action::Fixed),
                Err(e) => Check::error("session-index", format!("Auto-rebuild failed: {e}"))
                    .with_action(Action::Manual),
            }
        } else {
            Check::error(
                "session-index",
                format!(
                    "Missing — expected at {}. Run `draupnir index rebuild`.",
                    index_path.display()
                ),
            )
        };
        checks.push(check);
    } else {
        let content = std::fs::read_to_string(&index_path)
            .with_context(|| format!("Reading {}", index_path.display()))?;

        match serde_json::from_str::<SessionIndex>(&content) {
            Err(e) => checks.push(Check::error("session-index", format!("Parse error: {e}"))),
            Ok(idx) => {
                let stale_days = file_age_days(&index_path).unwrap_or(0);
                if stale_days >= INDEX_STALE_DAYS {
                    let check = if fix {
                        match index::rebuild_core(config, project_key) {
                            Ok((_, count)) => Check::ok(
                                "session-index",
                                format!(
                                    "Rebuilt — {} sessions (was {} days stale)",
                                    count, stale_days
                                ),
                            )
                            .with_action(Action::Fixed),
                            Err(e) => {
                                Check::warn("session-index", format!("Auto-rebuild failed: {e}"))
                                    .with_action(Action::Manual)
                            }
                        }
                    } else {
                        Check::warn(
                            "session-index",
                            format!(
                                "{} sessions — last updated {} day(s) ago (threshold: {}d). \
                                 Consider rebuilding.",
                                idx.count, stale_days, INDEX_STALE_DAYS
                            ),
                        )
                    };
                    checks.push(check);
                } else {
                    checks.push(Check::ok(
                        "session-index",
                        format!("{} sessions — updated {} day(s) ago", idx.count, stale_days),
                    ));
                }
            }
        }
    }

    // ── Brief ────────────────────────────────────────────────────────────────
    if !brief_meta_path.exists() {
        let check = if fix {
            match brief::generate_core(config, project_key, None) {
                Ok(_) => Check::ok("brief", "Generated fresh brief").with_action(Action::Fixed),
                Err(e) => Check::warn("brief", format!("Auto-generate failed: {e}"))
                    .with_action(Action::Manual),
            }
        } else {
            Check::warn(
                "brief",
                "No brief found. Run `draupnir brief generate`.".to_string(),
            )
        };
        checks.push(check);
    } else {
        let meta_content = std::fs::read_to_string(&brief_meta_path)
            .with_context(|| format!("Reading {}", brief_meta_path.display()))?;

        match serde_json::from_str::<BriefMeta>(&meta_content) {
            Err(e) => checks.push(Check::error("brief", format!("Meta parse error: {e}"))),
            Ok(meta) => {
                // Session-based staleness: compare latest_session_id against index head
                let stale = if index_path.exists() {
                    let idx_content = std::fs::read_to_string(&index_path)
                        .with_context(|| format!("Reading {}", index_path.display()))?;
                    match serde_json::from_str::<SessionIndex>(&idx_content) {
                        Ok(idx) => match (idx.sessions.first(), &meta.latest_session_id) {
                            (Some(latest), Some(brief_latest)) => latest.id != *brief_latest,
                            (Some(_), None) => true,
                            _ => false,
                        },
                        Err(_) => false,
                    }
                } else {
                    false
                };

                if stale {
                    let check = if fix {
                        match brief::generate_core(config, project_key, None) {
                            Ok(_) => Check::ok("brief", "Regenerated — was stale")
                                .with_action(Action::Fixed),
                            Err(e) => Check::warn("brief", format!("Auto-regenerate failed: {e}"))
                                .with_action(Action::Manual),
                        }
                    } else {
                        Check::warn(
                            "brief",
                            format!(
                                "Stale — newer sessions exist since brief was generated ({}). \
                                 Regenerate with `draupnir brief generate`.",
                                meta.generated_at
                            ),
                        )
                    };
                    checks.push(check);
                } else {
                    checks.push(Check::ok(
                        "brief",
                        format!("{} sessions included — up to date", meta.session_count),
                    ));
                }
            }
        }
    }

    // ── Instincts ────────────────────────────────────────────────────────────
    let instinct_files = vault::instincts_files(&vault_root, &project_config.path);
    if instinct_files.is_empty() {
        checks.push(Check::warn(
            "instincts",
            format!("No instincts file at {}.", instincts_path.display()),
        ));
    } else {
        let mut instincts = Vec::new();
        for path in &instinct_files {
            let content =
                vault::read_text(path).with_context(|| format!("Reading {}", path.display()))?;
            instincts.extend(parse_instincts(&content));
        }
        let needs_promotion: Vec<_> = instincts
            .iter()
            .filter(|i| i.confidence == "low" && i.confirmations >= 3)
            .collect();

        if needs_promotion.is_empty() {
            checks.push(Check::ok(
                "instincts",
                format!("{} instincts — none awaiting promotion", instincts.len()),
            ));
        } else {
            // Promotable instincts are advice, not auto-fixable
            checks.push(
                Check::warn(
                    "instincts",
                    format!(
                        "{} instincts — {} low-confidence with 3+ confirmations need promotion: {}",
                        instincts.len(),
                        needs_promotion.len(),
                        needs_promotion
                            .iter()
                            .map(|i| format!("[{}] {}", i.category, i.title))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
                .with_action(if fix { Action::Manual } else { Action::None }),
            );
        }
    }

    // ── Inbox ────────────────────────────────────────────────────────────────
    if !inbox_path.exists() {
        checks.push(Check::warn(
            "inbox",
            format!("Inbox directory missing at {}.", inbox_path.display()),
        ));
    } else {
        let count = std::fs::read_dir(&inbox_path)
            .with_context(|| format!("Reading {}", inbox_path.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
            .count();

        if count > 10 {
            checks.push(
                Check::warn(
                    "inbox",
                    format!("{} items in +Inbox/ — consider processing.", count),
                )
                .with_action(if fix { Action::Manual } else { Action::None }),
            );
        } else {
            checks.push(Check::ok("inbox", format!("{} item(s) in +Inbox/", count)));
        }
    }

    // ── Output ───────────────────────────────────────────────────────────────
    if json {
        println!("{}", serde_json::to_string_pretty(&checks)?);
    } else {
        for check in &checks {
            let icon = match check.status {
                Status::Ok => style::icon_ok(),
                Status::Warn => style::icon_warn(),
                Status::Error => style::icon_error(),
            };
            let action_suffix = match check.action {
                Action::Fixed => format!(" {}", style::success("[FIXED]")),
                Action::Manual => format!(" {}", style::warn("[MANUAL]")),
                Action::None => String::new(),
            };
            let name = style::label(&check.name);
            println!("[{}] {}: {}{}", icon, name, check.message, action_suffix);
        }

        let errors = checks
            .iter()
            .filter(|c| matches!(c.status, Status::Error))
            .count();
        let warns = checks
            .iter()
            .filter(|c| matches!(c.status, Status::Warn))
            .count();
        let fixed = checks
            .iter()
            .filter(|c| matches!(c.action, Action::Fixed))
            .count();

        println!();
        if fixed > 0 {
            println!(
                "{} fixed, {} error(s), {} warning(s) remaining.",
                style::success(&fixed.to_string()),
                style::error(&errors.to_string()),
                style::warn(&warns.to_string()),
            );
        } else if errors > 0 {
            println!(
                "{} error(s), {} warning(s) — {}",
                style::error(&errors.to_string()),
                style::warn(&warns.to_string()),
                style::error_bold("vault needs attention."),
            );
        } else if warns > 0 {
            println!(
                "0 errors, {} warning(s) — vault is mostly healthy.",
                style::warn(&warns.to_string()),
            );
        } else {
            println!(
                "{}",
                style::success_bold("All checks passed — vault is healthy.")
            );
        }
    }

    Ok(())
}

/// Return how many whole days old a file is based on its mtime.
fn file_age_days(path: &std::path::Path) -> Option<u64> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let elapsed = SystemTime::now().duration_since(modified).ok()?;
    Some(elapsed.as_secs() / 86_400)
}
