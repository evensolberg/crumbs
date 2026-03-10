use std::path::Path;

use anyhow::{Result, bail};
use chrono::Local;
use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui_core::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    terminal::Terminal,
};
use ratatui_crossterm::CrosstermBackend;
use ratatui_textarea::{Input, Key, TextArea};
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

/// Status of the in-TUI editor.
#[derive(PartialEq)]
enum EditorStatus {
    /// Normal editing mode.
    Editing,
    /// Ctrl-S was just pressed; show a brief "Saved" confirmation.
    Saved,
    /// Esc was pressed with unsaved changes; awaiting confirmation.
    ConfirmDiscard,
}

/// Outcome returned by `run_editor`.
enum EditorOutcome {
    /// User exited after saving at least once via Ctrl-S.
    /// `saved_lines` is the last Ctrl-S snapshot.
    Saved { saved_lines: Vec<String> },
    /// User exited cleanly with no changes and no Ctrl-S.
    NoChanges,
    /// User discarded changes (double-Esc) without saving.
    Discarded,
}

/// Run the TUI editor loop.
///
/// `save_fn` is called each time the user presses Ctrl-S; it receives the
/// current textarea lines and should write them to disk.  The loop tracks the
/// last snapshot written so it can decide whether there are unsaved changes
/// when Esc is pressed.
///
/// # Errors
///
/// Returns an error if drawing the terminal frame or reading an event fails,
/// or if `save_fn` returns an error.
#[allow(clippy::too_many_lines)]
fn run_editor(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    textarea: &mut TextArea,
    save_fn: &mut dyn FnMut(&[String]) -> Result<()>,
) -> Result<EditorOutcome> {
    let initial: Vec<String> = textarea.lines().to_vec();
    let mut last_saved: Option<Vec<String>> = None;
    let mut status = EditorStatus::Editing;

    loop {
        let (status_text, status_color) = match status {
            EditorStatus::Editing => ("  Ctrl-S save  │  Esc exit", Color::DarkGray),
            EditorStatus::Saved => ("  Saved  │  Ctrl-S save  │  Esc exit", Color::Green),
            EditorStatus::ConfirmDiscard => (
                "  Unsaved changes — Esc again to discard, any key to resume",
                Color::Yellow,
            ),
        };

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(f.area());

            #[allow(deprecated)]
            f.render_widget(textarea.widget(), chunks[0]);

            let bar = Paragraph::new(status_text).style(Style::default().fg(status_color));
            f.render_widget(bar, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match status {
                EditorStatus::ConfirmDiscard => {
                    if key.code == KeyCode::Esc {
                        // Second Esc — discard and exit.
                        return Ok(EditorOutcome::Discarded);
                    }
                    // Any other key cancels the discard prompt.
                    status = EditorStatus::Editing;
                    textarea.input(key);
                }

                EditorStatus::Editing | EditorStatus::Saved => {
                    match (key.modifiers, key.code) {
                        // Ctrl-S: save in place, stay in editor.
                        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                            save_fn(textarea.lines())?;
                            last_saved = Some(textarea.lines().to_vec());
                            status = EditorStatus::Saved;
                        }

                        // Ctrl-C / Esc: exit if clean, prompt if dirty.
                        (KeyModifiers::CONTROL, KeyCode::Char('c')) | (_, KeyCode::Esc) => {
                            let current = textarea.lines().to_vec();
                            let baseline = last_saved.as_ref().unwrap_or(&initial);
                            if current == *baseline {
                                return Ok(last_saved.map_or(EditorOutcome::NoChanges, |lines| {
                                    EditorOutcome::Saved { saved_lines: lines }
                                }));
                            }
                            status = EditorStatus::ConfirmDiscard;
                        }

                        // Word navigation.
                        // macOS Terminal/iTerm2 sends Option+Left/Right as Alt+b/f.
                        // Linux/Windows terminals send Ctrl+Left/Right or Alt+Left/Right.
                        (KeyModifiers::ALT, KeyCode::Char('b') | KeyCode::Left)
                        | (KeyModifiers::CONTROL, KeyCode::Left) => {
                            textarea.input(Input {
                                key: Key::Char('b'),
                                ctrl: false,
                                alt: true,
                                shift: false,
                            });
                        }
                        (KeyModifiers::ALT, KeyCode::Char('f') | KeyCode::Right)
                        | (KeyModifiers::CONTROL, KeyCode::Right) => {
                            textarea.input(Input {
                                key: Key::Char('f'),
                                ctrl: false,
                                alt: true,
                                shift: false,
                            });
                        }

                        // All other keys: pass through to textarea.
                        _ => {
                            if status == EditorStatus::Saved {
                                status = EditorStatus::Editing;
                            }
                            textarea.input(key);
                        }
                    }
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
///
/// # Errors
///
/// Returns an error if the file cannot be read.
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
#[must_use]
pub fn build_body_section(title: &str, body: &str) -> String {
    if body.is_empty() {
        format!("\n# {title}\n")
    } else {
        format!("\n# {title}\n\n{body}\n")
    }
}

/// Write the textarea lines back to `path`, updating frontmatter as needed.
fn write_lines(
    path: &Path,
    item: &mut crate::item::Item,
    lines: &[String],
    dir: &Path,
) -> Result<()> {
    let new_title = lines
        .first()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let new_body = lines
        .get(2..)
        .unwrap_or(&[])
        .join("\n")
        .trim_matches('\n')
        .to_string();

    let new_body_section = build_body_section(&new_title, &new_body);
    item.title = new_title;
    item.updated = Local::now().date_naive();
    item.description.clear();
    let frontmatter = serde_yaml_ng::to_string(item)?;
    let new_content = format!("---\n{frontmatter}---\n{new_body_section}");
    store::atomic_write(path, &new_content)?;
    store::reindex(dir)?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the item is not found, the file cannot be read or
/// written, or the TUI encounters an I/O error.
pub fn run(dir: &Path, id: &str) -> Result<()> {
    let Some((path, mut item)) = store::find_by_id(dir, id)? else {
        bail!("no item found with id: {id}");
    };

    let (title, body) = extract_title_and_body(&path)?;

    // Build TextArea: line 0 = title, line 1 = blank, lines 2+ = body
    let mut lines = vec![title, String::new()];
    for line in body.lines() {
        lines.push(line.to_string());
    }

    let mut textarea = TextArea::from(lines);

    // Enter raw mode; guard restores terminal on any exit path
    let mut terminal = setup_terminal()?;
    let guard = TerminalGuard;

    // Build a save closure that writes to disk immediately.
    let save_fn = {
        let path = path.clone();
        let dir = dir.to_path_buf();
        move |lines: &[String]| -> Result<()> {
            let new_title = lines
                .first()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            let new_body = lines
                .get(2..)
                .unwrap_or(&[])
                .join("\n")
                .trim_matches('\n')
                .to_string();
            let new_body_section = build_body_section(&new_title, &new_body);
            // We only update the body/title here; full item update happens on exit.
            let raw = std::fs::read_to_string(&path)?;
            let frontmatter_raw = raw
                .strip_prefix("---\n")
                .and_then(|s| s.split_once("\n---\n").map(|(fm, _)| fm))
                .unwrap_or("");
            let new_content = format!("---\n{frontmatter_raw}\n---\n{new_body_section}");
            store::atomic_write(&path, &new_content)?;
            store::reindex(&dir)?;
            Ok(())
        }
    };
    let mut save_fn = save_fn;

    let outcome = run_editor(&mut terminal, &mut textarea, &mut save_fn)?;

    // Restore terminal before any output.
    drop(guard);
    drop(terminal);

    match outcome {
        EditorOutcome::NoChanges | EditorOutcome::Discarded => {
            // Nothing to write — either no changes were made, or the user
            // discarded changes via double-Esc.
        }
        EditorOutcome::Saved { saved_lines } => {
            // Do a final authoritative write that updates the Item struct
            // (title, updated date) using the last Ctrl-S snapshot.
            write_lines(&path, &mut item, &saved_lines, dir)?;
            println!("Updated {}", item.id);
        }
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
