//! Link reference definitions (CommonMark).
//!
//! Definitions (`[label]: destination "title"`) are usable anywhere in the
//! document, including before their definition site, so the root parser
//! runs a collection pre-pass over the source before block parsing starts.
//! The collection phase uses the same block grammar as the final parse, so
//! code, HTML, containers, and definition precedence have one structural owner.
//! Definition paragraphs are consumed by [`Parser::try_parse_definition_node`]
//! in both phases; collection omits ordinary inline parsing.

use crate::ast::{Definition, Node, Span};
use compact_str::CompactString;
use rustc_hash::FxHashMap;

use super::Parser;
use super::line_comments::CommentDefinitionRegion;
use super::line_scan::{is_line_ending_byte, line_terminator_end};
use super::spans::SpanMap;

mod collect;
mod scan;

use scan::{line_end_if_blank_after, next_blank_line, skip_ws_one_newline};
// Block quotes, list items and the footnote-label scan share these line
// recognizers through the open-paragraph tracker.
pub(super) use scan::{fence_open, is_fence_close};

#[derive(Debug)]
pub(super) struct ReferenceDef<'a> {
    pub url: &'a str,
    pub title: Option<&'a str>,
}

pub(super) type ReferenceMap<'a> = FxHashMap<CompactString, ReferenceDef<'a>>;

/// One parsed definition and the bytes it consumed.
pub(super) struct ParsedDefinition<'a> {
    pub label: &'a str,
    pub url: &'a str,
    pub title: Option<&'a str>,
    pub consumed: usize,
}

#[inline]
const fn is_ascii_unicode_whitespace(byte: u8) -> bool {
    // Matches `char::is_whitespace()` on every ASCII byte (0x00..=0x7F),
    // including vertical tab (0x0B), which `u8::is_ascii_whitespace()` omits.
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

#[inline]
fn is_normalized_ascii_reference_label(label: &str) -> bool {
    let bytes = label.as_bytes();
    if bytes.is_empty() {
        return true;
    }
    if bytes[0] == b' ' || bytes[bytes.len() - 1] == b' ' {
        return false;
    }
    let mut prev_space = false;
    for &byte in bytes {
        if byte >= 0x80 || byte.is_ascii_uppercase() {
            return false;
        }
        if byte == b' ' {
            if prev_space {
                return false;
            }
            prev_space = true;
        } else if is_ascii_unicode_whitespace(byte) {
            return false;
        } else {
            prev_space = false;
        }
    }
    true
}

#[inline]
fn normalize_ascii_reference_label_into(bytes: &[u8], out: &mut [u8]) -> usize {
    let mut written = 0usize;
    let mut pending_space = false;
    for &byte in bytes {
        if is_ascii_unicode_whitespace(byte) {
            pending_space = written > 0;
        } else {
            if pending_space {
                out[written] = b' ';
                written += 1;
                pending_space = false;
            }
            out[written] = byte.to_ascii_lowercase();
            written += 1;
        }
    }
    written
}

impl<'a> Parser<'a> {
    /// Normalizes a reference label: trim, collapse internal whitespace to
    /// single spaces, lowercase (approximating Unicode case fold).
    pub(super) fn normalize_reference_label(label: &str) -> CompactString {
        if is_normalized_ascii_reference_label(label) {
            return CompactString::from(label);
        }
        if label.is_ascii() {
            let bytes = label.as_bytes();
            if bytes.len() <= 64 {
                let mut buf = [0u8; 64];
                let len = normalize_ascii_reference_label_into(bytes, &mut buf);
                if let Ok(normalized) = std::str::from_utf8(&buf[..len]) {
                    return CompactString::from(normalized);
                }
            }
            let mut out = CompactString::with_capacity(bytes.len());
            let mut pending_space = false;
            for &byte in bytes {
                if is_ascii_unicode_whitespace(byte) {
                    pending_space = !out.is_empty();
                } else {
                    if pending_space {
                        out.push(' ');
                        pending_space = false;
                    }
                    out.push(byte.to_ascii_lowercase() as char);
                }
            }
            return out;
        }
        let mut out = CompactString::default();
        let mut pending_space = false;
        for ch in label.chars() {
            if ch.is_whitespace() {
                pending_space = !out.is_empty();
            } else {
                if pending_space {
                    out.push(' ');
                    pending_space = false;
                }
                for lowered in ch.to_lowercase() {
                    // Case folding (not just lowercasing): the sharp s
                    // folds to "ss", so [SS] and [ẞ] label-match.
                    if lowered == 'ß' {
                        out.push_str("ss");
                    } else {
                        out.push(lowered);
                    }
                }
            }
        }
        out
    }

    pub(super) fn lookup_reference(&self, raw_label: &str) -> Option<&ReferenceDef<'a>> {
        let definitions = self.definitions.as_ref()?;
        if definitions.is_empty() {
            return None;
        }
        if is_normalized_ascii_reference_label(raw_label) {
            return definitions.get(raw_label);
        }
        if raw_label.is_ascii() && raw_label.len() <= 64 {
            let mut buf = [0u8; 64];
            let len = normalize_ascii_reference_label_into(raw_label.as_bytes(), &mut buf);
            if let Ok(normalized) = std::str::from_utf8(&buf[..len]) {
                return definitions.get(normalized);
            }
        }
        definitions.get(Self::normalize_reference_label(raw_label).as_str())
    }

    /// Parses a single definition at the start of `text`. `text` must not
    /// span a blank line (callers cut at paragraph boundaries).
    pub(super) fn parse_reference_definition(&self, text: &'a str) -> Option<ParsedDefinition<'a>> {
        if !self.options.allow_link_refs {
            return None;
        }
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        if i > 3 || bytes.get(i) != Some(&b'[') {
            return None;
        }

        let label_start = i + 1;
        let mut j = label_start;
        loop {
            if j - label_start > 1000 {
                return None;
            }
            match bytes.get(j)? {
                b'\\' if bytes.get(j + 1).is_some_and(u8::is_ascii_punctuation) => j += 2,
                b']' => break,
                b'[' => return None,
                _ => j += 1,
            }
        }
        let label = &text[label_start..j];
        #[allow(
            clippy::disallowed_methods,
            reason = "blank-label checks must agree with `normalize_reference_label`"
        )]
        let blank_label = label.trim().is_empty();
        if blank_label {
            return None;
        }
        // With footnotes enabled, `[^label]:` belongs to the footnote
        // parser; treating it as a link reference here would turn every
        // `[^label]` in the document into a link.
        if self.options.footnotes && label.starts_with('^') {
            return None;
        }
        if bytes.get(j + 1) != Some(&b':') {
            return None;
        }

        let dest_start = skip_ws_one_newline(bytes, j + 2)?;
        let (raw_url, after_dest, _) = super::inline::parse_link_destination(text, dest_start)?;
        if raw_url.is_empty() && bytes.get(dest_start) != Some(&b'<') {
            return None;
        }

        // End of the destination line, in case the title turns out absent
        // or invalid: only spaces/tabs may follow on that line.
        let dest_line_end = line_end_if_blank_after(bytes, after_dest);

        let mut k = after_dest;
        let mut ws_between = false;
        while matches!(bytes.get(k), Some(b' ' | b'\t')) {
            k += 1;
            ws_between = true;
        }
        let title_on_next_line = bytes.get(k).is_some_and(|byte| is_line_ending_byte(*byte));
        if title_on_next_line {
            k = line_terminator_end(bytes, k);
            ws_between = true;
            while matches!(bytes.get(k), Some(b' ' | b'\t')) {
                k += 1;
            }
        }

        if ws_between
            && let Some((raw_title, after_title)) = super::inline::parse_link_title(text, k)
            && let Some(end) = line_end_if_blank_after(bytes, after_title)
        {
            return Some(ParsedDefinition {
                label,
                url: self.unescape_reference_component(raw_url),
                title: Some(self.unescape_reference_component(raw_title)),
                consumed: end,
            });
        }

        // No (valid) title: the definition is still good if its
        // destination line ends cleanly.
        let end = dest_line_end?;
        Some(ParsedDefinition {
            label,
            url: self.unescape_reference_component(raw_url),
            title: None,
            consumed: end,
        })
    }

    /// Consumes one definition at the current block position, emitting the
    /// AST node. Returns `None` when the position does not start a
    /// definition (the caller falls through to paragraph parsing).
    pub(super) fn try_parse_definition_node(&mut self) -> Option<Node<'a>> {
        let start = self.position;
        // Definitions cannot contain blank lines; cut the candidate region
        // at the next one so the destination/title scanners stay in
        // paragraph bounds. Consecutive definitions share that boundary, so
        // reuse the last one rather than re-scanning to it per definition —
        // that scan is what made a definition-only document quadratic.
        let region_end = self.definition_region_end(start);
        let cached = self
            .comment_definition_region
            .as_ref()
            .is_some_and(|region| {
                region.start <= start && start < region.end && region.end == region_end
            });
        if self.options.line_comments && !cached {
            let (text, map) = self.without_line_comments(start, region_end);
            self.comment_definition_region = Some(std::rc::Rc::new(CommentDefinitionRegion {
                start,
                end: region_end,
                text,
                map,
            }));
        }
        let (parsed, end) = if let Some(region) = self.comment_definition_region.as_ref()
            && region.start <= start
            && start < region.end
            && region.end == region_end
            && let Some(map) = &region.map
        {
            let offset = map.generated_line_start(start)?;
            let parsed = self.parse_reference_definition(&region.text[offset..])?;
            let span = map.map_span(Span::new(offset as u32, (offset + parsed.consumed) as u32));
            (parsed, span.end as usize)
        } else {
            let parsed = self.parse_reference_definition(&self.source[start..region_end])?;
            let end = start + parsed.consumed;
            (parsed, end)
        };

        let identifier = if is_normalized_ascii_reference_label(parsed.label) {
            parsed.label
        } else {
            self.allocator
                .alloc_str(Self::normalize_reference_label(parsed.label).as_str())
        };
        self.position = end;
        Some(Node::Definition(self.allocator.boxed(Definition {
            identifier,
            label: Some(parsed.label),
            url: parsed.url,
            title: parsed.title,
            span: Span::new(start as u32, end as u32),
        })))
    }

    /// Position of the first blank line at or after `start`, reusing the
    /// previous scan whenever `start` falls inside the window it covered.
    fn definition_region_end(&mut self, start: usize) -> usize {
        if let Some((scanned_from, blank_line)) = self.definition_region
            && (scanned_from..=blank_line).contains(&start)
        {
            // No blank line lies in `scanned_from..blank_line`, so the first
            // one at or after any `start` in that window is the same one.
            return blank_line;
        }
        let blank_line = next_blank_line(self.source.as_bytes(), start);
        self.definition_region = Some((start, blank_line));
        blank_line
    }

    fn unescape_reference_component(&self, raw: &'a str) -> &'a str {
        self.unescape_link_component(raw)
    }
}

#[cfg(test)]
mod tests {
    use compact_str::CompactString;

    use super::{Parser, is_normalized_ascii_reference_label};

    fn oracle_normalize(label: &str) -> CompactString {
        let mut out = CompactString::default();
        let mut pending_space = false;
        for ch in label.chars() {
            if ch.is_whitespace() {
                pending_space = !out.is_empty();
            } else {
                if pending_space {
                    out.push(' ');
                    pending_space = false;
                }
                for lowered in ch.to_lowercase() {
                    if lowered == 'ß' {
                        out.push_str("ss");
                    } else {
                        out.push(lowered);
                    }
                }
            }
        }
        out
    }

    #[test]
    fn ascii_reference_label_fast_path_matches_oracle() {
        for byte in 0u8..=0x7f {
            let raw = [b'a', byte, b'b'];
            let s = std::str::from_utf8(&raw).unwrap();
            let got = Parser::normalize_reference_label(s);
            let expected = oracle_normalize(s);
            assert_eq!(got, expected, "byte {byte:#x}");
            assert_eq!(
                is_normalized_ascii_reference_label(s),
                got.as_str() == s,
                "is_normalized mismatch for byte {byte:#x}"
            );
        }
        let long_ascii = "  ABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABCABC   DEF\t\x0b\x0c\r\nGHI  ";
        for sample in [
            "",
            " ",
            "foo",
            "foo bar",
            "foo  bar",
            " foo",
            "foo ",
            "Foo\tBar\x0bBaz\x0cQux",
            "SS",
            "ß",
            "ẞ",
            "Äöü  Test",
            long_ascii,
        ] {
            assert_eq!(
                Parser::normalize_reference_label(sample),
                oracle_normalize(sample),
                "sample {sample:?}"
            );
        }
    }
}
