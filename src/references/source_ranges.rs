//! Verified source alignment used only by explicit reference analysis.

use crate::ast::{Link, Node, Span};
use crate::parser::{EntityValue, scan_entity};

pub(super) fn bare_link_value<'a>(link: &Link<'a>) -> Option<&'a str> {
    let [Node::Text(text)] = &link.children[..] else {
        return None;
    };
    (text.span == link.span && link.title.is_none() && link.attributes.is_none())
        .then_some(text.value)
}

pub(super) fn bare_link_text<'a>(node: &Node<'a>) -> Option<&'a str> {
    match node {
        Node::Link(link) => bare_link_value(link),
        _ => None,
    }
}

struct Mapping {
    decoded_start: usize,
    decoded_end: usize,
    source_start: usize,
    source_end: usize,
    linear: bool,
}

pub(super) fn resolve_run(source: &str, run: &[Node<'_>], end: u32) -> Vec<Option<Span>> {
    let mut spans = vec![None; run.len()];
    if !run.iter().any(|node| bare_link_text(node).is_some()) {
        return spans;
    }
    let start = run[0].span().start as usize;
    let end = match run.last() {
        Some(Node::Text(text)) => text.span.end,
        _ => end,
    };
    let Some(raw) = source.get(start..end as usize) else {
        return spans;
    };
    // The GFM post-pass splits one coalesced text run into text and links.
    // Rejoin its unchanged values for verification, without reparsing syntax.
    let mut expected = String::new();
    let mut links = Vec::new();
    for (index, node) in run.iter().enumerate() {
        let decoded_start = expected.len();
        let value = match node {
            Node::Text(text) => text.value,
            _ => bare_link_text(node).unwrap_or_default(),
        };
        expected.push_str(value);
        if matches!(node, Node::Link(_)) {
            links.push((index, decoded_start, expected.len()));
        }
    }
    let Some(mapping) = align(raw, &expected, false).or_else(|| align(raw, &expected, true)) else {
        // A wrong source or manually edited tree must never acquire a false span.
        return spans;
    };
    for (index, decoded_start, decoded_end) in links {
        let source_start = boundary(&mapping, decoded_start, false);
        let source_end = boundary(&mapping, decoded_end, true);
        if let (Some(source_start), Some(source_end)) = (source_start, source_end) {
            spans[index] = Some(Span::new(
                (start + source_start) as u32,
                (start + source_end) as u32,
            ));
        }
    }
    spans
}

fn boundary(mapping: &[Mapping], offset: usize, end: bool) -> Option<usize> {
    let index = mapping.partition_point(|token| {
        if end {
            token.decoded_end < offset
        } else {
            token.decoded_end <= offset
        }
    });
    let token = mapping.get(index)?;
    Some(if token.linear {
        token.source_start + offset - token.decoded_start
    } else if end {
        token.source_end
    } else {
        token.source_start
    })
}

enum Token<'a> {
    Slice(&'a str, usize, bool),
    Character(char, usize),
}

impl Token<'_> {
    fn matched(&self, expected: &[u8]) -> Option<(usize, usize, bool)> {
        let mut buffer = [0; 4];
        let (value, consumed, linear) = match self {
            Self::Slice(value, consumed, linear) => (value.as_bytes(), *consumed, *linear),
            Self::Character(value, consumed) => {
                (value.encode_utf8(&mut buffer).as_bytes(), *consumed, false)
            }
        };
        let matched = value.len().min(expected.len());
        (value[..matched] == expected[..matched]).then_some((
            matched,
            if linear { matched } else { consumed },
            linear,
        ))
    }
}

fn token(raw: &str) -> Token<'_> {
    let bytes = raw.as_bytes();
    match bytes[0] {
        b'&' => match scan_entity(raw) {
            Some((EntityValue::Named(value), consumed)) => Token::Slice(value, consumed, false),
            Some((EntityValue::Char(value), consumed)) => Token::Character(value, consumed),
            None => Token::Slice("&", 1, true),
        },
        b'\\' if bytes.get(1).is_some_and(u8::is_ascii_punctuation) => {
            Token::Slice(&raw[1..2], 2, false)
        }
        b'\\' => Token::Slice("\\", 1, true),
        b'\r' => Token::Slice("\n", 1 + usize::from(bytes.get(1) == Some(&b'\n')), false),
        b'\n' => Token::Slice("\n", 1, true),
        0 => Token::Character('\u{fffd}', 1),
        _ => {
            let mut length = bytes
                .iter()
                .position(|byte| matches!(byte, b'&' | b'\\' | b'\r' | b'\n' | 0))
                .unwrap_or(raw.len());
            if bytes
                .get(length)
                .is_some_and(|byte| matches!(byte, b'\r' | b'\n'))
            {
                let trimmed = raw[..length].trim_end_matches(' ').len();
                if trimmed > 0 {
                    length = trimmed;
                }
            }
            Token::Slice(&raw[..length], length, true)
        }
    }
}

fn align(raw: &str, expected: &str, skip_comments: bool) -> Option<Vec<Mapping>> {
    let mut mapping = Vec::new();
    let mut authored = 0;
    let mut decoded = 0;
    while decoded < expected.len() {
        let rest = raw.get(authored..).filter(|rest| !rest.is_empty())?;
        let value = &expected.as_bytes()[decoded..];
        // The inline parser trims trailing spaces before soft line endings.
        if value[0] == b'\n' {
            let spaces = rest.bytes().take_while(|&byte| byte == b' ').count();
            if spaces > 0
                && rest
                    .as_bytes()
                    .get(spaces)
                    .is_some_and(|byte| matches!(byte, b'\r' | b'\n'))
            {
                authored += spaces;
                continue;
            }
        }
        let physical_newline = matches!(rest.as_bytes()[0], b'\r' | b'\n');
        let (matched, consumed, linear) = token(rest).matched(value)?;
        mapping.push(Mapping {
            decoded_start: decoded,
            decoded_end: decoded + matched,
            source_start: authored,
            source_end: authored + consumed,
            linear,
        });
        decoded += matched;
        authored += consumed;
        if physical_newline && decoded < expected.len() {
            authored += continuation_prefix(
                &raw[authored..],
                &expected.as_bytes()[decoded..],
                skip_comments,
            );
        }
    }
    syntax_tail(&raw[authored..]).then_some(mapping)
}

fn syntax_tail(raw: &str) -> bool {
    // A last link's parser end can be in decoded coordinates. The enclosing
    // span may also include closing emphasis/span markers, heading attributes,
    // or a setext underline. Reject unconsumed prose instead of accepting an
    // earlier identical line (for example one removed by line comments).
    let rest = raw.trim_matches(|character: char| {
        character.is_ascii_whitespace()
            || matches!(
                character,
                '*' | '_' | '~' | '+' | '^' | '[' | ']' | '#' | '-' | '='
            )
    });
    rest.is_empty()
        || (rest.starts_with('{') && rest.ends_with('}') && !rest.contains(['\r', '\n']))
}

fn continuation_prefix(mut raw: &str, expected: &[u8], skip_comments: bool) -> usize {
    let mut skipped = 0;
    if skip_comments {
        loop {
            let indentation = raw.bytes().take_while(|&byte| byte == b' ').count();
            if indentation > 3 || !raw[indentation..].starts_with("//") {
                break;
            }
            let length = raw
                .bytes()
                .position(|byte| matches!(byte, b'\r' | b'\n'))
                .unwrap_or(raw.len());
            let terminator = if raw.as_bytes().get(length) == Some(&b'\r') {
                1 + usize::from(raw.as_bytes().get(length + 1) == Some(&b'\n'))
            } else {
                usize::from(raw.as_bytes().get(length) == Some(&b'\n'))
            };
            let consumed = length + terminator;
            skipped += consumed;
            raw = &raw[consumed..];
        }
    }
    // Container maps have already removed quote markers and list indentation
    // from AST text. Verify a complete continuation-line prefix before skipping
    // any bytes, so an authored literal '>' remains visible and correctly mapped.
    let limit = raw
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t' | b'>'))
        .count();
    skipped
        + (0..=limit)
            .find(|&skip| matches_line(&raw[skip..], expected))
            .unwrap_or(0)
}

fn matches_line(mut raw: &str, mut expected: &[u8]) -> bool {
    while !raw.is_empty() && !expected.is_empty() {
        if matches!(raw.as_bytes()[0], b'\r' | b'\n') {
            return expected[0] == b'\n';
        }
        if expected[0] == b'\n' {
            let spaces = raw.bytes().take_while(|&byte| byte == b' ').count();
            if spaces > 0
                && raw
                    .as_bytes()
                    .get(spaces)
                    .is_some_and(|byte| matches!(byte, b'\r' | b'\n'))
            {
                return true;
            }
        }
        let Some((matched, consumed, _)) = token(raw).matched(expected) else {
            return false;
        };
        expected = &expected[matched..];
        raw = &raw[consumed..];
    }
    expected.is_empty() || raw.is_empty()
}
