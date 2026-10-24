//! GitHub-style block quote callout markers, shared by parsing and rendering.
//!
//! The renderer recognizes markers such as `[!NOTE]` only at the beginning of a
//! block quote paragraph. This module keeps the marker grammar and presentation labels
//! together so block rendering can stay focused on emitting HTML.

use crate::ast::{Node, Paragraph};

/// GitHub-style callout kind shared by rendering and AST inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutKind {
    /// The `[!NOTE]` marker.
    Note,
    /// The `[!TIP]` marker.
    Tip,
    /// The `[!IMPORTANT]` marker.
    Important,
    /// The `[!WARNING]` marker.
    Warning,
    /// The `[!CAUTION]` marker.
    Caution,
}

impl CalloutKind {
    /// Recognizes a marker name without ASCII case sensitivity.
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("NOTE") {
            Some(Self::Note)
        } else if name.eq_ignore_ascii_case("TIP") {
            Some(Self::Tip)
        } else if name.eq_ignore_ascii_case("IMPORTANT") {
            Some(Self::Important)
        } else if name.eq_ignore_ascii_case("WARNING") {
            Some(Self::Warning)
        } else if name.eq_ignore_ascii_case("CAUTION") {
            Some(Self::Caution)
        } else {
            None
        }
    }

    /// Parses a marker prefix and returns its trimmed remainder.
    pub fn parse_marker(value: &str) -> Option<(Self, &str)> {
        let marker = value
            .strip_prefix("[!")
            .or_else(|| value.strip_prefix("\\[!"))?;
        let end = marker.find(']')?;
        // Allocation-free: the previous `to_ascii_uppercase().as_str()`
        // path allocated a fresh `String` for every `[!FOO]`-prefixed
        // text run that reached this branch. `eq_ignore_ascii_case`
        // compares the trimmed slice in place against each known label.
        let name = marker[..end].trim();
        let kind = Self::from_name(name)?;

        Some((
            kind,
            marker[end + 1..].trim_start_matches(char::is_whitespace),
        ))
    }

    /// CSS class suffix used by the built-in renderers.
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Important => "important",
            Self::Warning => "warning",
            Self::Caution => "caution",
        }
    }

    /// English presentation label used by renderers and text extraction.
    pub fn label(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Tip => "Tip",
            Self::Important => "Important",
            Self::Warning => "Warning",
            Self::Caution => "Caution",
        }
    }
}

/// A source-level predictor for lazy continuation before the quote has an AST.
/// The final figure decision uses `detect_callout` on parsed children.
pub fn source_starts_with_callout(source: &str) -> bool {
    if CalloutKind::parse_marker(source).is_some() {
        return true;
    }
    let decoded = if let Some(rest) = source.strip_prefix("&#") {
        let Some(end) = rest.find(';') else {
            return false;
        };
        let entity = &rest[..end];
        let value = if let Some(hex) = entity.strip_prefix(['x', 'X']) {
            u32::from_str_radix(hex, 16).ok()
        } else {
            entity.parse::<u32>().ok()
        };
        (value == Some(u32::from(b'['))).then_some(&rest[end + 1..])
    } else {
        source
            .strip_prefix("&lbrack;")
            .or_else(|| source.strip_prefix("&lsqb;"))
    };
    let Some(after_open) = decoded.and_then(|rest| rest.strip_prefix('!')) else {
        return false;
    };
    after_open
        .find(']')
        .and_then(|end| CalloutKind::from_name(after_open[..end].trim()))
        .is_some()
}

/// Detect the marker on parsed inline children so entities, links, headings,
/// reference definitions and code have the same meaning in parser and renderer.
pub fn detect_callout(paragraph: &Paragraph<'_>) -> Option<(CalloutKind, usize)> {
    let Node::Text(first_text) = paragraph.children.first()? else {
        return None;
    };
    if first_text.value.as_bytes().first() != Some(&b'[') {
        return None;
    }

    if let Some((kind, remainder)) = CalloutKind::parse_marker(first_text.value) {
        let consumed = first_text.value.len().saturating_sub(remainder.len());
        return Some((kind, consumed));
    }

    // Failed link parsing can split the marker into adjacent Text nodes.
    // IMPORTANT is the longest accepted name, so this stack buffer suffices.
    let mut name = [0u8; 9];
    let mut name_len = 0usize;
    let mut consumed = 0usize;
    let mut state = 0u8;
    let mut trailing_name_whitespace = false;

    for child in &paragraph.children {
        let Node::Text(text) = child else {
            return None;
        };

        for ch in text.value.chars() {
            consumed += ch.len_utf8();
            match state {
                0 if ch == '[' => state = 1,
                1 if ch == '!' => state = 2,
                0 | 1 => return None,
                _ if ch == ']' => {
                    let name = std::str::from_utf8(&name[..name_len]).ok()?;
                    return Some((CalloutKind::from_name(name)?, consumed));
                }
                _ if ch.is_whitespace() => {
                    trailing_name_whitespace |= name_len != 0;
                }
                _ => {
                    if trailing_name_whitespace || !ch.is_ascii() || name_len == name.len() {
                        return None;
                    }
                    name[name_len] = ch as u8;
                    name_len += 1;
                }
            }
        }
    }

    None
}

/// A recognized callout borrowed from a block quote. Recognition is identical
/// to the HTML/JSX renderers when their `callouts` option is enabled.
#[derive(Debug)]
pub struct Callout<'tree, 'a> {
    /// Recognized marker kind.
    pub kind: CalloutKind,
    /// Original source range of the marker, excluding following whitespace.
    /// For multiline markers this bounding range includes container prefixes.
    pub marker_span: crate::ast::Span,
    /// Parsed body with the structural marker and leading whitespace excluded.
    pub body: CalloutBody<'tree, 'a>,
}

/// The body of a callout without cloning or modifying its AST.
#[derive(Debug)]
pub struct CalloutBody<'tree, 'a> {
    /// Remaining children of the marker paragraph. The first Text child, if
    /// present, starts at `text_offset`; subsequent children are complete.
    pub inlines: &'tree [Node<'a>],
    /// UTF-8 byte offset into the first Text child's decoded `value`.
    /// Zero when the first child is not Text or there are no inline children.
    pub text_offset: usize,
    /// Complete block children following the marker paragraph.
    pub blocks: &'tree [Node<'a>],
}

impl<'a> crate::ast::BlockQuote<'a> {
    /// Inspects a callout without rendering or allocating. Pass the original
    /// source used to parse this tree so marker ranges include authored
    /// entities and escapes. Ordinary quotes return `None`.
    pub fn callout<'tree>(&'tree self, source: &str) -> Option<Callout<'tree, 'a>> {
        let Node::Paragraph(paragraph) = self.children.first()? else {
            return None;
        };
        let (kind, mut remaining) = detect_callout(paragraph)?;
        let Node::Text(first) = paragraph.children.first()? else {
            return None;
        };
        let mut marker_end = None;
        for node in &paragraph.children {
            let Node::Text(text) = node else {
                break;
            };
            if let Some(close) = text.value.find(']') {
                marker_end = decoded_boundary(text, close + 1, source);
                break;
            }
        }
        let mut index = 0;
        let mut text_offset = 0;
        for node in &paragraph.children {
            let Node::Text(text) = node else {
                break;
            };
            let skip = remaining.min(text.value.len());
            remaining -= skip;
            let rest = text.value.get(skip..)?;
            let trimmed = rest.trim_start();
            if !trimmed.is_empty() {
                text_offset = text.value.len() - trimmed.len();
                break;
            }
            index += 1;
        }
        Some(Callout {
            kind,
            marker_span: crate::ast::Span::new(first.span.start, marker_end?),
            body: CalloutBody {
                inlines: &paragraph.children[index..],
                text_offset,
                blocks: &self.children[1..],
            },
        })
    }
}

// Mapping a prefix of an already parsed Text node is decoding, not another
// Markdown recognition policy. Usually the source slice is exactly `value`;
// entities and backslash escapes are isolated Text nodes by the inline parser.
fn decoded_boundary(text: &crate::ast::Text<'_>, offset: usize, source: &str) -> Option<u32> {
    let raw = source.get(text.span.start as usize..text.span.end as usize)?;
    if offset == text.value.len() {
        return Some(text.span.end);
    }
    if offset == 0 {
        return Some(text.span.start);
    }
    if raw.starts_with(text.value.get(..offset)?) {
        return Some(text.span.start + offset as u32);
    }
    // A coalesced Text run can span container prefixes and CRLF. An
    // entity/escape is emitted as its own Text node (handled above); in a
    // partial run the recognized marker's `]` is the first literal closer.
    raw.find(']').map(|end| text.span.start + end as u32 + 1)
}
