//! Root-only metadata extraction before Markdown normalization and prepasses.

use ferromark_ast::{FrontMatter, FrontMatterKind, Span};

use super::line_scan::{line_end, line_terminator_end};

pub(super) fn extract(source: &str) -> Option<FrontMatter<'_>> {
    let start = usize::from(source.starts_with('\u{feff}')) * 3;
    let bytes = source.as_bytes();
    let kind = match bytes.get(start..start + 3)? {
        b"---" => FrontMatterKind::Yaml,
        b"+++" => FrontMatterKind::Toml,
        _ => return None,
    };
    let delimiter = &bytes[start..start + 3];
    let opening_end = line_end(bytes, start + 3);
    if opening_end == bytes.len() || !only_spacing(&bytes[start + 3..opening_end]) {
        return None;
    }
    let content_start = line_terminator_end(bytes, opening_end);
    let mut cursor = content_start;
    while cursor < bytes.len() {
        let end = line_end(bytes, cursor);
        let next = line_terminator_end(bytes, end);
        let line = &bytes[cursor..end];
        if line.starts_with(delimiter) && only_spacing(&line[3..]) {
            return Some(FrontMatter {
                kind,
                value: &source[content_start..cursor],
                span: Span::new(start as u32, next as u32),
                content_span: Span::new(content_start as u32, cursor as u32),
            });
        }
        cursor = next;
    }
    None
}

fn only_spacing(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| matches!(byte, b' ' | b'\t'))
}
