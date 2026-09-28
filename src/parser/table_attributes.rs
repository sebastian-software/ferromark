//! Opt-in table metadata, kept separate from GFM row recognition.

use crate::allocator::Box;
use crate::ast::TableAttributes;

use super::{Parser, attributes::caption_line};
use crate::parser::error::ParseResult;

impl<'a> Parser<'a> {
    pub(super) fn is_table_attributes_line(&self, position: usize) -> bool {
        caption_line(self.line_at(position), true).is_some_and(|line| {
            // A brace-free colon line in the middle of a table is still a
            // data row, including when its only pipe is escaped. A plain
            // caption must terminate the table; an explicit attribute block
            // is unambiguous even before another row.
            let (_, next) = self.line_and_next(position);
            line.tokens.is_some()
                || (!has_unescaped_pipe(line.content)
                    && (next >= self.source.len()
                        || self.line_at(next).trim_matches([' ', '\t']).is_empty()))
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

        let (id, classes) = if let Some(tokens) = line.tokens {
            let Some(parsed) = self.parse_id_classes(tokens) else {
                return Ok(None);
            };
            parsed
        } else {
            (None, self.allocator.new_vec())
        };
        let caption = self.parse_inline_block(line.content, position + line.content_offset)?;
        self.position = next_position;
        Ok(Some(self.allocator.boxed(TableAttributes {
            id,
            classes,
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
