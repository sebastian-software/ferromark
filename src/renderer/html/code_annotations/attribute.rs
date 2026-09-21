//! Parser for ox-content `annotate="..."` code fence metadata.
//!
//! The attribute syntax accepts semicolon-separated `kind:line-list` pairs, where the
//! line list can contain single line numbers and inclusive ranges. It intentionally
//! stays independent from VitePress metadata so users can opt into a stable native
//! grammar without changing their code fence language token.

// BTreeMap preserves ascending code line order during annotation rendering.
use std::collections::BTreeMap;
use std::num::IntErrorKind;

use smallvec::SmallVec;

use super::state::CodeAnnotationKind;

pub(in crate::renderer::html) fn parse_code_annotations(
    meta: &str,
    key: &str,
    line_count: usize,
) -> BTreeMap<usize, SmallVec<[CodeAnnotationKind; 2]>> {
    let Some(value) = extract_meta_attribute(meta, key) else {
        return BTreeMap::new();
    };

    let mut annotations = BTreeMap::new();

    for entry in value.split(';') {
        let Some((raw_kind, raw_lines)) = entry.split_once(':') else {
            continue;
        };

        let Some(kind) = CodeAnnotationKind::from_str(raw_kind.trim()) else {
            continue;
        };

        for line_number in parse_line_numbers(raw_lines.trim(), line_count) {
            push_code_annotation(&mut annotations, line_number, kind);
        }
    }

    annotations
}

fn extract_meta_attribute<'a>(meta: &'a str, target: &str) -> Option<&'a str> {
    // Fence meta is short but this parser runs for every annotated code block.
    // Scan the byte slice once and return a borrowed value slice for the target
    // attribute instead of tokenizing every key/value pair into owned strings.
    let bytes = meta.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        let key_start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'=' {
            index += 1;
        }

        if key_start == index {
            index += 1;
            continue;
        }

        let key = &meta[key_start..index];

        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() || bytes[index] != b'=' {
            continue;
        }

        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        let value = if bytes[index] == b'"' || bytes[index] == b'\'' {
            let quote = bytes[index];
            index += 1;
            let value_start = index;

            while index < bytes.len() && bytes[index] != quote {
                index += 1;
            }

            let value_end = index;
            if index < bytes.len() {
                index += 1;
            }
            &meta[value_start..value_end]
        } else {
            let value_start = index;
            while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            &meta[value_start..index]
        };

        if key == target {
            return Some(value);
        }
    }

    None
}

/// Parses a line list into the ascending, unique line numbers it selects.
///
/// `line_count` is the number of lines the annotated block actually has. A line
/// number past the end annotates nothing — the appliers look the line up with
/// `get_mut` and skip a miss — so the list is clamped to the block before it is
/// expanded. Without that clamp the written-out range decides the work: the
/// attribute syntax accepts any `usize` bound, so `highlight:1-40000` expanded
/// 40,000 entries while checking `contains` on each push (quadratic), and the
/// VitePress form `{1-999999999999}` never finished at all.
///
/// The entries are therefore collected as clamped ranges, sorted, and expanded
/// once without overlap, which bounds both the emitted list and the expansion
/// work by `line_count` and makes the rest of the cost linear in the metadata.
/// The result — ascending and de-duplicated — is what the previous
/// push-then-sort loop produced, so valid metadata renders byte for byte as
/// before.
pub(in crate::renderer::html) fn parse_line_numbers(
    value: &str,
    line_count: usize,
) -> SmallVec<[usize; 4]> {
    // Line lists are usually tiny (`1`, `1,3`, `2-4`), so SmallVec keeps the
    // common case stack-backed for both the ranges and the expanded list.
    let mut ranges: SmallVec<[(usize, usize); 4]> = SmallVec::new();

    for part in value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if let Some((raw_start, raw_end)) = part.split_once('-') {
            let (Some(start), Some(end)) = (parse_line_bound(raw_start), parse_line_bound(raw_end))
            else {
                continue;
            };

            // A range that starts past the last line selects nothing, so it is
            // dropped rather than clamped: clamping its start would move it
            // onto the final line, which the author never named.
            if start == 0 || end < start || start > line_count {
                continue;
            }

            ranges.push((start, end.min(line_count)));
            continue;
        }

        let Some(line_number) = parse_line_bound(part) else {
            continue;
        };

        if line_number > 0 && line_number <= line_count {
            ranges.push((line_number, line_number));
        }
    }

    ranges.sort_unstable();

    let mut line_numbers = SmallVec::new();
    // First line that has not been emitted yet. Ranges arrive sorted by start,
    // so skipping ahead to this line drops every overlap in one comparison and
    // keeps the output ascending and unique.
    let mut next_line = 1_usize;
    for (start, end) in ranges {
        let start = start.max(next_line);
        if start > end {
            continue;
        }
        line_numbers.extend(start..=end);
        next_line = end.saturating_add(1);
    }

    line_numbers
}

/// Parses one bound of a line list.
///
/// A bound too large for `usize` is still a line number, just one no block can
/// have; saturating keeps such a range valid — and therefore clamped to the
/// block — instead of silently dropping the whole entry. Everything else keeps
/// `usize::from_str`'s judgment, including its acceptance of a leading `+`.
fn parse_line_bound(value: &str) -> Option<usize> {
    match value.trim().parse::<usize>() {
        Ok(line_number) => Some(line_number),
        Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(usize::MAX),
        Err(_) => None,
    }
}

fn push_code_annotation(
    annotations: &mut BTreeMap<usize, SmallVec<[CodeAnnotationKind; 2]>>,
    line_number: usize,
    kind: CodeAnnotationKind,
) {
    let kinds = annotations.entry(line_number).or_default();
    if !kinds.contains(&kind) {
        kinds.push(kind);
    }
}
