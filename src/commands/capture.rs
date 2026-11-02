use crate::models::config::Config;
use crate::style;
use crate::vault;
use anyhow::{Context, Result};

/// Quick-capture a message to the vault inbox as a markdown file.
pub fn run(config: &Config, project_key: &str, message: &str) -> Result<()> {
    let vault_root = config.resolved_vault_root()?;
    let inbox = vault::inbox_dir(&vault_root);

    if !inbox.exists() {
        std::fs::create_dir_all(&inbox)
            .with_context(|| format!("Creating inbox directory at {}", inbox.display()))?;
    }

    let date = vault::today().format("%Y-%m-%d").to_string();
    let hhmm = vault::now_hhmm();
    let slug = make_slug(message);
    let filename = format!("{}-{}-{}.md", date, hhmm, slug);
    let file_path = inbox.join(&filename);

    let content = format!(
        "---\ndate: {}\nproject: {}\ntags: [inbox, capture]\n---\n\n{}\n",
        date,
        project_key,
        message.trim()
    );

    std::fs::write(&file_path, content)
        .with_context(|| format!("Writing capture file {}", file_path.display()))?;

    println!("{}", style::dim(&file_path.display().to_string()));
    Ok(())
}

/// Derive a filesystem-safe slug from the first few words of a message.
/// Keeps the first 5 words, lowercased, non-alphanumeric chars replaced with `-`,
/// consecutive `-` collapsed, leading/trailing `-` stripped.
fn make_slug(message: &str) -> String {
    let words: Vec<&str> = message.split_whitespace().take(5).collect();
    let joined = words.join("-");

    let slug: String = joined
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    // Collapse consecutive dashes and strip leading/trailing ones
    let mut result = String::new();
    let mut prev_dash = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_dash {
                result.push(c);
            }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }

    let result = result.trim_matches('-').to_string();
    if result.is_empty() {
        "capture".to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::make_slug;

    #[test]
    fn slug_basic() {
        assert_eq!(
            make_slug("Hello world foo bar baz qux"),
            "hello-world-foo-bar-baz"
        );
    }

    #[test]
    fn slug_punctuation() {
        assert_eq!(
            make_slug("What the hell?! Is this..."),
            "what-the-hell-is-this"
        );
    }

    #[test]
    fn slug_empty() {
        assert_eq!(make_slug("!!!"), "capture");
    }

    #[test]
    fn slug_short() {
        assert_eq!(make_slug("ship it"), "ship-it");
    }
}
