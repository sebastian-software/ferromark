//! Opt-in table metadata, kept separate from GFM row recognition.

use crate::allocator::Box;
use crate::ast::TableAttributes;

use super::Parser;
use crate::parser::error::ParseResult;

struct AttributeLine<'a> {
    caption: &'a str,
    caption_offset: usize,
    tokens: Option<&'a str>,
}

impl<'a> Parser<'a> {
    pub(super) fn is_table_attributes_line(&self, position: usize) -> bool {
        attribute_line(self.line_at(position)).is_some_and(|line| {
            line.tokens.is_none_or(|tokens| {
                self.parse_attributes(tokens, self.options.extended_attributes)
                    .is_some()
            })
        })
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
        let (blank_candidate, after_blank) = self.line_and_next(position);
        if blank_candidate.trim_matches([' ', '\t']).is_empty() {
            position = after_blank;
        }
        position = self.skip_line_comments_from(position);
        if position >= self.source.len() {
            return Ok(None);
        }
        let (raw_line, next_position) = self.line_and_next(position);
        let Some(line) = attribute_line(raw_line) else {
            return Ok(None);
        };

        let attributes = if let Some(tokens) = line.tokens {
            let Some(parsed) = self.parse_attributes(tokens, self.options.extended_attributes)
            else {
                return Ok(None);
            };
            parsed
        } else {
            self.empty_attributes()
        };
        let caption = self.parse_inline_block(line.caption, position + line.caption_offset)?;
        self.position = next_position;
        Ok(Some(self.allocator.boxed(TableAttributes {
            id: attributes.id,
            classes: attributes.classes,
            attributes: attributes.values,
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
    if !content.ends_with('}') {
        if content.contains(['{', '}']) {
            return None;
        }
        return (!content.is_empty()).then_some(AttributeLine {
            caption: content,
            caption_offset,
            tokens: None,
        });
    }
    let without_close = content.strip_suffix('}')?;
    let open = without_close.rfind('{')?;
    let caption = &content[..open];
    if !caption.is_empty() && !caption.ends_with([' ', '\t']) {
        return None;
    }
    let tokens = &without_close[open + 1..];
    Some(AttributeLine {
        caption: caption.trim_end_matches([' ', '\t']),
        caption_offset,
        tokens: Some(tokens),
    })
}
