//! Link and image parsing.
//!
//! Handles:
//! - Inline links: `[text](url "title")`
//! - Images: `![alt](url "title")`
//! - Autolinks: `<https://example.com>` and `<email@example.com>`

use crate::limits;

/// A resolved link or image.
#[derive(Debug, Clone)]
pub struct Link {
    /// Start of the opening bracket (or `!` for images).
    pub start: u32,
    /// End of the link text (position of `]`).
    pub text_end: u32,
    /// Start of the URL.
    pub url_start: u32,
    /// End of the URL.
    pub url_end: u32,
    /// Start of the title (if present).
    pub title_start: Option<u32>,
    /// End of the title (if present).
    pub title_end: Option<u32>,
    /// End of the entire link (after closing `)`).
    pub end: u32,
    /// Whether this is an image.
    pub is_image: bool,
}

/// A resolved autolink.
#[derive(Debug, Clone, Copy)]
pub struct Autolink {
    /// Start position (the `<`).
    pub start: u32,
    /// End position (after the `>`).
    pub end: u32,
    /// Start of the URL/email content.
    pub content_start: u32,
    /// End of the URL/email content.
    pub content_end: u32,
    /// Whether this is an email autolink.
    pub is_email: bool,
}

/// Parse links from text, given bracket positions.
/// Returns list of resolved links.
pub fn resolve_links(
    text: &[u8],
    open_brackets: &[(u32, bool)], // (position, is_image)
    close_brackets: &[u32],
) -> Vec<Link> {
    let mut links = Vec::new();
    let mut used_opens: Vec<bool> = vec![false; open_brackets.len()];
    let mut used_closes: Vec<bool> = vec![false; close_brackets.len()];

    // Process open brackets from right to left (innermost first)
    for (open_idx, &(open_pos, is_image)) in open_brackets.iter().enumerate().rev() {
        if used_opens[open_idx] {
            continue;
        }

        // Find the matching close bracket (accounting for nested brackets)
        let close_idx = find_matching_close(
            open_pos,
            open_brackets,
            close_brackets,
            &used_opens,
            &used_closes,
        );

        if let Some(close_idx) = close_idx {
            let close_pos = close_brackets[close_idx];

            // Check for `](` after the close bracket
            let after_close = (close_pos + 1) as usize;
            if after_close < text.len() && text[after_close] == b'(' {
                // Try to parse link destination
                if let Some((url_start, url_end, title_start, title_end, end)) =
                    parse_link_destination(text, after_close + 1)
                {
                    links.push(Link {
                        start: if is_image { open_pos - 1 } else { open_pos },
                        text_end: close_pos,
                        url_start: url_start as u32,
                        url_end: url_end as u32,
                        title_start: title_start.map(|s| s as u32),
                        title_end: title_end.map(|e| e as u32),
                        end: end as u32,
                        is_image,
                    });
                    used_opens[open_idx] = true;
                    used_closes[close_idx] = true;

                    // For links (not images), deactivate any outer open brackets
                    // that would contain this link (links cannot contain links)
                    if !is_image {
                        for (i, &(pos, _)) in open_brackets.iter().enumerate() {
                            if pos < open_pos && !used_opens[i] {
                                // This outer bracket would contain our link
                                // Check if there's a close bracket after our link
                                // that could match the outer open
                                let has_outer_close = close_brackets.iter()
                                    .enumerate()
                                    .any(|(ci, &cpos)| !used_closes[ci] && cpos > close_pos);
                                if has_outer_close {
                                    // The outer bracket could form a link containing our link
                                    // which is not allowed, so deactivate it
                                    used_opens[i] = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by start position
    links.sort_by_key(|l| l.start);
    links
}

/// Find the matching close bracket for an open bracket, accounting for nesting.
fn find_matching_close(
    open_pos: u32,
    open_brackets: &[(u32, bool)],
    close_brackets: &[u32],
    used_opens: &[bool],
    used_closes: &[bool],
) -> Option<usize> {
    // Count nested brackets to find the matching close
    let mut depth = 1i32;
    let mut close_idx = None;

    // Create a merged, sorted list of bracket positions for nesting calculation
    let mut events: Vec<(u32, bool)> = Vec::new(); // (pos, is_open)

    for (i, &(pos, _)) in open_brackets.iter().enumerate() {
        if !used_opens[i] && pos > open_pos {
            events.push((pos, true));
        }
    }
    for (i, &pos) in close_brackets.iter().enumerate() {
        if !used_closes[i] && pos > open_pos {
            events.push((pos, false));
        }
    }
    events.sort_by_key(|&(pos, _)| pos);

    for (pos, is_open) in events {
        if is_open {
            depth += 1;
        } else {
            depth -= 1;
            if depth == 0 {
                // Found matching close, find its index in close_brackets
                close_idx = close_brackets.iter().position(|&p| p == pos);
                break;
            }
        }
    }

    close_idx
}

/// Parse link destination and optional title.
/// Returns (url_start, url_end, title_start, title_end, end) or None.
fn parse_link_destination(text: &[u8], start: usize) -> Option<(usize, usize, Option<usize>, Option<usize>, usize)> {
    let mut pos = start;
    let len = text.len();

    // Skip leading whitespace
    while pos < len && (text[pos] == b' ' || text[pos] == b'\t') {
        pos += 1;
    }

    if pos >= len {
        return None;
    }

    let (url_start, url_end, mut pos) = if text[pos] == b'<' {
        // Angle-bracketed URL
        pos += 1;
        let url_start = pos;
        while pos < len && text[pos] != b'>' && text[pos] != b'\n' {
            if text[pos] == b'\\' && pos + 1 < len {
                pos += 2; // Skip escaped char
            } else {
                pos += 1;
            }
        }
        if pos >= len || text[pos] != b'>' {
            return None;
        }
        let url_end = pos;
        pos += 1; // Skip '>'
        (url_start, url_end, pos)
    } else {
        // Bare URL - count parentheses
        let url_start = pos;
        let mut paren_depth = 0u32;

        while pos < len {
            let b = text[pos];
            match b {
                b'(' => {
                    paren_depth += 1;
                    if paren_depth > limits::MAX_LINK_PAREN_DEPTH as u32 {
                        return None;
                    }
                    pos += 1;
                }
                b')' => {
                    if paren_depth == 0 {
                        break;
                    }
                    paren_depth -= 1;
                    pos += 1;
                }
                b' ' | b'\t' | b'\n' => break,
                b'\\' if pos + 1 < len => {
                    pos += 2; // Skip escaped char
                }
                _ => pos += 1,
            }
        }
        (url_start, pos, pos)
    };

    // Skip whitespace before title or closing paren
    while pos < len && (text[pos] == b' ' || text[pos] == b'\t' || text[pos] == b'\n') {
        pos += 1;
    }

    if pos >= len {
        return None;
    }

    // Check for title or closing paren
    let (title_start, title_end, end) = if text[pos] == b')' {
        // No title
        (None, None, pos + 1)
    } else if text[pos] == b'"' || text[pos] == b'\'' || text[pos] == b'(' {
        // Title
        let quote = if text[pos] == b'(' { b')' } else { text[pos] };
        pos += 1;
        let title_start = pos;

        while pos < len && text[pos] != quote {
            if text[pos] == b'\\' && pos + 1 < len {
                pos += 2;
            } else if text[pos] == b'\n' {
                // Newlines allowed in title but not multiple blank lines
                pos += 1;
            } else {
                pos += 1;
            }
        }

        if pos >= len || text[pos] != quote {
            return None;
        }
        let title_end = pos;
        pos += 1;

        // Skip whitespace after title
        while pos < len && (text[pos] == b' ' || text[pos] == b'\t') {
            pos += 1;
        }

        if pos >= len || text[pos] != b')' {
            return None;
        }

        (Some(title_start), Some(title_end), pos + 1)
    } else {
        return None;
    };

    Some((url_start, url_end, title_start, title_end, end))
}

/// Find autolinks in text.
pub fn find_autolinks(text: &[u8]) -> Vec<Autolink> {
    let mut autolinks = Vec::new();
    let mut pos = 0;
    let len = text.len();

    while pos < len {
        if text[pos] == b'<' {
            if let Some(autolink) = try_parse_autolink(text, pos) {
                autolinks.push(autolink);
                pos = autolink.end as usize;
                continue;
            }
        }
        pos += 1;
    }

    autolinks
}

/// Try to parse an autolink at the given position.
fn try_parse_autolink(text: &[u8], start: usize) -> Option<Autolink> {
    let len = text.len();
    if start >= len || text[start] != b'<' {
        return None;
    }

    let content_start = start + 1;
    let mut pos = content_start;

    // Find the closing '>'
    while pos < len && text[pos] != b'>' && text[pos] != b' ' && text[pos] != b'\n' {
        pos += 1;
    }

    if pos >= len || text[pos] != b'>' {
        return None;
    }

    let content_end = pos;
    let content = &text[content_start..content_end];

    // Check if it's a valid URL or email
    if is_uri_autolink(content) {
        Some(Autolink {
            start: start as u32,
            end: (pos + 1) as u32,
            content_start: content_start as u32,
            content_end: content_end as u32,
            is_email: false,
        })
    } else if is_email_autolink(content) {
        Some(Autolink {
            start: start as u32,
            end: (pos + 1) as u32,
            content_start: content_start as u32,
            content_end: content_end as u32,
            is_email: true,
        })
    } else {
        None
    }
}

/// Check if content is a valid URI autolink.
/// Per CommonMark: scheme followed by : and non-empty path
/// Scheme: 2-32 chars, starting with letter, followed by letters/digits/+/-/.
fn is_uri_autolink(content: &[u8]) -> bool {
    // Minimum: "ab:x" (4 chars - 2-char scheme + colon + 1 char)
    if content.len() < 4 {
        return false;
    }

    let mut pos = 0;

    // First char must be letter
    if !content[pos].is_ascii_alphabetic() {
        return false;
    }
    pos += 1;

    // Following chars: letters, digits, +, -, . (up to 32 total for scheme)
    while pos < content.len() && pos < 32 {
        let b = content[pos];
        if b == b':' {
            break;
        }
        if !b.is_ascii_alphanumeric() && b != b'+' && b != b'-' && b != b'.' {
            return false;
        }
        pos += 1;
    }

    // Must have found a colon, and scheme must be at least 2 chars
    if pos < 2 || pos >= content.len() || content[pos] != b':' {
        return false;
    }

    // Must have something after the colon
    pos + 1 < content.len()
}

/// Check if content is a valid email autolink.
fn is_email_autolink(content: &[u8]) -> bool {
    // Simple check: must contain @ with text before and after
    if let Some(at_pos) = content.iter().position(|&b| b == b'@') {
        if at_pos > 0 && at_pos < content.len() - 1 {
            // Check for valid email characters
            let local = &content[..at_pos];
            let domain = &content[at_pos + 1..];

            // Local part: alphanumeric and some special chars
            let local_valid = local.iter().all(|&b| {
                b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_' || b == b'+'
            });

            // Domain: alphanumeric, dots, hyphens
            let domain_valid = domain.iter().all(|&b| {
                b.is_ascii_alphanumeric() || b == b'.' || b == b'-'
            }) && domain.contains(&b'.');

            return local_valid && domain_valid;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_link() {
        let text = b"[text](https://example.com)";
        let links = resolve_links(text, &[(0, false)], &[5]);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url_start, 7);
        assert_eq!(links[0].url_end, 26);
        assert!(!links[0].is_image);
    }

    #[test]
    fn test_parse_link_with_title() {
        let text = b"[text](url \"title\")";
        let links = resolve_links(text, &[(0, false)], &[5]);

        assert_eq!(links.len(), 1);
        assert!(links[0].title_start.is_some());
    }

    #[test]
    fn test_parse_image() {
        let text = b"![alt](image.png)";
        let links = resolve_links(text, &[(1, true)], &[5]);

        assert_eq!(links.len(), 1);
        assert!(links[0].is_image);
    }

    #[test]
    fn test_uri_autolink() {
        let text = b"<https://example.com>";
        let autolinks = find_autolinks(text);

        assert_eq!(autolinks.len(), 1);
        assert!(!autolinks[0].is_email);
    }

    #[test]
    fn test_email_autolink() {
        let text = b"<test@example.com>";
        let autolinks = find_autolinks(text);

        assert_eq!(autolinks.len(), 1);
        assert!(autolinks[0].is_email);
    }

    #[test]
    fn test_not_autolink() {
        let text = b"<not valid>";
        let autolinks = find_autolinks(text);

        assert_eq!(autolinks.len(), 0);
    }
}
