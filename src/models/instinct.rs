use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

/// A parsed instinct entry from the instincts.md registry.
#[derive(Debug, Clone, Serialize)]
pub struct Instinct {
    pub category: String,
    pub title: String,
    pub first_seen: String,
    pub confirmations: u32,
    pub confidence: String,
    pub context: String,
    pub pattern: String,
    pub evidence: String,
}

/// Stats summary for instinct registry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstinctStats {
    pub total: usize,
    pub by_confidence: InstinctConfidenceCounts,
    pub by_category: Vec<(String, usize)>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstinctConfidenceCounts {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

static HEADING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^### \[([^\]]+)\] (.+)$").unwrap());
static FIELD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^- \*\*([^*]+)\*\*:\s*(.+)$").unwrap());

/// Parse the instincts.md file into a list of Instinct structs.
/// Handles multiline field values (accumulates lines until the next `- **` or `###`).
pub fn parse_instincts(content: &str) -> Vec<Instinct> {
    let mut instincts = Vec::new();
    let mut current: Option<InstinctBuilder> = None;
    // Track which field is currently accumulating multiline content
    let mut active_field: Option<String> = None;

    for line in content.lines() {
        if let Some(caps) = HEADING_RE.captures(line) {
            // Flush previous instinct
            if let Some(builder) = current.take() {
                instincts.push(builder.build());
            }
            active_field = None;
            current = Some(InstinctBuilder {
                category: caps[1].to_string(),
                title: caps[2].to_string(),
                ..Default::default()
            });
        } else if let Some(ref mut builder) = current {
            if let Some(caps) = FIELD_RE.captures(line) {
                let field_name = caps[1].trim().to_string();
                let value = caps[2].trim().to_string();
                match field_name.as_str() {
                    "First seen" => {
                        builder.first_seen = value;
                        active_field = None;
                    }
                    "Confirmations" => {
                        // Parse "2 (2026-02-26, 2026-04-06)" → 2
                        builder.confirmations = value
                            .split_whitespace()
                            .next()
                            .and_then(|n| n.parse().ok())
                            .unwrap_or(1);
                        active_field = None;
                    }
                    "Confidence" => {
                        builder.confidence = value;
                        active_field = None;
                    }
                    "Context" => {
                        builder.context = value;
                        active_field = Some("Context".to_string());
                    }
                    "Pattern" => {
                        builder.pattern = value;
                        active_field = Some("Pattern".to_string());
                    }
                    "Evidence" => {
                        builder.evidence = value;
                        active_field = Some("Evidence".to_string());
                    }
                    _ => {
                        active_field = None;
                    }
                }
            } else if let Some(ref field) = active_field {
                // Continuation line for a multiline field
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let target = match field.as_str() {
                        "Context" => &mut builder.context,
                        "Pattern" => &mut builder.pattern,
                        "Evidence" => &mut builder.evidence,
                        _ => continue,
                    };
                    target.push('\n');
                    target.push_str(trimmed);
                }
            }
        }
    }

    // Flush last instinct
    if let Some(builder) = current {
        instincts.push(builder.build());
    }

    instincts
}

#[derive(Default)]
struct InstinctBuilder {
    category: String,
    title: String,
    first_seen: String,
    confirmations: u32,
    confidence: String,
    context: String,
    pattern: String,
    evidence: String,
}

impl InstinctBuilder {
    fn build(self) -> Instinct {
        Instinct {
            category: self.category,
            title: self.title,
            first_seen: self.first_seen,
            confirmations: self.confirmations,
            confidence: self.confidence,
            context: self.context,
            pattern: self.pattern,
            evidence: self.evidence,
        }
    }
}

/// Record one more confirmation on the entry titled `title` (case-insensitive) and set its
/// confidence from the count: 1 low, 2 medium, 3+ high. Returns the new text and count,
/// or None when no entry has that title.
pub fn confirm_entry(content: &str, title: &str, date: &str) -> Option<(String, u32)> {
    let mut out = Vec::new();
    let mut in_target = false;
    let mut count = None;
    let mut first_seen = None;
    for line in content.lines() {
        if let Some(caps) = HEADING_RE.captures(line) {
            in_target = count.is_none() && caps[2].trim().eq_ignore_ascii_case(title.trim());
        } else if in_target && let Some(caps) = FIELD_RE.captures(line) {
            let value = caps[2].trim();
            match caps[1].trim() {
                "First seen" => first_seen = value.split_whitespace().next().map(str::to_string),
                "Confirmations" => {
                    // "2 (2026-02-26, 2026-04-06)" → "3 (2026-02-26, 2026-04-06, <date>)"
                    let n: u32 = value
                        .split_whitespace()
                        .next()
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(1)
                        + 1;
                    let dates = value
                        .split_once('(')
                        .map(|(_, d)| d.trim_end_matches(')').trim().to_string())
                        .filter(|d| !d.is_empty());
                    // A bare count of 1 has no dates; its one confirmation is First seen.
                    let dates = match (dates, &first_seen) {
                        (Some(d), _) => format!("{d}, {date}"),
                        (None, Some(f)) if n == 2 => format!("{f}, {date}"),
                        (None, _) => date.to_string(),
                    };
                    out.push(format!("- **Confirmations**: {n} ({dates})"));
                    count = Some(n);
                    continue;
                }
                "Confidence" if count.is_some() => {
                    let level = match count {
                        Some(n) if n >= 3 => "high",
                        Some(2) => "medium",
                        _ => "low",
                    };
                    out.push(format!("- **Confidence**: {level}"));
                    continue;
                }
                _ => {}
            }
        }
        out.push(line.to_string());
    }
    let mut text = out.join("\n");
    if content.ends_with('\n') {
        text.push('\n');
    }
    count.map(|n| (text, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_instinct() {
        let content = r#"
### [TCA] Avoid global state in reducers
- **First seen**: 2026-01-15
- **Confirmations**: 3
- **Confidence**: high
- **Context**: TCA reducer design
- **Pattern**: Use dependency injection
- **Evidence**: Seen in 3 sessions
"#;
        let result = parse_instincts(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category, "TCA");
        assert_eq!(result[0].title, "Avoid global state in reducers");
        assert_eq!(result[0].confirmations, 3);
        assert_eq!(result[0].confidence, "high");
    }

    #[test]
    fn parse_multiline_pattern() {
        let content = r#"
### [Swift] Use structured concurrency
- **First seen**: 2026-02-01
- **Confirmations**: 1
- **Confidence**: low
- **Context**: Async code
- **Pattern**: Always prefer async/await over callbacks.
  Use TaskGroup for parallel work.
  Avoid detached tasks.
- **Evidence**: Observed once
"#;
        let result = parse_instincts(content);
        assert_eq!(result.len(), 1);
        assert!(result[0].pattern.contains("TaskGroup for parallel work"));
        assert!(result[0].pattern.contains("Avoid detached tasks"));
        // Evidence should NOT include pattern continuation lines
        assert_eq!(result[0].evidence, "Observed once");
    }

    #[test]
    fn parse_confirmations_with_dates() {
        let content = r#"
### [Git] Commit early
- **First seen**: 2026-01-01
- **Confirmations**: 5 (2026-01-01, 2026-01-15, 2026-02-01, 2026-02-15, 2026-03-01)
- **Confidence**: high
- **Context**: Git workflow
- **Pattern**: Commit often
- **Evidence**: Many sessions
"#;
        let result = parse_instincts(content);
        assert_eq!(result[0].confirmations, 5);
    }

    #[test]
    fn parse_multiple_instincts() {
        let content = r#"
### [A] First
- **First seen**: 2026-01-01
- **Confirmations**: 1
- **Confidence**: low
- **Context**: ctx1
- **Pattern**: pat1
- **Evidence**: ev1

### [B] Second
- **First seen**: 2026-02-01
- **Confirmations**: 2
- **Confidence**: medium
- **Context**: ctx2
- **Pattern**: pat2
- **Evidence**: ev2
"#;
        let result = parse_instincts(content);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].category, "A");
        assert_eq!(result[1].category, "B");
    }

    #[test]
    fn serialize_roundtrip() {
        let inst = Instinct {
            category: "Test".to_string(),
            title: "Test instinct".to_string(),
            first_seen: "2026-01-01".to_string(),
            confirmations: 1,
            confidence: "low".to_string(),
            context: "ctx".to_string(),
            pattern: "pat".to_string(),
            evidence: "ev".to_string(),
        };
        let json = serde_json::to_string(&inst).unwrap();
        assert!(json.contains("\"category\":\"Test\""));
        assert!(json.contains("\"confirmations\":1"));
    }

    #[test]
    fn filter_by_confidence() {
        let content = r#"
### [A] High one
- **First seen**: 2026-01-01
- **Confirmations**: 5
- **Confidence**: high
- **Context**: c
- **Pattern**: p
- **Evidence**: e

### [B] Low one
- **First seen**: 2026-01-01
- **Confirmations**: 1
- **Confidence**: low
- **Context**: c
- **Pattern**: p
- **Evidence**: e
"#;
        let instincts = parse_instincts(content);
        let high: Vec<_> = instincts
            .iter()
            .filter(|i| i.confidence.eq_ignore_ascii_case("high"))
            .collect();
        assert_eq!(high.len(), 1);
        assert_eq!(high[0].title, "High one");
    }

    #[test]
    fn filter_by_category() {
        let content = r#"
### [TCA] TCA thing
- **First seen**: 2026-01-01
- **Confirmations**: 1
- **Confidence**: low
- **Context**: c
- **Pattern**: p
- **Evidence**: e

### [Swift] Swift thing
- **First seen**: 2026-01-01
- **Confirmations**: 1
- **Confidence**: low
- **Context**: c
- **Pattern**: p
- **Evidence**: e
"#;
        let instincts = parse_instincts(content);
        let tca: Vec<_> = instincts
            .iter()
            .filter(|i| i.category.eq_ignore_ascii_case("tca"))
            .collect();
        assert_eq!(tca.len(), 1);
        assert_eq!(tca[0].title, "TCA thing");
    }

    #[test]
    fn confirm_bumps_count_dates_and_confidence() {
        let content = "### [Git] Other\n- **Confirmations**: 1\n- **Confidence**: low\n\n\
                       ### [Git] Pin images\n- **First seen**: 2026-01-01\n\
                       - **Confirmations**: 2 (2026-01-01, 2026-02-01)\n- **Confidence**: medium\n";
        let (text, n) = confirm_entry(content, "pin IMAGES", "2026-03-01").unwrap();
        assert_eq!(n, 3);
        assert!(text.contains(
            "- **Confirmations**: 3 (2026-01-01, 2026-02-01, 2026-03-01)\n- **Confidence**: high\n"
        ));
        assert!(
            text.starts_with("### [Git] Other\n- **Confirmations**: 1\n- **Confidence**: low\n")
        );
        assert_eq!(parse_instincts(&text)[1].confidence, "high");

        let (text, n) = confirm_entry(
            "### [CI] X\n- **Confirmations**: 1\n- **Confidence**: low",
            "X",
            "2026-03-01",
        )
        .unwrap();
        assert_eq!(n, 2);
        assert_eq!(
            text,
            "### [CI] X\n- **Confirmations**: 2 (2026-03-01)\n- **Confidence**: medium"
        );

        let fresh =
            "### [CI] Y\n- **First seen**: 2026-02-01 (session: s)\n- **Confirmations**: 1\n";
        let (text, _) = confirm_entry(fresh, "Y", "2026-03-01").unwrap();
        assert!(text.contains("- **Confirmations**: 2 (2026-02-01, 2026-03-01)"));

        assert!(confirm_entry(content, "missing", "2026-03-01").is_none());
    }
}

/// Compute stats from a list of instincts.
pub fn compute_stats(instincts: &[Instinct]) -> InstinctStats {
    let mut by_category: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;

    for inst in instincts {
        *by_category.entry(inst.category.clone()).or_default() += 1;
        match inst.confidence.as_str() {
            "high" => high += 1,
            "medium" => medium += 1,
            _ => low += 1,
        }
    }

    let mut categories: Vec<(String, usize)> = by_category.into_iter().collect();
    categories.sort_by_key(|c| std::cmp::Reverse(c.1));

    InstinctStats {
        total: instincts.len(),
        by_confidence: InstinctConfidenceCounts { high, medium, low },
        by_category: categories,
    }
}
