//! Semantic color helpers for human-readable CLI output.
//!
//! All helpers auto-detect terminal support via `owo-colors` + `supports-colors`.
//! When stdout is not a TTY (piped, redirected), colors are stripped automatically.

use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;
use owo_colors::Style;

// ── Status icons ─────────────────────────────────────────────────────────────

pub fn icon_ok() -> String {
    "✓".if_supports_color(Stdout, |t| t.green()).to_string()
}

pub fn icon_warn() -> String {
    "⚠".if_supports_color(Stdout, |t| t.yellow()).to_string()
}

pub fn icon_error() -> String {
    "✗".if_supports_color(Stdout, |t| t.red()).to_string()
}

// ── Semantic formatters ──────────────────────────────────────────────────────

/// Bold text — headings, project keys, titles.
pub fn heading(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.bold()).to_string()
}

/// Cyan — field labels like "title:", "path:", "Sessions:".
pub fn label(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.cyan()).to_string()
}

/// Dim — paths, dates, secondary metadata.
pub fn dim(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.dimmed()).to_string()
}

/// Green — success messages, positive counts.
pub fn success(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.green()).to_string()
}

/// Yellow — warnings, stale indicators.
pub fn warn(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.yellow()).to_string()
}

/// Red — errors, failure messages.
pub fn error(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.red()).to_string()
}

/// Magenta — tags, categories.
pub fn tag(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.magenta()).to_string()
}

/// Bold + green — completion / summary lines.
pub fn success_bold(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.style(Style::new().green().bold()))
        .to_string()
}

/// Bold + red — error prefix.
pub fn error_bold(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.style(Style::new().red().bold()))
        .to_string()
}

/// Bold + yellow — warning prefix.
pub fn warn_bold(text: &str) -> String {
    text.if_supports_color(Stdout, |t| t.style(Style::new().yellow().bold()))
        .to_string()
}

// ── Banner ───────────────────────────────────────────────────────────────────

/// Big colorized ASCII banner for `draupnir` with no subcommand.
pub fn banner() -> String {
    let version = env!("CARGO_PKG_VERSION");

    const LOGO: [&str; 6] = [
        "██████╗ ██████╗  █████╗ ██╗   ██╗██████╗ ███╗   ██╗██╗██████╗ ",
        "██╔══██╗██╔══██╗██╔══██╗██║   ██║██╔══██╗████╗  ██║██║██╔══██╗",
        "██║  ██║██████╔╝███████║██║   ██║██████╔╝██╔██╗ ██║██║██████╔╝",
        "██║  ██║██╔══██╗██╔══██║██║   ██║██╔═══╝ ██║╚██╗██║██║██╔══██╗",
        "██████╔╝██║  ██║██║  ██║╚██████╔╝██║     ██║ ╚████║██║██║  ██║",
        "╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝",
    ];

    let gold = Style::new().yellow();
    let gold_bold = Style::new().yellow().bold();

    let colored_logo: Vec<String> = LOGO
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let s = if i < 3 { gold_bold } else { gold };
            format!("  {}", line.if_supports_color(Stdout, |t| t.style(s)))
        })
        .collect();

    let separator = dim("  ─────────────────────────────────────────────────────────────────");
    let tagline = format!(
        "  {}{}{}",
        dim("Odin's ring for your vault"),
        dim(&" ".repeat(36 - version.len())),
        label(&format!("v{version}")),
    );

    format!(
        "\n{}\n{}\n{}\n",
        colored_logo.join("\n"),
        separator,
        tagline,
    )
}
