use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize)]
pub struct Decision {
    pub date: String,
    pub decision: String,
    pub why: String,
}

static TABLE_ROW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|$").unwrap());

/// Parse markdown table rows from a Decision Log section into Decision structs.
/// Skips the header row and separator row (lines containing `---`).
pub fn parse_decision_rows(section: &str) -> Vec<Decision> {
    let mut decisions = Vec::new();

    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.contains("---") {
            continue;
        }
        // Skip the header row (contains "Date" or "Decision" literally)
        if trimmed.contains("| Date") || trimmed.contains("| decision") {
            continue;
        }

        if let Some(caps) = TABLE_ROW_RE.captures(trimmed) {
            let date = caps[1].trim().to_string();
            let decision = caps[2].trim().to_string();
            let why = caps[3].trim().to_string();

            // Skip if this looks like a header (non-date in date column)
            if date.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                decisions.push(Decision {
                    date,
                    decision,
                    why,
                });
            }
        }
    }

    decisions
}

/// Extract the raw "## Decision Log" section from file content.
/// Returns everything from the `## Decision Log` header to the next `## ` header or EOF.
pub fn extract_decision_section(content: &str) -> Option<&str> {
    let start = content.find("## Decision Log")?;
    let after_header = &content[start..];

    // Find the next ## heading after the Decision Log heading itself
    let section_end = after_header[1..]
        .find("\n## ")
        .map(|pos| pos + 1)
        .unwrap_or(after_header.len());

    Some(&content[start..start + section_end])
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SECTION: &str = r#"## Decision Log

| Date | Decision | Why |
|------|----------|-----|
| 2026-04-23 | Ship without accounts | Fewer screens before launch |
| 2026-05-01 | Drop the export feature | Nobody asked for it |
| 2026-05-04 | WebSocket over polling | Lower latency for real-time state |
"#;

    #[test]
    fn parse_rows_from_section() {
        let decisions = parse_decision_rows(SAMPLE_SECTION);
        assert_eq!(decisions.len(), 3);
        assert_eq!(decisions[0].date, "2026-04-23");
        assert_eq!(decisions[0].decision, "Ship without accounts");
        assert_eq!(decisions[0].why, "Fewer screens before launch");
        assert_eq!(decisions[2].decision, "WebSocket over polling");
    }

    #[test]
    fn parse_empty_section() {
        let section = "## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n";
        let decisions = parse_decision_rows(section);
        assert_eq!(decisions.len(), 0);
    }

    #[test]
    fn extract_section_at_end_of_file() {
        let content = "# Product Scope\n\nSome content here.\n\n## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n| 2026-05-01 | Something | Reason |\n";
        let section = extract_decision_section(content).unwrap();
        assert!(section.starts_with("## Decision Log"));
        assert!(section.contains("2026-05-01"));
    }

    #[test]
    fn extract_section_with_following_heading() {
        let content = "## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n| 2026-05-01 | X | Y |\n\n## Another Section\n\nStuff here.\n";
        let section = extract_decision_section(content).unwrap();
        assert!(section.contains("2026-05-01"));
        assert!(!section.contains("Another Section"));
    }

    #[test]
    fn extract_section_missing() {
        let content = "# Product Scope\n\nNo decision log here.\n";
        assert!(extract_decision_section(content).is_none());
    }

    #[test]
    fn parse_row_with_pipes_in_content() {
        // Ensures we only match the outer pipes (3-column table)
        let section = "| 2026-05-04 | Use A over B | B has issues |\n";
        let decisions = parse_decision_rows(section);
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].decision, "Use A over B");
    }
}
