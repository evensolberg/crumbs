use std::path::Path;

use anyhow::Result;

/// Extract the title and body from an item's `.md` file.
///
/// The file format is:
/// ```text
/// ---
/// <YAML frontmatter>
/// ---
///
/// # Title
///
/// Body text...
/// ```
///
/// Returns `(title, body)` where `body` is everything after the heading line,
/// with leading/trailing newlines stripped.
pub fn extract_title_and_body(path: &Path) -> Result<(String, String)> {
    let raw = std::fs::read_to_string(path)?;
    let body_section = raw
        .strip_prefix("---\n")
        .and_then(|s| s.split_once("\n---\n").map(|(_, b)| b))
        .unwrap_or("");
    let trimmed = body_section.trim_start_matches('\n');
    let (heading, rest) = trimmed.split_once('\n').unwrap_or((trimmed, ""));
    let title = heading.trim_start_matches('#').trim().to_string();
    let body = rest.trim_matches('\n').to_string();
    Ok((title, body))
}

/// Reassemble the markdown body section from a title and body string.
///
/// Returns the string that goes after `---\n` (the closing frontmatter fence),
/// including the leading newline.
pub fn build_body_section(title: &str, body: &str) -> String {
    if body.is_empty() {
        format!("\n# {title}\n")
    } else {
        format!("\n# {title}\n\n{body}\n")
    }
}

pub fn run(_dir: &Path, _id: &str) -> Result<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    fn write_item_file(title: &str, body: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        let content = if body.is_empty() {
            format!("---\nid: ts-aaa\ntitle: {title}\n---\n\n# {title}\n")
        } else {
            format!("---\nid: ts-aaa\ntitle: {title}\n---\n\n# {title}\n\n{body}\n")
        };
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn extract_title_and_body_no_body() {
        let f = write_item_file("Fix login bug", "");
        let (title, body) = extract_title_and_body(f.path()).unwrap();
        assert_eq!(title, "Fix login bug");
        assert_eq!(body, "");
    }

    #[test]
    fn extract_title_and_body_with_body() {
        let f = write_item_file("Fix login bug", "Some description here.\n\nMore text.");
        let (title, body) = extract_title_and_body(f.path()).unwrap();
        assert_eq!(title, "Fix login bug");
        assert_eq!(body, "Some description here.\n\nMore text.");
    }

    #[test]
    fn build_body_section_no_body() {
        let result = build_body_section("Fix login bug", "");
        assert_eq!(result, "\n# Fix login bug\n");
    }

    #[test]
    fn build_body_section_with_body() {
        let result = build_body_section("Fix login bug", "Some description.");
        assert_eq!(result, "\n# Fix login bug\n\nSome description.\n");
    }

    #[test]
    fn build_body_section_roundtrip() {
        // build_body_section output, when parsed by extract_title_and_body,
        // should round-trip cleanly.
        let title = "My Task";
        let body = "Line one.\n\nLine two.";
        let section = build_body_section(title, body);
        // Wrap in fake frontmatter to simulate full file
        let full = format!("---\nid: ts-aaa\ntitle: {title}\n---\n{section}");
        let mut f = tempfile::NamedTempFile::new().unwrap();
        use std::io::Write as _;
        f.write_all(full.as_bytes()).unwrap();
        let (rt_title, rt_body) = extract_title_and_body(f.path()).unwrap();
        assert_eq!(rt_title, title);
        assert_eq!(rt_body, body);
    }
}
