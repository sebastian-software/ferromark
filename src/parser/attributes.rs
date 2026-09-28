//! Shared ID and class grammar for opt-in Markdown attributes.

use crate::allocator::{Box, Vec};
use crate::ast::ElementAttributes;
use memchr::memchr3;

use super::Parser;

/// The common colon-prefixed caption/metadata line shape. Attachment points
/// decide whether an empty caption is meaningful and which element owns it.
pub(super) struct CaptionLine<'a> {
    pub content: &'a str,
    pub content_offset: usize,
    pub tokens: Option<&'a str>,
}

pub(super) fn caption_line(line: &str, allow_empty_caption: bool) -> Option<CaptionLine<'_>> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let after_colon = trimmed.strip_prefix(':')?;
    if !after_colon.starts_with([' ', '\t']) {
        return None;
    }
    let content = after_colon.trim_start_matches([' ', '\t']);
    let content_offset = line.len() - content.len();
    let content = content.trim_end_matches([' ', '\t']);
    if content.is_empty() {
        return None;
    }
    if let Some(without_close) = content.strip_suffix('}')
        && let Some(open) = without_close.rfind('{')
        && (open == 0 || without_close[..open].ends_with([' ', '\t']))
    {
        let caption = without_close[..open].trim_end_matches([' ', '\t']);
        let tokens = &without_close[open + 1..];
        scan_id_classes(tokens, |_, _| ())?;
        if !allow_empty_caption && caption.is_empty() {
            return None;
        }
        return Some(CaptionLine {
            content: caption,
            content_offset,
            tokens: Some(tokens),
        });
    }
    if content.contains(['{', '}']) {
        return None;
    }
    Some(CaptionLine {
        content,
        content_offset,
        tokens: None,
    })
}

/// Validate ID/class tokens without allocating. The same scanner builds AST
/// values and checks caption-line lookahead, so the two cannot disagree.
fn scan_id_classes<'a>(tokens: &'a str, mut on_token: impl FnMut(&'a str, bool)) -> Option<()> {
    let mut seen_id = false;
    let mut found = false;
    for token in tokens.split_whitespace() {
        let (name, is_id) = if let Some(name) = token.strip_prefix('#') {
            (name, true)
        } else {
            (token.strip_prefix('.')?, false)
        };
        if name.is_empty()
            || name.chars().any(|ch| {
                ch.is_control() || matches!(ch, '"' | '\'' | '<' | '>' | '=' | '{' | '}' | '\\')
            })
            || (is_id && seen_id)
        {
            return None;
        }
        seen_id |= is_id;
        found = true;
        on_token(name, is_id);
    }
    found.then_some(())
}

impl<'a> Parser<'a> {
    /// Parse a nonempty brace body. Invalid tokens and repeated IDs make the
    /// entire block ordinary text rather than silently discarding metadata.
    pub(super) fn parse_id_classes(
        &self,
        tokens: &'a str,
    ) -> Option<(Option<&'a str>, Vec<'a, &'a str>)> {
        let mut id = None;
        let mut classes = self.allocator.new_vec();
        scan_id_classes(tokens, |name, is_id| {
            if is_id {
                id = Some(name);
            } else {
                classes.push(name);
            }
        })?;
        Some((id, classes))
    }

    /// A brace suffix starts at `end`, directly after an inline image.
    pub(super) fn parse_image_id_classes(
        &self,
        content: &'a str,
        end: usize,
    ) -> (Option<Box<'a, ElementAttributes<'a>>>, usize) {
        if !self.options.image_attributes || content.as_bytes().get(end) != Some(&b'{') {
            return (None, end);
        }
        let rest = &content.as_bytes()[end + 1..];
        // Stop at another opener or a line ending. Repeated malformed
        // suffixes then scan disjoint regions instead of every remaining byte.
        let Some(offset) = memchr3(b'}', b'{', b'\n', rest) else {
            return (None, end);
        };
        if rest[offset] != b'}' || rest[..offset].contains(&b'\r') {
            return (None, end);
        }
        let close = end + 1 + offset;
        let Some((id, classes)) = self.parse_id_classes(&content[end + 1..close]) else {
            return (None, end);
        };
        (
            Some(self.allocator.boxed(ElementAttributes { id, classes })),
            close + 1,
        )
    }
}
