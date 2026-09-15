//! Opt-in table metadata, kept separate from GFM row recognition.

use crate::allocator::Box;
use crate::ast::TableAttributes;

use super::Parser;
use super::line_scan::next_line_start;
use crate::parser::error::ParseResult;

struct AttributeLine<'a> {
    caption: &'a str,
    caption_offset: usize,
    tokens: &'a str,
}

impl<'a> Parser<'a> {
    pub(super) fn is_table_attributes_line(&self, position: usize) -> bool {
        attribute_line(self.line_at(position)).is_some()
    }

    /// Attach one following metadata line, optionally separated by one blank
    /// line. Failed lookahead never consumes text or allocates AST metadata.
    pub(super) fn parse_table_attributes(
        &mut self,
    ) -> ParseResult<Option<Box<'a, TableAttributes<'a>>>> {
        if !self.options.table_attributes || self.is_at_end() {
            return Ok(None);
        }
        let mut position = self.position;
        if self.line_at(position).trim_matches([' ', '\t']).is_empty() {
            position = next_line_start(self.source.as_bytes(), position);
        }
        position = self.skip_line_comments_from(position);
        if position >= self.source.len() {
            return Ok(None);
        }
        let Some(line) = attribute_line(self.line_at(position)) else {
            return Ok(None);
        };

        let mut id = None;
        let mut classes = self.allocator.new_vec();
        for token in line.tokens.split_whitespace() {
            if let Some(name) = token.strip_prefix('#') {
                id = Some(name);
            } else if let Some(name) = token.strip_prefix('.') {
                classes.push(name);
            }
        }
        let caption = self.parse_inline_block(line.caption, position + line.caption_offset)?;
        self.position = next_line_start(self.source.as_bytes(), position);
        Ok(Some(self.allocator.boxed(TableAttributes {
            id,
            classes,
            caption,
        })))
    }
}

fn attribute_line(line: &str) -> Option<AttributeLine<'_>> {
    let trimmed_start = line.trim_start_matches(' ');
    let indent = line.len() - trimmed_start.len();
    if indent > 3 {
        return None;
    }
    let after_colon = trimmed_start.strip_prefix(':')?;
    if !after_colon.starts_with([' ', '\t']) {
        return None;
    }
    let content = after_colon.trim_start_matches([' ', '\t']);
    let caption_offset = line.len() - content.len();
    let content = content.trim_end_matches([' ', '\t']);
    let without_close = content.strip_suffix('}')?;
    let open = without_close.rfind('{')?;
    let caption = &content[..open];
    if !caption.is_empty() && !caption.ends_with([' ', '\t']) {
        return None;
    }
    let tokens = &without_close[open + 1..];
    let mut seen_id = false;
    let mut seen_attribute = false;
    for token in tokens.split_whitespace() {
        let name = if let Some(name) = token.strip_prefix('#') {
            if seen_id {
                return None;
            }
            seen_id = true;
            name
        } else {
            token.strip_prefix('.')?
        };
        if name.is_empty()
            || name.chars().any(|ch| {
                ch.is_control() || matches!(ch, '"' | '\'' | '<' | '>' | '=' | '{' | '}' | '\\')
            })
        {
            return None;
        }
        seen_attribute = true;
    }
    seen_attribute.then_some(AttributeLine {
        caption: caption.trim_end_matches([' ', '\t']),
        caption_offset,
        tokens,
    })
}
