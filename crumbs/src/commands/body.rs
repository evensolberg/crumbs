use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;
use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use dialoguer::Confirm;
use ratatui_core::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    terminal::Terminal,
};
use ratatui_crossterm::CrosstermBackend;
use ratatui_textarea::TextArea;
use ratatui_widgets::paragraph::Paragraph;

use crate::store;

// serde_yaml_ng is a direct dep of the crumbs crate (see Cargo.toml)

/// RAII guard that restores the terminal on drop.
///
/// This ensures the terminal is always restored even if `run` returns early
/// via `?` or panics.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, Show);
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

/// Run the TUI editor loop.
///
/// Returns `true` if the user pressed Ctrl-S (explicit save),
/// `false` if they pressed Ctrl-C or Esc (cancel/exit).
fn run_editor(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    textarea: &mut TextArea,
) -> Result<bool> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(f.area());

            f.render_widget(textarea.widget(), chunks[0]);

            let status = Paragraph::new("  Ctrl-S save  │  Ctrl-C / Esc cancel")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(status, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => return Ok(true),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) | (_, KeyCode::Esc) => {
                    return Ok(false);
                }
                _ => {
                    textarea.input(key);
                }
            }
        }
    }
}

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

pub fn run(dir: &Path, id: &str) -> Result<()> {
    let (path, mut item) = match store::find_by_id(dir, id)? {
        None => bail!("no item found with id: {id}"),
        Some(found) => found,
    };

    let (title, body) = extract_title_and_body(&path)?;

    // Build TextArea: line 0 = title, line 1 = blank, lines 2+ = body
    let mut lines = vec![title.clone(), String::new()];
    for line in body.lines() {
        lines.push(line.to_string());
    }

    let mut textarea = TextArea::from(lines);
    let original_lines: Vec<String> = textarea.lines().to_vec();

    // Enter raw mode; guard restores terminal on any exit path
    let mut terminal = setup_terminal()?;
    let _guard = TerminalGuard;

    let saved = run_editor(&mut terminal, &mut textarea)?;

    // Restore terminal before any dialoguer prompt
    drop(_guard);
    drop(terminal);

    let new_lines: Vec<String> = textarea.lines().to_vec();
    let changed = new_lines != original_lines;

    let should_save = if saved {
        true
    } else if changed {
        Confirm::new()
            .with_prompt("Save changes?")
            .default(true)
            .interact()?
    } else {
        Confirm::new()
            .with_prompt("No changes — save anyway?")
            .default(false)
            .interact()?
    };

    if should_save {
        let new_title = new_lines
            .first()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let new_body = new_lines
            .get(2..)
            .unwrap_or(&[])
            .join("\n")
            .trim_matches('\n')
            .to_string();

        let new_body_section = build_body_section(&new_title, &new_body);
        item.title = new_title;
        item.updated = Local::now().date_naive();
        item.description.clear();
        let frontmatter = serde_yaml_ng::to_string(&item)?;
        let new_content = format!("---\n{frontmatter}---\n{new_body_section}");
        store::atomic_write(&path, &new_content)?;
        store::reindex(dir)?;
        println!("Updated {}", item.id);
    }

    Ok(())
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
