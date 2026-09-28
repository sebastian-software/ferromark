//! Shared authored-attribute grammar. HTML name mapping belongs to the renderer.

use crate::allocator::{Box, String as ArenaString, Vec};
use crate::ast::{Attribute, ElementAttributes};

use super::Parser;

/// Shared shape for table and figure caption lines. The attachment point
/// validates the suffix against the enabled attribute grammar.
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
        && let Some(open) = trailing_attribute_open(without_close, false)
    {
        let caption = without_close[..open].trim_end_matches([' ', '\t']);
        if !allow_empty_caption && caption.is_empty() {
            return None;
        }
        return Some(CaptionLine {
            content: caption,
            content_offset,
            tokens: Some(&without_close[open + 1..]),
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

/// Locate the final suffix opener without treating braces inside quoted
/// values as new attribute blocks. A prior closed brace resets a candidate.
pub(super) fn trailing_attribute_open(content: &str, unicode_space: bool) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut candidate = None;
    let mut quote = None;
    let mut escaped = false;
    for (index, &byte) in bytes.iter().enumerate() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
        } else if candidate.is_some() && matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == b'{'
            && (index == 0
                || if unicode_space {
                    content[..index]
                        .chars()
                        .next_back()
                        .is_some_and(char::is_whitespace)
                } else {
                    content[..index].ends_with([' ', '\t'])
                })
        {
            candidate = Some(index);
        } else if byte == b'}' && index + 1 < bytes.len() {
            candidate = None;
        }
    }
    candidate
}

fn scan_id_classes<'a>(tokens: &'a str, mut on_token: impl FnMut(&'a str, bool)) -> Option<()> {
    let mut seen_id = false;
    let mut found = false;
    for token in tokens.split_whitespace() {
        let (name, is_id) = if let Some(name) = token.strip_prefix('#') {
            (name, true)
        } else {
            (token.strip_prefix('.')?, false)
        };
        if !valid_shorthand(name) || (is_id && seen_id) {
            return None;
        }
        seen_id |= is_id;
        found = true;
        on_token(name, is_id);
    }
    found.then_some(())
}

pub(super) struct ParsedAttributes<'a> {
    pub id: Option<&'a str>,
    pub classes: Vec<'a, &'a str>,
    pub values: Vec<'a, Attribute<'a>>,
}

impl<'a> Parser<'a> {
    pub(super) fn boxed_attributes(
        &self,
        parsed: ParsedAttributes<'a>,
    ) -> Box<'a, ElementAttributes<'a>> {
        self.allocator.boxed(ElementAttributes {
            id: parsed.id,
            classes: parsed.classes,
            values: parsed.values,
        })
    }

    pub(super) fn empty_attributes(&self) -> ParsedAttributes<'a> {
        ParsedAttributes {
            id: None,
            classes: self.allocator.new_vec(),
            values: self.allocator.new_vec(),
        }
    }

    /// Final explicit IDs and repeated keys win; classes accumulate in order.
    /// A malformed block is left entirely literal by every attachment point.
    pub(super) fn parse_attributes(
        &self,
        source: &'a str,
        key_values: bool,
    ) -> Option<ParsedAttributes<'a>> {
        if !key_values {
            let mut parsed = self.empty_attributes();
            scan_id_classes(source, |name, is_id| {
                if is_id {
                    parsed.id = Some(name);
                } else {
                    parsed.classes.push(name);
                }
            })?;
            return Some(parsed);
        }
        let mut parsed = self.empty_attributes();
        let bytes = source.as_bytes();
        let mut pos = 0;
        let mut found = false;
        let mut token_count = 0;
        while pos < bytes.len() {
            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos == bytes.len() {
                break;
            }
            if token_count == 64 {
                return None;
            }
            token_count += 1;
            found = true;
            if matches!(bytes[pos], b'#' | b'.') {
                let kind = bytes[pos];
                pos += 1;
                let start = pos;
                while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }
                let name = &source[start..pos];
                if !valid_shorthand(name) {
                    return None;
                }
                if kind == b'#' {
                    parsed.id = Some(name);
                } else {
                    parsed.classes.push(name);
                }
                continue;
            }
            let start = pos;
            while pos < bytes.len() && valid_name_byte(bytes[pos]) {
                pos += 1;
            }
            if start == pos || bytes.get(pos) != Some(&b'=') {
                return None;
            }
            let raw_name = &source[start..pos];
            if !raw_name.as_bytes()[0].is_ascii_alphabetic() {
                return None;
            }
            let name = if raw_name.bytes().any(|byte| byte.is_ascii_uppercase()) {
                self.allocator.alloc_str(&raw_name.to_ascii_lowercase())
            } else {
                raw_name
            };
            pos += 1;
            let value = if matches!(bytes.get(pos), Some(b'\'' | b'"')) {
                let quote = bytes[pos];
                pos += 1;
                let start = pos;
                let mut tail = start;
                let mut unescaped: Option<ArenaString<'a>> = None;
                while pos < bytes.len() && bytes[pos] != quote {
                    if bytes[pos] == b'\\' {
                        let next = *bytes.get(pos + 1)?;
                        if next != quote && next != b'\\' {
                            return None;
                        }
                        let output = unescaped.get_or_insert_with(|| self.allocator.new_string());
                        output.push_str(&source[tail..pos]);
                        output.push(next as char);
                        pos += 2;
                        tail = pos;
                    } else {
                        pos += 1;
                    }
                }
                if bytes.get(pos) != Some(&quote) {
                    return None;
                }
                let value = if let Some(mut output) = unescaped {
                    output.push_str(&source[tail..pos]);
                    output.into_bump_str()
                } else {
                    &source[start..pos]
                };
                pos += 1;
                value
            } else {
                let start = pos;
                while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
                    if matches!(bytes[pos], b'{' | b'}' | b'"' | b'\'' | b'<' | b'>' | b'`') {
                        return None;
                    }
                    pos += 1;
                }
                if start == pos {
                    return None;
                }
                &source[start..pos]
            };
            if pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
                return None;
            }
            if name == "id" {
                if !valid_shorthand(value) {
                    return None;
                }
                parsed.id = Some(value);
            } else if name == "class" {
                for class in value.split_whitespace() {
                    if !valid_shorthand(class) {
                        return None;
                    }
                    parsed.classes.push(class);
                }
            } else if let Some(existing) = parsed.values.iter_mut().find(|item| item.name == name) {
                existing.value = value;
            } else {
                parsed.values.push(Attribute { name, value });
            }
        }
        found.then_some(parsed)
    }

    pub(super) fn parse_attribute_suffix(
        &self,
        content: &'a str,
        end: usize,
        enabled: bool,
        key_values: bool,
    ) -> (Option<ParsedAttributes<'a>>, usize) {
        if !enabled || content.as_bytes().get(end) != Some(&b'{') {
            return (None, end);
        }
        let Some(close) = attribute_close(content.as_bytes(), end + 1) else {
            return (None, end);
        };
        let Some(parsed) = self.parse_attributes(&content[end + 1..close], key_values) else {
            return (None, end);
        };
        (Some(parsed), close + 1)
    }

    pub(super) fn parse_image_attributes(
        &self,
        content: &'a str,
        end: usize,
    ) -> (Option<Box<'a, ElementAttributes<'a>>>, usize) {
        if !(self.options.image_attributes || self.options.extended_attributes)
            || content.as_bytes().get(end) != Some(&b'{')
        {
            return (None, end);
        }
        let (parsed, next) =
            self.parse_attribute_suffix(content, end, true, self.options.extended_attributes);
        (
            parsed.map(|attributes| self.boxed_attributes(attributes)),
            next,
        )
    }
}

fn valid_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}

fn attribute_close(bytes: &[u8], mut pos: usize) -> Option<usize> {
    // A finite suffix window keeps repeated malformed openers linear even
    // when one distant brace lies inside an unterminated quoted value.
    let limit = pos.saturating_add(512).min(bytes.len());
    let mut quote = None;
    while pos < limit {
        let byte = bytes[pos];
        if matches!(byte, b'\n' | b'\r') {
            return None;
        }
        if let Some(delimiter) = quote {
            if byte == b'\\' {
                pos += 2;
                continue;
            }
            if byte == delimiter {
                quote = None;
            }
        } else if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == b'}' {
            return Some(pos);
        }
        pos += 1;
    }
    None
}

fn valid_shorthand(name: &str) -> bool {
    !name.is_empty()
        && !name.chars().any(|ch| {
            ch.is_control() || matches!(ch, '"' | '\'' | '<' | '>' | '=' | '{' | '}' | '\\')
        })
}
