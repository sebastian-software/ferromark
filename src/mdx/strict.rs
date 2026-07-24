use super::expr::find_expression_end;
use super::jsx_tag::parse_jsx_tag;
use super::splitter::{is_esm_start, try_esm};
use super::{MdxDiagnostic, MdxDiagnosticCode, SpannedSegment, segment_spanned};
use crate::Range;

const UNTERMINATED_EXPRESSION: &str = "expected `}` to close this flow expression";
const UNTERMINATED_JSX_TAG: &str = "expected `>` to close this JSX tag";
const INVALID_JSX_TAG: &str = "invalid JSX tag structure";
const UNEXPECTED_JSX_CLOSING_TAG: &str = "closing JSX tag has no matching opening tag";
const MISMATCHED_JSX_CLOSING_TAG: &str = "closing JSX tag does not match the innermost opening tag";
const UNCLOSED_JSX_TAG: &str = "expected a matching closing JSX tag";
const INVALID_ESM_POSITION: &str = "ESM blocks must begin at column 1 after a blank Markdown line";

#[derive(Debug)]
struct OpenTag {
    name: String,
    range: Range,
}

pub(super) fn segment_strict(input: &str) -> Result<Vec<SpannedSegment<'_>>, Vec<MdxDiagnostic>> {
    let diagnostics = validate(input);
    if diagnostics.is_empty() {
        Ok(segment_spanned(input))
    } else {
        Err(diagnostics)
    }
}

fn validate(input: &str) -> Vec<MdxDiagnostic> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut diagnostics = Vec::new();
    let mut open_tags = Vec::new();
    let mut in_paragraph = false;
    let mut pos = 0;

    while pos < len {
        let line_start = pos;
        let first_non_ws = skip_indentation(bytes, pos);

        if first_non_ws >= len {
            break;
        }

        let first = bytes[first_non_ws];
        if first == b'\n' || first == b'\r' {
            in_paragraph = false;
            pos = next_line(bytes, pos);
            continue;
        }

        if first_non_ws == pos && !in_paragraph {
            if let Some(esm_end) = try_esm(bytes, pos) {
                in_paragraph = false;
                pos = esm_end;
                continue;
            }
        }

        if is_esm_start(&bytes[first_non_ws..]) {
            diagnostics.push(diagnostic(
                MdxDiagnosticCode::InvalidEsmPosition,
                INVALID_ESM_POSITION,
                first_non_ws,
                first_non_ws + "import".len(),
                None,
            ));
            in_paragraph = true;
            pos = next_line(bytes, pos);
            continue;
        }

        if first == b'{' {
            match find_expression_end(&bytes[first_non_ws..]) {
                Some(expression_len) => {
                    let end = first_non_ws + expression_len;
                    if !has_trailing_content(bytes, end) {
                        in_paragraph = false;
                        pos = consume_trailing_newline(bytes, end);
                        continue;
                    }
                }
                None => {
                    diagnostics.push(diagnostic(
                        MdxDiagnosticCode::UnterminatedExpression,
                        UNTERMINATED_EXPRESSION,
                        first_non_ws,
                        len,
                        None,
                    ));
                    break;
                }
            }
        }

        if is_jsx_candidate(bytes, first_non_ws) {
            if let Some(tag) = parse_jsx_tag(&bytes[first_non_ws..]) {
                let end = first_non_ws + tag.end_offset;
                if !has_trailing_content(bytes, end) {
                    let range = Range::from_usize(first_non_ws, end);
                    if has_empty_attribute_value(&bytes[first_non_ws..end]) {
                        diagnostics.push(MdxDiagnostic {
                            code: MdxDiagnosticCode::InvalidJsxTag,
                            message: INVALID_JSX_TAG,
                            primary_range: range,
                            related_range: None,
                        });
                    } else if tag.is_closing {
                        validate_closing_tag(&mut diagnostics, &mut open_tags, tag.name, range);
                    } else if !tag.is_self_closing {
                        open_tags.push(OpenTag {
                            name: tag.name.to_owned(),
                            range,
                        });
                    }
                    in_paragraph = false;
                    pos = consume_trailing_newline(bytes, end);
                    continue;
                }
            } else if let Some(expression_start) = unterminated_tag_expression(bytes, first_non_ws)
            {
                diagnostics.push(diagnostic(
                    MdxDiagnosticCode::UnterminatedExpression,
                    UNTERMINATED_EXPRESSION,
                    expression_start,
                    len,
                    None,
                ));
                break;
            } else if let Some(end) = find_tag_terminator(bytes, first_non_ws) {
                diagnostics.push(diagnostic(
                    MdxDiagnosticCode::InvalidJsxTag,
                    INVALID_JSX_TAG,
                    first_non_ws,
                    end,
                    None,
                ));
                in_paragraph = false;
                pos = consume_trailing_newline(bytes, end);
                continue;
            } else {
                diagnostics.push(diagnostic(
                    MdxDiagnosticCode::UnterminatedJsxTag,
                    UNTERMINATED_JSX_TAG,
                    first_non_ws,
                    len,
                    None,
                ));
                break;
            }
        }

        in_paragraph = true;
        pos = next_line(bytes, line_start);
    }

    diagnostics.extend(open_tags.into_iter().map(|tag| MdxDiagnostic {
        code: MdxDiagnosticCode::UnclosedJsxTag,
        message: UNCLOSED_JSX_TAG,
        primary_range: tag.range,
        related_range: None,
    }));
    diagnostics
}

fn validate_closing_tag(
    diagnostics: &mut Vec<MdxDiagnostic>,
    open_tags: &mut Vec<OpenTag>,
    name: &str,
    range: Range,
) {
    let Some(top) = open_tags.last() else {
        diagnostics.push(MdxDiagnostic {
            code: MdxDiagnosticCode::UnexpectedJsxClosingTag,
            message: UNEXPECTED_JSX_CLOSING_TAG,
            primary_range: range,
            related_range: None,
        });
        return;
    };

    if top.name == name {
        open_tags.pop();
        return;
    }

    diagnostics.push(MdxDiagnostic {
        code: MdxDiagnosticCode::MismatchedJsxClosingTag,
        message: MISMATCHED_JSX_CLOSING_TAG,
        primary_range: range,
        related_range: Some(top.range),
    });

    if let Some(match_index) = open_tags.iter().rposition(|tag| tag.name == name) {
        open_tags.truncate(match_index);
    }
}

fn diagnostic(
    code: MdxDiagnosticCode,
    message: &'static str,
    start: usize,
    end: usize,
    related_range: Option<Range>,
) -> MdxDiagnostic {
    MdxDiagnostic {
        code,
        message,
        primary_range: Range::from_usize(start, end),
        related_range,
    }
}

fn is_jsx_candidate(bytes: &[u8], start: usize) -> bool {
    if bytes.get(start) != Some(&b'<') {
        return false;
    }

    bytes
        .get(start + 1)
        .is_some_and(|byte| *byte == b'/' || *byte == b'>' || byte.is_ascii_alphabetic())
}

fn unterminated_tag_expression(bytes: &[u8], start: usize) -> Option<usize> {
    let mut pos = start + 1;
    while pos < bytes.len() {
        match bytes[pos] {
            b'>' => return None,
            b'"' | b'\'' => pos = skip_quoted(bytes, pos)?,
            b'{' => match find_expression_end(&bytes[pos..]) {
                Some(length) => pos += length,
                None => return Some(pos),
            },
            _ => pos += 1,
        }
    }
    None
}

fn find_tag_terminator(bytes: &[u8], start: usize) -> Option<usize> {
    let mut pos = start + 1;
    while pos < bytes.len() {
        match bytes[pos] {
            b'>' => return Some(pos + 1),
            b'"' | b'\'' => pos = skip_quoted(bytes, pos)?,
            b'{' => pos += find_expression_end(&bytes[pos..])?,
            _ => pos += 1,
        }
    }
    None
}

fn has_empty_attribute_value(tag: &[u8]) -> bool {
    let mut pos = 1;
    if tag.get(pos) == Some(&b'/') {
        return false;
    }
    if tag.get(pos) == Some(&b'>') {
        return false;
    }

    while matches!(
        tag.get(pos),
        Some(byte) if byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-' | b'.' | b':')
    ) {
        pos += 1;
    }

    while pos < tag.len() {
        pos = skip_whitespace(tag, pos);
        if matches!(tag.get(pos), Some(b'>') | Some(b'/')) {
            return false;
        }
        if tag.get(pos) == Some(&b'{') {
            let Some(length) = find_expression_end(&tag[pos..]) else {
                return false;
            };
            pos += length;
            continue;
        }

        while matches!(
            tag.get(pos),
            Some(byte)
                if byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'-' | b'.' | b':')
        ) {
            pos += 1;
        }
        pos = skip_whitespace(tag, pos);
        if tag.get(pos) != Some(&b'=') {
            continue;
        }
        pos = skip_whitespace(tag, pos + 1);
        if matches!(tag.get(pos), Some(b'>'))
            || (tag.get(pos) == Some(&b'/') && tag.get(pos + 1) == Some(&b'>'))
        {
            return true;
        }

        match tag.get(pos) {
            Some(b'"' | b'\'') => {
                let Some(next) = skip_quoted(tag, pos) else {
                    return false;
                };
                pos = next;
            }
            Some(b'{') => {
                let Some(length) = find_expression_end(&tag[pos..]) else {
                    return false;
                };
                pos += length;
            }
            Some(_) => {
                while matches!(
                    tag.get(pos),
                    Some(byte) if !byte.is_ascii_whitespace() && !matches!(*byte, b'>' | b'/')
                ) {
                    pos += 1;
                }
            }
            None => return false,
        }
    }

    false
}

fn skip_quoted(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = bytes[start];
    let mut pos = start + 1;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' => pos += 2,
            byte if byte == quote => return Some(pos + 1),
            _ => pos += 1,
        }
    }
    None
}

fn skip_whitespace(bytes: &[u8], mut pos: usize) -> usize {
    while matches!(bytes.get(pos), Some(byte) if byte.is_ascii_whitespace()) {
        pos += 1;
    }
    pos
}

fn skip_indentation(bytes: &[u8], mut pos: usize) -> usize {
    while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
        pos += 1;
    }
    pos
}

fn next_line(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos] != b'\n' {
        pos += 1;
    }
    if pos < bytes.len() { pos + 1 } else { pos }
}

fn has_trailing_content(bytes: &[u8], mut pos: usize) -> bool {
    while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
        pos += 1;
    }
    matches!(bytes.get(pos), Some(byte) if *byte != b'\n' && *byte != b'\r')
}

fn consume_trailing_newline(bytes: &[u8], mut pos: usize) -> usize {
    while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
        pos += 1;
    }
    if bytes.get(pos) == Some(&b'\r') {
        pos += 1;
    }
    if bytes.get(pos) == Some(&b'\n') {
        pos += 1;
    }
    pos
}
