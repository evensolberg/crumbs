/// Expand `:shortcode:` sequences to Unicode emoji in `text`.
///
/// Returns `Cow::Borrowed` when no shortcode is found (zero allocation).
/// Returns `Cow::Owned` when at least one shortcode was replaced.
///
/// Fenced code blocks (` ``` ` / `~~~`) and inline backtick spans are
/// skipped — their contents pass through unchanged.
///
/// Unknown shortcodes (`:notreal:`) are preserved as-is.
#[must_use]
pub fn expand_shortcodes(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.contains(':') {
        return std::borrow::Cow::Borrowed(text);
    }

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut result = String::new();
    let mut modified = false;
    let mut i = 0;
    let mut in_fenced = false;
    let mut fence_char = ' ';

    while i < len {
        // Detect fenced code block openers/closers at start of line.
        // We check for ``` or ~~~ at the beginning of a "line" (after \n or at start).
        let at_line_start = i == 0 || (i > 0 && chars[i - 1] == '\n');
        if at_line_start && i + 2 < len {
            let c = chars[i];
            if (c == '`' || c == '~') && chars[i + 1] == c && chars[i + 2] == c {
                if !in_fenced {
                    in_fenced = true;
                    fence_char = c;
                } else if c == fence_char {
                    in_fenced = false;
                }
                if !modified {
                    result.push_str(&text[..char_byte_offset(&chars, i)]);
                    modified = true;
                }
                result.push(chars[i]);
                result.push(chars[i + 1]);
                result.push(chars[i + 2]);
                i += 3;
                continue;
            }
        }

        if in_fenced {
            if modified {
                result.push(chars[i]);
            }
            i += 1;
            continue;
        }

        // Skip inline code spans delimited by backticks.
        if chars[i] == '`' {
            if !modified {
                result.push_str(&text[..char_byte_offset(&chars, i)]);
                modified = true;
            }
            i = skip_inline_code_span(&chars, i, &mut result);
            continue;
        }

        // Try to match a shortcode starting with ':'.
        if chars[i] == ':' {
            let name_start = i + 1;
            let mut j = name_start;
            while j < len && is_shortcode_char(chars[j]) {
                j += 1;
            }
            let name_len = j - name_start;
            if (1..=64).contains(&name_len) && j < len && chars[j] == ':' {
                // Valid shortcode syntax — attempt lookup.
                let name: String = chars[name_start..j].iter().collect();
                if let Some(emoji) = emojis::get_by_shortcode(&name) {
                    if !modified {
                        result.push_str(&text[..char_byte_offset(&chars, i)]);
                        modified = true;
                    }
                    result.push_str(emoji.as_str());
                    i = j + 1; // skip past closing ':'
                    continue;
                }
            }
            // No match — emit ':' and continue.
            if modified {
                result.push(':');
            }
            i += 1;
            continue;
        }

        if modified {
            result.push(chars[i]);
        }
        i += 1;
    }

    if modified {
        std::borrow::Cow::Owned(result)
    } else {
        std::borrow::Cow::Borrowed(text)
    }
}

/// Emit an inline backtick code span starting at `chars[i]` into `result`.
/// Returns the index immediately after the closing ticks, or `chars.len()` if
/// no matching closing delimiter is found.
fn skip_inline_code_span(chars: &[char], mut i: usize, result: &mut String) -> usize {
    let len = chars.len();
    let mut tick_count = 0;
    while i + tick_count < len && chars[i + tick_count] == '`' {
        tick_count += 1;
    }
    for _ in 0..tick_count {
        result.push('`');
    }
    i += tick_count;
    while i < len {
        if chars[i] == '`' {
            let mut close_count = 0;
            while i + close_count < len && chars[i + close_count] == '`' {
                close_count += 1;
            }
            for _ in 0..close_count {
                result.push('`');
            }
            i += close_count;
            if close_count == tick_count {
                break;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    i
}

#[inline]
const fn is_shortcode_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-'
}

/// Return the byte offset in the original string for `chars[idx]`.
fn char_byte_offset(chars: &[char], idx: usize) -> usize {
    chars[..idx].iter().map(|c| c.len_utf8()).sum()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_shortcode_expands() {
        let result = expand_shortcodes(":smile:");
        assert_eq!(result, "😄");
    }

    #[test]
    fn unknown_shortcode_preserved() {
        let result = expand_shortcodes(":notarealcode:");
        assert_eq!(result, ":notarealcode:");
    }

    #[test]
    fn no_shortcodes_returns_borrowed() {
        let input = "hello world";
        let result = expand_shortcodes(input);
        assert!(matches!(result, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn multiple_shortcodes() {
        let result = expand_shortcodes(":tada: done :white_check_mark:");
        assert_eq!(result, "🎉 done ✅");
    }

    #[test]
    fn fenced_code_block_preserved() {
        let input = "```\n:smile:\n```";
        let result = expand_shortcodes(input);
        assert_eq!(result, input);
    }

    #[test]
    fn inline_code_preserved() {
        let input = "`:smile:`";
        let result = expand_shortcodes(input);
        assert_eq!(result, input);
    }

    #[test]
    fn partial_colon_not_expanded() {
        let result = expand_shortcodes(":smile");
        assert_eq!(result, ":smile");
    }

    #[test]
    fn empty_colons_not_expanded() {
        // "::" has name_len == 0 (< 1), so it passes through unchanged
        let result = expand_shortcodes("::");
        assert_eq!(result, "::");
    }

    #[test]
    fn plus_one_shortcode() {
        let result = expand_shortcodes(":+1:");
        assert_eq!(result, "👍");
    }
}
