//! Opt-in table metadata, kept separate from GFM row recognition.

use crate::allocator::Box;
use crate::ast::TableAttributes;

use super::{Parser, attributes::caption_line};
use crate::parser::error::ParseResult;

impl<'a> Parser<'a> {
    pub(super) fn is_table_attributes_line(&self, position: usize) -> bool {
        caption_line(self.line_at(position), true).is_some_and(|line| {
            (line.tokens.is_some() || !has_unescaped_pipe(line.content))
                && line.tokens.is_none_or(|tokens| {
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
        let Some(line) = caption_line(raw_line, true) else {
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
        let caption = self.parse_inline_block(line.content, position + line.content_offset)?;
        self.position = next_position;
        Ok(Some(self.allocator.boxed(TableAttributes {
            id: attributes.id,
            classes: attributes.classes,
            attributes: attributes.values,
            caption,
        })))
    }
}

fn has_unescaped_pipe(source: &str) -> bool {
    let mut escaped = false;
    for byte in source.bytes() {
        if byte == b'\\' {
            escaped = !escaped;
            continue;
        }
        if byte == b'|' && !escaped {
            return true;
        }
        escaped = false;
    }
    false
}
