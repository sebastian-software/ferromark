//! Link and image parsing.
//!
//! Handles:
//! - Inline links: `[text](url "title")`
//! - Images: `![alt](url "title")`
//! - Autolinks: `<https://example.com>` and `<email@example.com>`

use crate::limits;
use crate::link_ref::{LinkRefStore, normalize_label_into};
use memchr::memchr;

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

/// A resolved reference-style link or image.
#[derive(Debug, Clone)]
pub struct RefLink {
    /// Start of the opening bracket (or `!` for images).
    pub start: u32,
    /// End of the link text (position of `]`).
    pub text_end: u32,
    /// End of the entire reference (after the closing `]` of label, or close of text for shortcut).
    pub end: u32,
    /// Whether this is an image.
    pub is_image: bool,
    /// Index into the link reference store.
    pub def_index: usize,
}

/// Parse links from text, given bracket positions.
/// Returns list of resolved links.
#[allow(dead_code)]
pub fn resolve_links(
    text: &[u8],
    open_brackets: &[(u32, bool)], // (position, is_image)
    close_brackets: &[u32],
) -> Vec<Link> {
    let mut out_links = Vec::new();
    let mut formed_opens = Vec::new();
    let mut inactive_opens = Vec::new();
    let mut used_closes = Vec::new();
    resolve_links_into(
        text,
        open_brackets,
        close_brackets,
        &mut out_links,
        &mut formed_opens,
        &mut inactive_opens,
        &mut used_closes,
    );
    out_links
}

/// Parse links from text, given bracket positions, reusing caller-owned buffers.
pub fn resolve_links_into(
    text: &[u8],
    open_brackets: &[(u32, bool)], // (position, is_image)
    close_brackets: &[u32],
    out_links: &mut Vec<Link>,
    formed_opens: &mut Vec<bool>,
    inactive_opens: &mut Vec<bool>,
    used_closes: &mut Vec<bool>,
) {
    out_links.clear();
    // Track opens that have formed links (consumed with their close)
    formed_opens.clear();
    formed_opens.resize(open_brackets.len(), false);
    // Track opens that are deactivated (can't form links, but still count for depth)
    inactive_opens.clear();
    inactive_opens.resize(open_brackets.len(), false);
    used_closes.clear();
    used_closes.resize(close_brackets.len(), false);

    // Process open brackets from right to left (innermost first)
    for (open_idx, &(open_pos, is_image)) in open_brackets.iter().enumerate().rev() {
        if formed_opens[open_idx] || inactive_opens[open_idx] {
            continue;
        }

        // Find the matching close bracket (accounting for nested brackets)
        let close_idx = find_matching_close(
            open_pos,
            open_brackets,
            close_brackets,
            formed_opens,
            used_closes,
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
                    out_links.push(Link {
                        start: if is_image { open_pos - 1 } else { open_pos },
                        text_end: close_pos,
                        url_start: url_start as u32,
                        url_end: url_end as u32,
                        title_start: title_start.map(|s| s as u32),
                        title_end: title_end.map(|e| e as u32),
                        end: end as u32,
                        is_image,
                    });
                    formed_opens[open_idx] = true;
                    used_closes[close_idx] = true;

                    // For links (not images), deactivate any outer LINK open brackets
                    // that would contain this link (links cannot contain links,
                    // but images CAN contain links)
                    if !is_image {
                        for (i, &(pos, outer_is_image)) in open_brackets.iter().enumerate() {
                            // Only deactivate outer LINK brackets, not image brackets
                            if pos < open_pos
                                && !formed_opens[i]
                                && !inactive_opens[i]
                                && !outer_is_image
                            {
                                // This outer link bracket would contain our link
                                // Check if there's a close bracket after our link
                                // that could match the outer open
                                let has_outer_close = close_brackets
                                    .iter()
                                    .enumerate()
                                    .any(|(ci, &cpos)| !used_closes[ci] && cpos > close_pos);
                                if has_outer_close {
                                    // The outer bracket could form a link containing our link
                                    // which is not allowed, so mark it inactive
                                    // (but it still contributes to bracket depth)
                                    inactive_opens[i] = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by start position
    out_links.sort_by_key(|l| l.start);
}

/// Resolve reference-style links/images using link reference definitions.
#[allow(clippy::too_many_arguments)]
pub fn resolve_reference_links_into(
    text: &[u8],
    open_brackets: &[(u32, bool)],
    close_brackets: &[u32],
    inline_links: &[Link],
    defs: &LinkRefStore,
    out_links: &mut Vec<RefLink>,
    label_buf: &mut String,
    formed_opens: &mut Vec<bool>,
    used_closes: &mut Vec<bool>,
    occupied: &mut Vec<(u32, u32)>,
) {
    out_links.clear();
    label_buf.clear();

    formed_opens.clear();
    formed_opens.resize(open_brackets.len(), false);
    used_closes.clear();
    used_closes.resize(close_brackets.len(), false);
    occupied.clear();
    occupied.extend(
        inline_links
            .iter()
            .filter(|l| !l.is_image)
            .map(|l| (l.start, l.end)),
    );
    // Mark opens/closes used by inline links
    for link in inline_links {
        let open_pos = if link.is_image {
            link.start + 1
        } else {
            link.start
        };
        if let Some(idx) = find_open_idx(open_brackets, open_pos) {
            formed_opens[idx] = true;
        }
        if let Some(idx) = find_close_idx(close_brackets, link.text_end) {
            used_closes[idx] = true;
        }
    }

    // Process open brackets from left to right
    let mut nested_label_buf = String::new();
    for open_idx in 0..open_brackets.len() {
        if formed_opens[open_idx] {
            continue;
        }
        let (open_pos, is_image) = open_brackets[open_idx];

        let close_idx = find_matching_close(
            open_pos,
            open_brackets,
            close_brackets,
            formed_opens,
            used_closes,
        );

        let Some(close_idx) = close_idx else { continue };
        let close_pos = close_brackets[close_idx];

        // Determine reference label
        let mut end = close_pos + 1;
        let label_start = (open_pos + 1) as usize;
        let label_end = close_pos as usize;
        let mut label_bytes = &text[label_start..label_end];

        let mut ref_label: Option<(usize, usize, usize)> = None;
        if let Some((ref_start, ref_end, ref_close_pos)) =
            parse_ref_label_immediate(text, close_pos as usize + 1)
        {
            // Full or collapsed reference: [label][ref] or [label][]
            if ref_start == ref_end {
                // Collapsed: use link text as label
            } else {
                label_bytes = &text[ref_start..ref_end];
            }
            end = (ref_close_pos + 1) as u32;
            ref_label = Some((ref_start, ref_end, ref_close_pos));
        }

        normalize_label_into(label_bytes, label_buf);
        if label_buf.is_empty() {
            continue;
        }
        let Some(def_index) = defs.get_index(label_buf) else {
            continue;
        };

        // Links cannot contain links (but can contain images)
        if contains_link(occupied, open_pos, close_pos)
            || contains_ref_link_candidate(
                text,
                open_brackets,
                close_brackets,
                defs,
                open_pos,
                close_pos,
                &mut nested_label_buf,
            )
        {
            continue;
        }

        formed_opens[open_idx] = true;
        used_closes[close_idx] = true;

        if let Some((ref_start, _ref_end, ref_close_pos)) = ref_label {
            if let Some(idx) = find_open_idx(open_brackets, (ref_start - 1) as u32) {
                formed_opens[idx] = true;
            }
            if let Some(idx) = find_close_idx(close_brackets, ref_close_pos as u32) {
                used_closes[idx] = true;
            }
        }

        out_links.push(RefLink {
            start: if is_image { open_pos - 1 } else { open_pos },
            text_end: close_pos,
            end,
            is_image,
            def_index,
        });

        occupied.push((open_pos, end));
    }

    out_links.sort_by_key(|l| l.start);
}

fn contains_ref_link_candidate(
    text: &[u8],
    open_brackets: &[(u32, bool)],
    close_brackets: &[u32],
    defs: &LinkRefStore,
    start: u32,
    end: u32,
    label_buf: &mut String,
) -> bool {
    label_buf.clear();
    for &(open_pos, is_image) in open_brackets {
        if open_pos <= start || open_pos >= end || is_image {
            continue;
        }
        let close_pos = close_brackets
            .iter()
            .copied()
            .find(|&c| c > open_pos && c < end);
        let Some(close_pos) = close_pos else { continue };

        let label_start = (open_pos + 1) as usize;
        let label_end = close_pos as usize;
        if label_start >= label_end || label_end > text.len() {
            continue;
        }
        let mut label_bytes = &text[label_start..label_end];

        if let Some((ref_start, ref_end, _ref_close)) =
            parse_ref_label_immediate(text, close_pos as usize + 1)
        {
            if ref_start != ref_end {
                label_bytes = &text[ref_start..ref_end];
            }
        }

        normalize_label_into(label_bytes, label_buf);
        if label_buf.is_empty() {
            continue;
        }
        if defs.get_index(label_buf).is_some() {
            return true;
        }
    }
    false
}

#[inline]
fn find_open_idx(open_brackets: &[(u32, bool)], pos: u32) -> Option<usize> {
    open_brackets.binary_search_by_key(&pos, |(p, _)| *p).ok()
}

#[inline]
fn find_close_idx(close_brackets: &[u32], pos: u32) -> Option<usize> {
    close_brackets.binary_search(&pos).ok()
}

fn contains_link(links: &[(u32, u32)], start: u32, end: u32) -> bool {
    links.iter().any(|&(s, e)| s >= start && e <= end)
}

/// Find the matching close bracket for an open bracket, accounting for nesting.
/// `formed_opens` indicates opens that have formed links (and consumed their close).
/// Inactive opens (deactivated but not formed) still contribute to depth.
fn find_matching_close(
    open_pos: u32,
    open_brackets: &[(u32, bool)],
    close_brackets: &[u32],
    formed_opens: &[bool],
    used_closes: &[bool],
) -> Option<usize> {
    // Count nested brackets to find the matching close.
    // Only skip opens that have formed links (their closes are also consumed).
    // Inactive opens still contribute to depth.
    let mut depth = 1i32;

    let mut open_idx = match open_brackets.binary_search_by_key(&open_pos, |(p, _)| *p) {
        Ok(i) => i + 1,
        Err(i) => i,
    };
    let mut close_idx = match close_brackets.binary_search(&open_pos) {
        Ok(i) => i + 1,
        Err(i) => i,
    };

    loop {
        while open_idx < open_brackets.len() {
            let (pos, _) = open_brackets[open_idx];
            if pos > open_pos && !formed_opens[open_idx] {
                break;
            }
            open_idx += 1;
        }
        while close_idx < close_brackets.len() {
            let pos = close_brackets[close_idx];
            if pos > open_pos && !used_closes[close_idx] {
                break;
            }
            close_idx += 1;
        }

        let next_open = if open_idx < open_brackets.len() {
            open_brackets[open_idx].0
        } else {
            u32::MAX
        };
        let next_close = if close_idx < close_brackets.len() {
            close_brackets[close_idx]
        } else {
            u32::MAX
        };

        if next_open == u32::MAX && next_close == u32::MAX {
            break;
        }

        if next_open <= next_close {
            depth += 1;
            open_idx += 1;
        } else {
            depth -= 1;
            if depth == 0 {
                return Some(close_idx);
            }
            close_idx += 1;
        }
    }

    None
}

fn parse_ref_label_immediate(text: &[u8], mut pos: usize) -> Option<(usize, usize, usize)> {
    let len = text.len();

    if pos >= len || text[pos] != b'[' {
        return None;
    }
    let label_start = pos + 1;
    pos += 1;

    while pos < len {
        match text[pos] {
            b'\\' => {
                if pos + 1 < len {
                    pos += 2;
                } else {
                    return None;
                }
            }
            b'[' => return None,
            b']' => break,
            _ => pos += 1,
        }
    }
    if pos >= len || text[pos] != b']' {
        return None;
    }
    let label_end = pos;
    Some((label_start, label_end, pos))
}

/// Parse link destination and optional title.
/// Returns (url_start, url_end, title_start, title_end, end) or None.
#[allow(clippy::type_complexity)]
fn parse_link_destination(
    text: &[u8],
    start: usize,
) -> Option<(usize, usize, Option<usize>, Option<usize>, usize)> {
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
#[cfg(test)]
pub fn find_autolinks(text: &[u8]) -> Vec<Autolink> {
    let mut autolinks = Vec::new();
    find_autolinks_into(text, &mut autolinks);
    autolinks
}

/// Find autolinks in text, appending results into the provided buffer.
pub fn find_autolinks_into(text: &[u8], out: &mut Vec<Autolink>) {
    out.clear();
    let mut pos = 0;
    while let Some(offset) = memchr::memchr(b'<', &text[pos..]) {
        let idx = pos + offset;
        if let Some(autolink) = try_parse_autolink(text, idx) {
            out.push(autolink);
            pos = autolink.end as usize;
        } else {
            pos = idx + 1;
        }
    }
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
            let domain_valid = domain
                .iter()
                .all(|&b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
                && domain.contains(&b'.');

            return local_valid && domain_valid;
        }
    }
    false
}

// ─── Autolink Literals (GFM extension) ───────────────────────────────────────

/// Kind of autolink literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutolinkLiteralKind {
    /// `http://` or `https://` or `ftp://` URL.
    Url,
    /// `www.` link (needs `http://` prepended in href).
    Www,
    /// Email address.
    Email,
}

/// A detected autolink literal (bare URL, www link, or email).
#[derive(Debug, Clone, Copy)]
pub struct AutolinkLiteral {
    pub start: u32,
    pub end: u32,
    pub kind: AutolinkLiteralKind,
}

/// Find autolink literals in text, avoiding code spans, HTML ranges, existing autolinks, and link ranges.
#[allow(clippy::too_many_arguments)]
pub fn find_autolink_literals_into(
    text: &[u8],
    code_spans: &[(u32, u32)],
    html_ranges: &[(u32, u32)],
    autolink_ranges: &[(u32, u32)],
    link_ranges: &[(u32, u32)],
    out: &mut Vec<AutolinkLiteral>,
) {
    out.clear();
    let len = text.len();
    if len < 4 {
        return;
    }

    // Scan for @ (email), : (URL protocol), and w/W (www) using memchr jumps
    // instead of checking every byte position.

    // Pass 1: Email autolinks — scan for '@'
    {
        let mut pos = 1; // @ at pos 0 can't have a local part
        while pos < len {
            if let Some(offset) = memchr(b'@', &text[pos..]) {
                let at_pos = pos + offset;
                if !in_any_range(at_pos as u32, code_spans)
                    && !in_any_range(at_pos as u32, html_ranges)
                    && !in_any_range(at_pos as u32, autolink_ranges)
                    && !in_any_range(at_pos as u32, link_ranges)
                {
                    if let Some(al) = try_email_autolink(text, at_pos) {
                        if !overlaps_any_range(al.start, al.end, code_spans)
                            && !overlaps_any_range(al.start, al.end, html_ranges)
                            && !overlaps_any_range(al.start, al.end, autolink_ranges)
                            && !overlaps_any_range(al.start, al.end, link_ranges)
                        {
                            pos = al.end as usize;
                            out.push(al);
                            continue;
                        }
                    }
                }
                pos = at_pos + 1;
            } else {
                break;
            }
        }
    }

    // Pass 2: URL autolinks — scan for ':'
    {
        let mut pos = 0;
        while pos < len {
            if let Some(offset) = memchr(b':', &text[pos..]) {
                let colon = pos + offset;
                // Need at least "X://" where X is the protocol prefix
                if colon >= 1
                    && colon + 2 < len
                    && text[colon + 1] == b'/'
                    && text[colon + 2] == b'/'
                {
                    // Walk backwards to find protocol start (http, https, ftp)
                    let proto_start = find_protocol_start(text, colon);
                    if let Some(start) = proto_start {
                        if !in_any_range(start as u32, code_spans)
                            && !in_any_range(start as u32, html_ranges)
                            && !in_any_range(start as u32, autolink_ranges)
                            && !in_any_range(start as u32, link_ranges)
                            && is_valid_autolink_preceding(text, start)
                        {
                            if let Some(al) = try_url_autolink(text, start) {
                                if !overlaps_any_range(al.start, al.end, code_spans)
                                    && !overlaps_any_range(al.start, al.end, html_ranges)
                                    && !overlaps_any_range(al.start, al.end, autolink_ranges)
                                    && !overlaps_any_range(al.start, al.end, link_ranges)
                                {
                                    pos = al.end as usize;
                                    out.push(al);
                                    continue;
                                }
                            }
                        }
                    }
                }
                pos = colon + 1;
            } else {
                break;
            }
        }
    }

    // Pass 3: WWW autolinks — scan for '.'
    {
        let mut pos = 3; // Need at least "www" before the dot
        while pos < len {
            if let Some(offset) = memchr(b'.', &text[pos..]) {
                let dot = pos + offset;
                if dot >= 3
                    && (text[dot - 3] | 0x20) == b'w'
                    && (text[dot - 2] | 0x20) == b'w'
                    && (text[dot - 1] | 0x20) == b'w'
                    && is_valid_autolink_preceding(text, dot - 3)
                {
                    let start = dot - 3;
                    if !in_any_range(start as u32, code_spans)
                        && !in_any_range(start as u32, html_ranges)
                        && !in_any_range(start as u32, autolink_ranges)
                        && !in_any_range(start as u32, link_ranges)
                    {
                        if let Some(al) = try_www_autolink(text, start) {
                            if !overlaps_any_range(al.start, al.end, code_spans)
                                && !overlaps_any_range(al.start, al.end, html_ranges)
                                && !overlaps_any_range(al.start, al.end, autolink_ranges)
                                && !overlaps_any_range(al.start, al.end, link_ranges)
                            {
                                pos = al.end as usize;
                                out.push(al);
                                continue;
                            }
                        }
                    }
                }
                pos = dot + 1;
            } else {
                break;
            }
        }
    }

    // Sort by start position since we found them in separate passes
    if out.len() > 1 {
        out.sort_unstable_by_key(|al| al.start);
    }
}

/// Find the start of a URL protocol (http, https, ftp) ending at the colon position.
fn find_protocol_start(text: &[u8], colon: usize) -> Option<usize> {
    // Check for https:// (most common)
    if colon >= 5 && text[colon - 5..colon].eq_ignore_ascii_case(b"https") {
        return Some(colon - 5);
    }
    // Check for http://
    if colon >= 4 && text[colon - 4..colon].eq_ignore_ascii_case(b"http") {
        return Some(colon - 4);
    }
    // Check for ftp://
    if colon >= 3 && text[colon - 3..colon].eq_ignore_ascii_case(b"ftp") {
        return Some(colon - 3);
    }
    None
}

/// Check if the character preceding `pos` is valid for an autolink literal start.
/// Must be: start of text, whitespace, or one of `*`, `_`, `~`, `(`
fn is_valid_autolink_preceding(text: &[u8], pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    let prev = text[pos - 1];
    matches!(
        prev,
        b' ' | b'\t' | b'\n' | b'\r' | b'*' | b'_' | b'~' | b'(' | b'"' | b'\'' | b';'
    ) || prev == b'\x0c'
}

/// Try to parse a URL autolink (http://, https://, ftp://).
fn try_url_autolink(text: &[u8], start: usize) -> Option<AutolinkLiteral> {
    let len = text.len();
    let rest = &text[start..];

    // Match protocol
    let protocol_len = if rest.len() >= 8 && rest[..8].eq_ignore_ascii_case(b"https://") {
        8
    } else if rest.len() >= 7 && rest[..7].eq_ignore_ascii_case(b"http://") {
        7
    } else if rest.len() >= 6 && rest[..6].eq_ignore_ascii_case(b"ftp://") {
        6
    } else {
        return None;
    };

    // Must have valid domain after protocol
    let domain_start = start + protocol_len;
    if domain_start >= len {
        return None;
    }

    // Validate domain: at least one valid char
    if !text[domain_start].is_ascii_alphanumeric() {
        return None;
    }

    // Extend URL until whitespace or < or control char
    let mut end = domain_start;
    while end < len {
        let b = text[end];
        if b == b'<' || b <= b' ' {
            break;
        }
        end += 1;
    }

    // Apply trailing trimming
    let end = trim_autolink_trailing(text, start, end);

    // Must have something after protocol
    if end <= domain_start {
        return None;
    }

    Some(AutolinkLiteral {
        start: start as u32,
        end: end as u32,
        kind: AutolinkLiteralKind::Url,
    })
}

/// Try to parse a WWW autolink (www.).
fn try_www_autolink(text: &[u8], start: usize) -> Option<AutolinkLiteral> {
    let len = text.len();
    let rest = &text[start..];

    // Must start with "www."
    if rest.len() < 4 || !rest[..4].eq_ignore_ascii_case(b"www.") {
        return None;
    }

    // Must have valid domain char after www.
    if start + 4 >= len || !text[start + 4].is_ascii_alphanumeric() {
        return None;
    }

    // Check domain validity: need at least one more dot
    let mut dot_count = 0u32;
    let mut pos = start + 4;
    while pos < len {
        let b = text[pos];
        if b == b'<' || b <= b' ' {
            break;
        }
        if b == b'.' {
            dot_count += 1;
        }
        pos += 1;
    }

    if dot_count == 0 {
        return None;
    }

    // Validate no underscores in last two domain segments
    let url_end = pos;
    let domain_text = &text[start..url_end];
    // Find where path starts (first / after domain)
    let path_start = domain_text
        .iter()
        .position(|&b| b == b'/')
        .unwrap_or(domain_text.len());
    let domain_part = &domain_text[..path_start];
    if has_underscore_in_last_two_segments(domain_part) {
        return None;
    }

    // Apply trailing trimming
    let end = trim_autolink_trailing(text, start, url_end);

    if end <= start + 4 {
        return None;
    }

    Some(AutolinkLiteral {
        start: start as u32,
        end: end as u32,
        kind: AutolinkLiteralKind::Www,
    })
}

/// Try to parse an email autolink at the `@` position.
fn try_email_autolink(text: &[u8], at_pos: usize) -> Option<AutolinkLiteral> {
    // Find local part (backwards from @)
    let mut local_start = at_pos;
    while local_start > 0 {
        let b = text[local_start - 1];
        if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'+' | b'-' | b'_') {
            local_start -= 1;
        } else {
            break;
        }
    }

    // Must have at least one char in local part
    if local_start == at_pos {
        return None;
    }

    // Context: character before local part must not be alphanumeric
    // (it must be a word boundary)
    if local_start > 0 {
        let prev = text[local_start - 1];
        if prev.is_ascii_alphanumeric() {
            return None; // Not a word boundary
        }
    }

    // Find domain part (forward from @)
    let len = text.len();
    let mut domain_end = at_pos + 1;
    let mut dot_count = 0u32;

    while domain_end < len {
        let b = text[domain_end];
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'.' {
            if b == b'.' {
                dot_count += 1;
            }
            domain_end += 1;
        } else {
            break;
        }
    }

    // Must have at least one dot in domain
    if dot_count == 0 {
        return None;
    }

    // Must have at least one char after @
    if domain_end <= at_pos + 1 {
        return None;
    }

    // Domain must not end with hyphen or dot
    let last = text[domain_end - 1];
    if last == b'-' || last == b'.' {
        // Trim trailing dots/hyphens
        while domain_end > at_pos + 1 && matches!(text[domain_end - 1], b'-' | b'.') {
            domain_end -= 1;
        }
    }

    // Recount dots after trimming
    let domain_slice = &text[at_pos + 1..domain_end];
    let has_dot = domain_slice.contains(&b'.');
    if !has_dot {
        return None;
    }

    // Last char of domain must be alphanumeric
    if domain_end <= at_pos + 1 || !text[domain_end - 1].is_ascii_alphanumeric() {
        return None;
    }

    // Check no underscores in last two domain segments
    if has_underscore_in_last_two_segments(domain_slice) {
        return None;
    }

    Some(AutolinkLiteral {
        start: local_start as u32,
        end: domain_end as u32,
        kind: AutolinkLiteralKind::Email,
    })
}

/// Trim trailing punctuation from an autolink per GFM rules.
fn trim_autolink_trailing(text: &[u8], start: usize, mut end: usize) -> usize {
    loop {
        if end <= start {
            return end;
        }
        let last = text[end - 1];

        // Rule 1: Strip trailing punctuation chars
        if matches!(
            last,
            b'?' | b'!' | b'.' | b',' | b':' | b'*' | b'_' | b'~' | b'\'' | b'"'
        ) {
            end -= 1;
            continue;
        }

        // Rule 2: Parenthesis balancing
        if last == b')' {
            let url = &text[start..end];
            let open_count = url.iter().filter(|&&b| b == b'(').count();
            let close_count = url.iter().filter(|&&b| b == b')').count();
            if close_count > open_count {
                end -= 1;
                continue;
            }
        }

        // Rule 3: Entity check - if ends with `;` preceded by `&<alphanum>+`
        if last == b';' {
            if let Some(new_end) = trim_trailing_entity(text, start, end) {
                end = new_end;
                continue;
            }
        }

        break;
    }
    end
}

/// If the URL ends with `&<alphanum>+;`, trim the entity reference.
fn trim_trailing_entity(text: &[u8], start: usize, end: usize) -> Option<usize> {
    // Look backwards from end-1 (the ';') to find '&'
    let mut i = end - 2; // skip the ';'
    while i > start {
        let b = text[i];
        if b == b'&' {
            // Found entity: &...;
            // Verify chars between & and ; are alphanumeric
            let entity_body = &text[i + 1..end - 1];
            if !entity_body.is_empty() && entity_body.iter().all(|b| b.is_ascii_alphanumeric()) {
                return Some(i);
            }
            return None;
        }
        if !b.is_ascii_alphanumeric() {
            return None;
        }
        i -= 1;
    }
    None
}

/// Check if underscores appear in the last two domain segments.
fn has_underscore_in_last_two_segments(domain: &[u8]) -> bool {
    // Split by dots, check last two segments
    let mut segments: [&[u8]; 2] = [&[], &[]];
    let mut seg_start = 0usize;
    let mut seg_idx = 0usize;

    for (i, &b) in domain.iter().enumerate() {
        if b == b'.' {
            if seg_idx >= 2 {
                segments[0] = segments[1];
                segments[1] = &domain[seg_start..i];
            } else {
                segments[seg_idx] = &domain[seg_start..i];
                seg_idx += 1;
            }
            seg_start = i + 1;
        }
    }
    // Last segment
    let last_seg = &domain[seg_start..];
    if seg_idx >= 2 {
        segments[0] = segments[1];
        segments[1] = last_seg;
    } else if seg_idx == 1 {
        segments[1] = last_seg;
    } else {
        segments[0] = last_seg;
    }

    // Check both segments for underscore
    segments.iter().any(|seg| seg.contains(&b'_'))
}

#[inline]
fn in_any_range(pos: u32, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|&(s, e)| pos >= s && pos < e)
}

#[inline]
fn overlaps_any_range(start: u32, end: u32, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|&(s, e)| start < e && end > s)
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
