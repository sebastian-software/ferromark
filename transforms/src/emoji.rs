//! Optional conversion of versioned Gemoji shortcodes into Unicode emoji.

use std::ops::Range;

use ferromark::ast::{Document, Span};

use crate::emoji_data::SHORTCODES;
use crate::prose::{NodeList, is_in_raw_html, visit_document_prose};
use crate::{BoxError, TransformContext, TransformPass, text_runs};

/// Replaces known `gemoji@8.1.0` shortcode aliases in prose with Unicode emoji.
///
/// Names are case-sensitive, unknown names remain unchanged, and `:+1:` is
/// supported. Code, math, raw HTML, MDX expressions, image metadata, URL text,
/// link destinations and titles stay unchanged. Ordinary link labels are
/// transformed, matching the selected upstream behavior. The bundled map is
/// generated from the MIT-licensed `gemoji@8.1.0` artifact; construction and
/// use are opt-in, so default parser and renderer calls do not load or scan it.
#[derive(Debug, Default, Clone, Copy)]
pub struct EmojiShortcodesPass;

impl EmojiShortcodesPass {
    /// Creates the optional emoji shortcode pass.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TransformPass for EmojiShortcodesPass {
    fn name(&self) -> &'static str {
        "emoji-shortcodes"
    }

    fn apply<'arena>(
        &mut self,
        document: &mut Document<'arena>,
        context: &TransformContext<'arena>,
    ) -> Result<(), BoxError> {
        visit_document_prose(document, context, true, &mut transform_text_runs)
    }
}

struct EmojiMatch {
    node_range: Range<usize>,
    byte_range: Range<usize>,
    replacement: &'static str,
}

fn transform_text_runs<'arena>(
    nodes: &mut NodeList<'arena>,
    context: &TransformContext<'arena>,
    raw_html_spans: &[Span],
) -> Result<(), BoxError> {
    let mut matches = Vec::new();
    for run in text_runs(nodes) {
        if is_in_raw_html(&run, raw_html_spans) {
            continue;
        }
        let value = run.value();
        // Fast path: skip URL protection and shortcode scanning completely
        // when the run contains no colon at all.
        if !value.contains(':') {
            continue;
        }
        let protected = run.protected_url_ranges(context);
        for (byte_range, replacement) in find_shortcodes(&value, &protected) {
            matches.push(EmojiMatch {
                node_range: run.node_range(),
                byte_range,
                replacement,
            });
        }
    }

    let mut active_run = None;
    let mut boundary = 0;
    for matched in matches.into_iter().rev() {
        let start = matched.node_range.start;
        if active_run != Some(start) {
            active_run = Some(start);
            boundary = matched.node_range.end;
        }
        let (_, inserted_index) = TransformContext::replace_text_range_with_node(
            nodes,
            start..boundary,
            matched.byte_range,
            |span| {
                Some(ferromark::ast::Node::Text(
                    context.replacement_text(matched.replacement, span),
                ))
            },
        )?;
        boundary = inserted_index;
    }

    Ok(())
}

fn find_shortcodes(value: &str, protected: &[Range<usize>]) -> Vec<(Range<usize>, &'static str)> {
    let bytes = value.as_bytes();
    let mut matches = Vec::new();
    let mut cursor = 0;
    let mut protected_idx = 0;

    while let Some(rel) = value[cursor..].find(':') {
        cursor += rel;
        while protected_idx < protected.len() && protected[protected_idx].end <= cursor {
            protected_idx += 1;
        }
        if let Some(range) = protected.get(protected_idx)
            && range.start <= cursor
            && cursor < range.end
        {
            cursor = range.end;
            continue;
        }

        if let Some((end, name)) = parse_shortcode(bytes, cursor) {
            let range = cursor..end;
            let overlaps_protected = protected
                .get(protected_idx)
                .is_some_and(|prot| prot.start < range.end && range.start < prot.end);
            if !overlaps_protected && let Some(emoji) = shortcode_value(name) {
                matches.push((range, emoji));
                cursor = end;
                continue;
            }
        }

        cursor += 1;
    }

    matches
}

fn parse_shortcode(bytes: &[u8], start: usize) -> Option<(usize, &str)> {
    let content_start = start + 1;
    if bytes.get(content_start..content_start + 2) == Some(b"+1")
        && bytes.get(content_start + 2) == Some(&b':')
    {
        return Some((content_start + 3, "+1"));
    }

    let mut cursor = content_start;
    while bytes
        .get(cursor)
        .is_some_and(|byte| is_shortcode_byte(*byte))
    {
        cursor += 1;
    }
    if cursor == content_start || bytes.get(cursor) != Some(&b':') {
        return None;
    }

    // The matched bytes are limited to ASCII shortcode characters.
    let name = std::str::from_utf8(&bytes[content_start..cursor]).ok()?;
    Some((cursor + 1, name))
}

fn is_shortcode_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}

fn shortcode_value(name: &str) -> Option<&'static str> {
    SHORTCODES
        .binary_search_by(|(candidate, _)| candidate.cmp(&name))
        .ok()
        .map(|index| SHORTCODES[index].1)
}

#[cfg(test)]
mod tests {
    use super::{find_shortcodes, shortcode_value};

    #[test]
    fn scans_aliases_case_sensitively_and_keeps_overlapping_unknown_prefixes() {
        let source = "Known :rocket: and :+1:; unknown :not_real:; Case :Smile:; :other:smile:.";
        let matches = find_shortcodes(source, &[]);
        let replacements = matches
            .iter()
            .map(|(range, emoji)| (&source[range.clone()], *emoji))
            .collect::<Vec<_>>();

        assert_eq!(
            replacements,
            vec![(":rocket:", "🚀"), (":+1:", "👍"), (":smile:", "😄")]
        );
        assert_eq!(shortcode_value("Smile"), None);
    }

    #[test]
    fn does_not_match_shortcodes_overlapping_a_protected_url() {
        let source = "https://host/:smile: and :rocket:";
        let url_end = source.find(" and ").unwrap_or(source.len());
        let protected_url = 0..url_end;
        let matches = find_shortcodes(source, std::slice::from_ref(&protected_url));
        assert_eq!(
            matches
                .iter()
                .map(|(range, emoji)| (&source[range.clone()], *emoji))
                .collect::<Vec<_>>(),
            vec![(":rocket:", "🚀")]
        );
    }
}
