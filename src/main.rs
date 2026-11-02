mod commands;
mod config;
mod models;
mod style;
mod vault;

use anyhow::Result;
use clap::{Args, CommandFactory, Parser, Subcommand};
use std::path::PathBuf;
use std::process;

// ---------------------------------------------------------------------------
// Top-level CLI
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    name = "draupnir",
    version,
    about = "Draupnir vault CLI — sessions, briefs, instincts, health"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

// ---------------------------------------------------------------------------
// Shared --project flag (optional everywhere)
// ---------------------------------------------------------------------------

#[derive(Debug, Args)]
struct ProjectFlag {
    /// Project key (detected from cwd if omitted)
    #[arg(long, short = 'p')]
    project: Option<String>,
}

// ---------------------------------------------------------------------------
// Top-level subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
enum Command {
    /// Write a session log from stdin
    Session {
        #[command(subcommand)]
        sub: SessionCommand,
    },

    /// Manage the session index
    Index {
        #[command(subcommand)]
        sub: IndexCommand,
    },

    /// Manage the project brief
    Brief {
        #[command(subcommand)]
        sub: BriefCommand,
    },

    /// Manage learned instinct patterns
    Instincts {
        #[command(subcommand)]
        sub: InstinctsCommand,
    },

    /// Manage the product decision log
    Decision {
        #[command(subcommand)]
        sub: DecisionCommand,
    },

    /// Run vault and project health checks
    Doctor {
        #[command(flatten)]
        project: ProjectFlag,

        /// Auto-fix what can be fixed (rebuild index, regenerate brief)
        #[arg(long)]
        fix: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Manage project configuration
    Project {
        #[command(subcommand)]
        sub: ProjectCommand,
    },

    /// Quick-capture a note to the vault inbox
    Capture {
        #[command(flatten)]
        project: ProjectFlag,

        /// Message words (joined with spaces)
        #[arg(required = true, trailing_var_arg = true)]
        message: Vec<String>,
    },

    /// Inspect the vault itself
    Vault {
        #[command(subcommand)]
        sub: VaultCommand,
    },

    /// Install the Claude Code skills bundled with this binary
    Skills {
        #[command(subcommand)]
        sub: SkillsCommand,
    },
}

#[derive(Debug, Subcommand)]
enum VaultCommand {
    /// Print the resolved vault root path
    Root,
}

#[derive(Debug, Subcommand)]
enum SkillsCommand {
    /// Write bundled skills to the Claude skills directory
    Install {
        /// Target directory (default: $CLAUDE_CONFIG_DIR/skills or ~/.claude/skills)
        #[arg(long)]
        dir: Option<PathBuf>,

        /// Overwrite skills that were modified locally (symlinks are never touched)
        #[arg(long)]
        force: bool,
    },
}

// ---------------------------------------------------------------------------
// session subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
enum SessionCommand {
    /// Write a session log (content read from stdin)
    Write {
        #[command(flatten)]
        project: ProjectFlag,

        /// Session topic / title
        #[arg(long)]
        topic: String,

        /// Tags to attach (repeatable)
        #[arg(long = "tag")]
        tags: Vec<String>,
    },

    /// Search session index with filters
    Search {
        #[command(flatten)]
        project: ProjectFlag,

        /// Substring match on title (case-insensitive)
        #[arg(long)]
        query: Option<String>,

        /// Exact tag match (case-insensitive)
        #[arg(long)]
        tag: Option<String>,

        /// Exact ticket ID match (case-insensitive)
        #[arg(long)]
        ticket: Option<String>,

        /// Only sessions on or after this date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Maximum results to return
        #[arg(long, default_value = "20")]
        limit: usize,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

// ---------------------------------------------------------------------------
// index subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
enum IndexCommand {
    /// Rebuild the full session index from vault files
    Rebuild {
        #[command(flatten)]
        project: ProjectFlag,
    },

    /// Append a single session file to the existing index
    Append {
        #[command(flatten)]
        project: ProjectFlag,

        /// Path to the session log file
        #[arg(long)]
        file: PathBuf,
    },

    /// Print index statistics
    Stats {
        #[command(flatten)]
        project: ProjectFlag,
    },
}

// ---------------------------------------------------------------------------
// brief subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
enum BriefCommand {
    /// Generate a fresh brief from recent sessions
    Generate {
        #[command(flatten)]
        project: ProjectFlag,

        /// Path to the project repo (optional — falls back to config repos[0])
        #[arg(long)]
        repo: Option<PathBuf>,
    },

    /// Show brief staleness and metadata
    Status {
        #[command(flatten)]
        project: ProjectFlag,

        /// Path to the project repo (optional — falls back to config repos[0])
        #[arg(long)]
        repo: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Invalidate (delete) the current brief
    Invalidate {
        #[command(flatten)]
        project: ProjectFlag,
    },

    /// Print the current brief (or its metadata with --json)
    Show {
        #[command(flatten)]
        project: ProjectFlag,

        /// Output meta JSON instead of markdown content
        #[arg(long)]
        json: bool,
    },
}

// ---------------------------------------------------------------------------
// instincts subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
enum InstinctsCommand {
    /// Add an instinct entry (description read from stdin)
    Add {
        #[command(flatten)]
        project: ProjectFlag,

        /// Category (e.g. Git, Testing, Architecture)
        #[arg(long)]
        category: String,

        /// Short title for the instinct
        #[arg(long)]
        title: String,
    },

    /// Record one more confirmation on an existing instinct (raises its confidence)
    Confirm {
        #[command(flatten)]
        project: ProjectFlag,

        /// Exact title of the instinct (case-insensitive)
        #[arg(long)]
        title: String,
    },

    /// List instinct entries with optional filters
    List {
        #[command(flatten)]
        project: ProjectFlag,

        /// Filter by confidence level (high, medium, low)
        #[arg(long)]
        confidence: Option<String>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Print instinct statistics
    Stats {
        #[command(flatten)]
        project: ProjectFlag,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

// ---------------------------------------------------------------------------
// decision subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
enum DecisionCommand {
    /// Append a decision to the Decision Log
    Add {
        #[command(flatten)]
        project: ProjectFlag,

        /// The decision made
        #[arg(long)]
        decision: String,

        /// Why this decision was made
        #[arg(long)]
        why: String,

        /// Date (YYYY-MM-DD, defaults to today)
        #[arg(long)]
        date: Option<String>,
    },

    /// List decisions with optional filters
    List {
        #[command(flatten)]
        project: ProjectFlag,

        /// Only decisions on or after this date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Output the raw Decision Log section
    Read {
        #[command(flatten)]
        project: ProjectFlag,
    },
}

// ---------------------------------------------------------------------------
// project subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
enum ProjectCommand {
    /// Add a new project to the config
    Add {
        /// Project key (short lowercase identifier, e.g. "freelance")
        key: String,

        /// Human-readable title (e.g. "Freelance")
        #[arg(long)]
        title: String,

        /// Ticket prefix (e.g. "FRL")
        #[arg(long)]
        prefix: Option<String>,

        /// Vault-relative path (defaults to Areas/Work/Projects/<title>)
        #[arg(long)]
        path: Option<String>,

        /// Repository paths (repeatable)
        #[arg(long = "repo")]
        repos: Vec<String>,

        /// CWD detection patterns (repeatable)
        #[arg(long = "detect")]
        detect_patterns: Vec<String>,
    },

    /// List all configured projects
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    if let Err(err) = run() {
        eprintln!("{} {err:#}", style::error_bold("error:"));
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let Some(command) = cli.command else {
        print!("{}", style::banner());
        Cli::command().print_help()?;
        return Ok(());
    };

    // Skills install works before a config exists — it is part of first-time setup.
    if let Command::Skills {
        sub: SkillsCommand::Install { dir, force },
    } = &command
    {
        return commands::skills::install(dir.as_deref(), *force);
    }

    let config = config::load_config()?;

    match command {
        Command::Session { sub } => match sub {
            SessionCommand::Write {
                project,
                topic,
                tags,
            } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::session::write_session(&config, &project_key, &topic, &tags)?;
            }
            SessionCommand::Search {
                project,
                query,
                tag,
                ticket,
                since,
                limit,
                json,
            } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::session::search(
                    &config,
                    &project_key,
                    query.as_deref(),
                    tag.as_deref(),
                    ticket.as_deref(),
                    since.as_deref(),
                    limit,
                    json,
                )?;
            }
        },

        Command::Index { sub } => match sub {
            IndexCommand::Rebuild { project } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::index::rebuild(&config, &project_key)?;
            }
            IndexCommand::Append { project, file } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::index::append(&config, &project_key, &file)?;
            }
            IndexCommand::Stats { project } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::index::stats(&config, &project_key)?;
            }
        },

        Command::Brief { sub } => match sub {
            BriefCommand::Generate { project, repo } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                let repo_str = repo.as_ref().map(|r| r.to_string_lossy().to_string());
                commands::brief::generate(&config, &project_key, repo_str.as_deref())?;
            }
            BriefCommand::Status {
                project,
                repo,
                json,
            } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                let repo_str = repo.as_ref().map(|r| r.to_string_lossy().to_string());
                commands::brief::status(&config, &project_key, repo_str.as_deref(), json)?;
            }
            BriefCommand::Invalidate { project } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::brief::invalidate(&config, &project_key)?;
            }
            BriefCommand::Show { project, json } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::brief::show(&config, &project_key, json)?;
            }
        },

        Command::Instincts { sub } => match sub {
            InstinctsCommand::Add {
                project,
                category,
                title,
            } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::instincts::add(&config, &project_key, &category, &title)?;
            }
            InstinctsCommand::Confirm { project, title } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::instincts::confirm(&config, &project_key, &title)?;
            }
            InstinctsCommand::List {
                project,
                confidence,
                category,
                json,
            } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::instincts::list(
                    &config,
                    &project_key,
                    confidence.as_deref(),
                    category.as_deref(),
                    json,
                )?;
            }
            InstinctsCommand::Stats { project, json } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::instincts::stats(&config, &project_key, json)?;
            }
        },

        Command::Decision { sub } => match sub {
            DecisionCommand::Add {
                project,
                decision,
                why,
                date,
            } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::decision::add(&config, &project_key, &decision, &why, date.as_deref())?;
            }
            DecisionCommand::List {
                project,
                since,
                json,
            } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::decision::list(&config, &project_key, since.as_deref(), json)?;
            }
            DecisionCommand::Read { project } => {
                let project_key = config::resolve_project(&config, project.project.as_deref())?;
                commands::decision::read(&config, &project_key)?;
            }
        },

        Command::Doctor { project, fix, json } => {
            let project_key = config::resolve_project(&config, project.project.as_deref())?;
            commands::doctor::run(&config, &project_key, fix, json)?;
        }

        Command::Project { sub } => match sub {
            ProjectCommand::Add {
                key,
                title,
                prefix,
                path,
                repos,
                detect_patterns,
            } => {
                commands::project::add(
                    &key,
                    &title,
                    prefix.as_deref(),
                    path.as_deref(),
                    &repos,
                    &detect_patterns,
                )?;
            }
            ProjectCommand::List { json } => {
                commands::project::list(&config, json)?;
            }
        },

        Command::Capture { project, message } => {
            let project_key = config::resolve_project(&config, project.project.as_deref())?;
            let joined = message.join(" ");
            commands::capture::run(&config, &project_key, &joined)?;
        }

        Command::Vault {
            sub: VaultCommand::Root,
        } => {
            println!("{}", config.resolved_vault_root()?.display());
        }

        Command::Skills { .. } => unreachable!("handled before config load"),
    }

    Ok(())
}
