use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A single entry in the session index JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEntry {
    pub id: String,
    pub project: String,
    pub date: String,
    #[serde(default)]
    pub ticket_ids: Vec<String>,
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub path: String,
    #[serde(default)]
    pub linked_prs: Vec<String>,
    pub generated_at: String,
}

/// The full session index file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIndex {
    pub generated_at: String,
    pub count: usize,
    pub sessions: Vec<SessionEntry>,
}

/// Frontmatter parsed from a session log markdown file.
/// Used by session write and index rebuild to generate filenames.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SessionFrontmatter {
    pub date: NaiveDate,
    pub time: String,
    pub topic: String,
    pub tags: Vec<String>,
}

impl SessionFrontmatter {
    /// Generate the canonical filename for this session.
    #[allow(dead_code)]
    pub fn filename(&self, project: &str) -> String {
        format!(
            "{}-{}-{}-{}.md",
            self.date.format("%Y-%m-%d"),
            self.time.replace(':', ""),
            project,
            self.topic
        )
    }

    /// Generate the session ID (filename without .md).
    #[allow(dead_code)]
    pub fn id(&self, project: &str) -> String {
        let filename = self.filename(project);
        filename.trim_end_matches(".md").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(
        id: &str,
        date: &str,
        title: &str,
        tags: &[&str],
        tickets: &[&str],
    ) -> SessionEntry {
        SessionEntry {
            id: id.to_string(),
            project: "test".to_string(),
            date: date.to_string(),
            ticket_ids: tickets.iter().map(|s| s.to_string()).collect(),
            title: title.to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            path: format!("Areas/Work/Session-Logs/{}.md", id),
            linked_prs: vec![],
            generated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn sample_entries() -> Vec<SessionEntry> {
        vec![
            make_entry(
                "s1",
                "2026-03-15",
                "Auth migration",
                &["auth", "backend"],
                &["ACME-14"],
            ),
            make_entry(
                "s2",
                "2026-03-10",
                "Map feature",
                &["map", "ios"],
                &["ACME-89"],
            ),
            make_entry(
                "s3",
                "2026-02-20",
                "Build optimization",
                &["build", "ci"],
                &[],
            ),
            make_entry("s4", "2026-01-15", "Initial setup", &["setup"], &[]),
        ]
    }

    #[test]
    fn search_by_query() {
        let entries = sample_entries();
        let results: Vec<_> = entries
            .iter()
            .filter(|e| e.title.to_lowercase().contains("auth"))
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "s1");
    }

    #[test]
    fn search_by_tag() {
        let entries = sample_entries();
        let results: Vec<_> = entries
            .iter()
            .filter(|e| e.tags.iter().any(|t| t.eq_ignore_ascii_case("ios")))
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "s2");
    }

    #[test]
    fn search_by_ticket() {
        let entries = sample_entries();
        let results: Vec<_> = entries
            .iter()
            .filter(|e| {
                e.ticket_ids
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case("acme-14"))
            })
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "s1");
    }

    #[test]
    fn search_by_since() {
        let since = NaiveDate::parse_from_str("2026-03-01", "%Y-%m-%d").unwrap();
        let entries = sample_entries();
        let results: Vec<_> = entries
            .iter()
            .filter(|e| {
                NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                    .map(|d| d >= since)
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_combined_filters() {
        let entries = sample_entries();
        let query = "map";
        let tag = "ios";
        let results: Vec<_> = entries
            .iter()
            .filter(|e| {
                e.title.to_lowercase().contains(query)
                    && e.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
            })
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "s2");
    }

    #[test]
    fn search_no_results() {
        let entries = sample_entries();
        let results: Vec<_> = entries
            .iter()
            .filter(|e| e.title.to_lowercase().contains("nonexistent"))
            .collect();
        assert!(results.is_empty());
    }

    #[test]
    fn sort_order_preserved() {
        let mut entries = vec![
            make_entry("s2", "2026-01-01", "B", &[], &[]),
            make_entry("s1", "2026-03-01", "A", &[], &[]),
            make_entry("s3", "2026-02-01", "C", &[], &[]),
        ];
        entries.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.id.cmp(&a.id)));
        assert_eq!(entries[0].id, "s1"); // 2026-03-01
        assert_eq!(entries[1].id, "s3"); // 2026-02-01
        assert_eq!(entries[2].id, "s2"); // 2026-01-01
    }
}
