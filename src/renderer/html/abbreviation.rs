//! Optional, render-time recognition of technical abbreviations.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::renderer::html::escape::write_attribute_escaped_into;
use rustc_hash::FxHashMap;

/// A deliberately small, maintained dictionary of common technical terms.
///
/// Values are authored project data rather than an imported corpus, so there
/// is no third-party dictionary license or generated-data update process.
const BUILT_INS: &[(&str, &str)] = &[
    ("AI", "Artificial Intelligence"),
    ("API", "Application Programming Interface"),
    (
        "ASCII",
        "American Standard Code for Information Interchange",
    ),
    ("CLI", "Command-Line Interface"),
    ("CPU", "Central Processing Unit"),
    ("CSS", "Cascading Style Sheets"),
    ("DNS", "Domain Name System"),
    ("DOM", "Document Object Model"),
    ("GPU", "Graphics Processing Unit"),
    ("HTML", "Hypertext Markup Language"),
    ("HTML5", "Hypertext Markup Language, version 5"),
    ("HTTP", "Hypertext Transfer Protocol"),
    ("HTTP/2", "Hypertext Transfer Protocol, version 2"),
    ("HTTPS", "Hypertext Transfer Protocol Secure"),
    ("IDE", "Integrated Development Environment"),
    ("IP", "Internet Protocol"),
    ("JSON", "JavaScript Object Notation"),
    ("JWT", "JSON Web Token"),
    ("MIME", "Multipurpose Internet Mail Extensions"),
    ("RAM", "Random Access Memory"),
    ("REST", "Representational State Transfer"),
    ("RPC", "Remote Procedure Call"),
    ("SQL", "Structured Query Language"),
    ("SSH", "Secure Shell"),
    ("SSL", "Secure Sockets Layer"),
    ("TCP", "Transmission Control Protocol"),
    ("TLS", "Transport Layer Security"),
    ("URI", "Uniform Resource Identifier"),
    ("URL", "Uniform Resource Locator"),
    ("UTF-8", "Unicode Transformation Format, 8-bit"),
    ("UTF8", "Unicode Transformation Format, 8-bit"),
    ("XML", "Extensible Markup Language"),
    ("YAML", "YAML Ain't Markup Language"),
];

#[derive(Debug)]
pub(super) struct Entry {
    term: Cow<'static, str>,
    title: Option<Cow<'static, str>>,
    suppressed: bool,
    is_override: bool,
}

/// Precompiled dictionary and token-start lookup for one reusable renderer.
#[derive(Debug)]
pub(super) struct AbbreviationMatcher {
    entries: Vec<Entry>,
    /// Half-open entry ranges grouped by first byte. This bounds the work per
    /// candidate to terms that can actually match its first byte.
    first_byte_ranges: [usize; 257],
    /// Exact built-in/override lookup for the heuristic fallback.
    exact_terms: FxHashMap<Cow<'static, str>, usize>,
}

/// The match action borrowed from a reusable dictionary entry.
pub(super) struct AbbreviationMatch<'a> {
    /// Byte after the matched term. A lowercase plural 's' is left outside.
    pub(super) end: usize,
    /// An explicit or built-in entry; None represents an unknown acronym.
    pub(super) entry: Option<&'a Entry>,
}

impl AbbreviationMatcher {
    /// Builds an immutable longest-term-first dictionary once per renderer.
    pub(super) fn new(overrides: BTreeMap<String, Option<String>>) -> Self {
        let mut entries = BUILT_INS
            .iter()
            .map(|(term, title)| Entry {
                term: Cow::Borrowed(term),
                title: Some(Cow::Borrowed(title)),
                suppressed: false,
                is_override: false,
            })
            .collect::<Vec<_>>();

        for (term, title) in overrides {
            if term.is_empty() {
                continue;
            }
            entries.push(Entry {
                term: Cow::Owned(term),
                suppressed: title.is_none(),
                title: title.filter(|value| !value.is_empty()).map(Cow::Owned),
                is_override: true,
            });
        }

        entries.sort_by(|left, right| {
            left.term
                .as_bytes()
                .first()
                .cmp(&right.term.as_bytes().first())
                .then_with(|| right.term.len().cmp(&left.term.len()))
                .then_with(|| right.is_override.cmp(&left.is_override))
                .then_with(|| left.term.cmp(&right.term))
        });

        let mut first_byte_ranges = [0; 257];
        let mut cursor = 0;
        for (first, range_start) in first_byte_ranges[..256].iter_mut().enumerate() {
            *range_start = cursor;
            while entries
                .get(cursor)
                .and_then(|entry| entry.term.as_bytes().first())
                .is_some_and(|byte| usize::from(*byte) == first)
            {
                cursor += 1;
            }
        }
        first_byte_ranges[256] = entries.len();

        let mut exact_terms = FxHashMap::default();
        for (index, entry) in entries.iter().enumerate() {
            if entry.is_override {
                exact_terms.insert(entry.term.clone(), index);
            } else {
                exact_terms.entry(entry.term.clone()).or_insert(index);
            }
        }

        Self {
            entries,
            first_byte_ranges,
            exact_terms,
        }
    }

    /// Finds an explicit entry first, then a complete uppercase token.
    pub(super) fn find<'a>(
        &'a self,
        text: &str,
        start: usize,
        before: Option<char>,
        after: Option<char>,
    ) -> Option<AbbreviationMatch<'a>> {
        let bytes = text.as_bytes();
        let first = *bytes.get(start)?;
        let first_index = usize::from(first);
        let range = self.first_byte_ranges[first_index]..self.first_byte_ranges[first_index + 1];
        if range.is_empty() && !first.is_ascii_uppercase() {
            return None;
        }
        if !has_token_boundary_before(text, start, before) {
            return None;
        }

        for entry in &self.entries[range] {
            if !text[start..].starts_with(entry.term.as_ref()) {
                continue;
            }
            let end = start + entry.term.len();
            if has_token_boundary_after(text, end, after) {
                return Some(AbbreviationMatch {
                    end,
                    entry: Some(entry),
                });
            }
        }

        let mut end = start;
        let mut uppercase_count = 0;
        while let Some(byte) = bytes.get(end).copied() {
            if byte.is_ascii_uppercase() {
                uppercase_count += 1;
                end += 1;
            } else if byte.is_ascii_digit() {
                end += 1;
            } else {
                break;
            }
        }
        if uppercase_count < 2 {
            return None;
        }

        let plural = bytes.get(end) == Some(&b's')
            && has_token_boundary_after(text, end.saturating_add(1), after);
        if !plural && !has_token_boundary_after(text, end, after) {
            return None;
        }

        let term = &text[start..end];
        let entry = self
            .exact_terms
            .get(term)
            .map(|index| &self.entries[*index]);
        Some(AbbreviationMatch { end, entry })
    }
}

/// Writes prose with abbreviation markup, escaping both text and titles. The
/// optional characters come from adjacent text nodes at the two ends of a run.
pub(super) fn write_abbreviations_into_with_boundaries(
    output: &mut String,
    text: &str,
    matcher: &AbbreviationMatcher,
    before: Option<char>,
    after: Option<char>,
) {
    let mut cursor = 0usize;
    let mut scan = 0usize;
    while scan < text.len() {
        let byte = text.as_bytes()[scan];
        let potential_start = matcher.first_byte_ranges[usize::from(byte)]
            < matcher.first_byte_ranges[usize::from(byte) + 1]
            || byte.is_ascii_uppercase();
        if potential_start && let Some(found) = matcher.find(text, scan, before, after) {
            if scan > cursor {
                crate::renderer::html::escape::write_escaped_into(output, &text[cursor..scan]);
            }
            let term = &text[scan..found.end];
            if !found.entry.is_some_and(|entry| entry.suppressed) {
                output.push_str("<abbr");
                if let Some(title) = found.entry.and_then(|entry| entry.title.as_deref()) {
                    output.push_str(" title=\"");
                    write_attribute_escaped_into(output, title);
                    output.push('"');
                }
                output.push('>');
                crate::renderer::html::escape::write_escaped_into(output, term);
                output.push_str("</abbr>");
            } else {
                crate::renderer::html::escape::write_escaped_into(output, term);
            }
            cursor = found.end;
            scan = found.end;
            continue;
        }

        let Some(character) = text[scan..].chars().next() else {
            break;
        };
        scan += character.len_utf8();
    }

    if cursor < text.len() {
        crate::renderer::html::escape::write_escaped_into(output, &text[cursor..]);
    }
}

fn has_token_boundary_before(text: &str, position: usize, before: Option<char>) -> bool {
    let previous = text[..position].chars().next_back().or(before);
    previous.is_none_or(|character| !is_identifier_character(character))
}

fn has_token_boundary_after(text: &str, position: usize, after: Option<char>) -> bool {
    let next = text[position..].chars().next().or(after);
    next.is_none_or(|character| !is_identifier_character(character))
}

fn is_identifier_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}
