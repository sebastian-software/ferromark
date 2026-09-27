//! Shared authored-attribute grammar. HTML name mapping belongs to the renderer.

use crate::allocator::{String as ArenaString, Vec};
use crate::ast::Attribute;

use super::Parser;

pub(super) struct ParsedAttributes<'a> {
    pub id: Option<&'a str>,
    pub classes: Vec<'a, &'a str>,
    pub values: Vec<'a, Attribute<'a>>,
}

impl<'a> Parser<'a> {
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
            let mut found = false;
            for token in source.split_whitespace() {
                let (name, is_id) = if let Some(name) = token.strip_prefix('#') {
                    (name, true)
                } else {
                    (token.strip_prefix('.')?, false)
                };
                if !valid_shorthand(name) {
                    return None;
                }
                if is_id {
                    if parsed.id.replace(name).is_some() {
                        return None;
                    }
                } else {
                    parsed.classes.push(name);
                }
                found = true;
            }
            return found.then_some(parsed);
        }
        let mut parsed = self.empty_attributes();
        let bytes = source.as_bytes();
        let mut pos = 0;
        let mut found = false;
        while pos < bytes.len() {
            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos == bytes.len() {
                break;
            }
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
    ) -> (ParsedAttributes<'a>, usize) {
        if !enabled || content.as_bytes().get(end) != Some(&b'{') {
            return (self.empty_attributes(), end);
        }
        if !self.has_closer_from(content, end + 1, b'}') {
            return (self.empty_attributes(), end);
        }
        let Some(close) = attribute_close(content.as_bytes(), end + 1) else {
            return (self.empty_attributes(), end);
        };
        let Some(parsed) = self.parse_attributes(&content[end + 1..close], key_values) else {
            return (self.empty_attributes(), end);
        };
        (parsed, close + 1)
    }

    pub(super) fn parse_image_attributes(
        &self,
        content: &'a str,
        end: usize,
    ) -> (ParsedAttributes<'a>, usize) {
        self.parse_attribute_suffix(
            content,
            end,
            self.options.image_attributes || self.options.extended_attributes,
            self.options.extended_attributes,
        )
    }
}

fn valid_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':')
}

fn attribute_close(bytes: &[u8], mut pos: usize) -> Option<usize> {
    // A finite suffix window keeps repeated malformed openers linear even
    // when one distant brace lies inside an unterminated quoted value.
    let limit = pos.saturating_add(4096).min(bytes.len());
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
