//! Opt-in table metadata, kept separate from GFM row recognition.

use crate::allocator::Box;
use crate::ast::TableAttributes;

use super::{
    Parser,
    attributes::{ParsedAttributes, caption_line},
};
use crate::parser::error::ParseResult;

pub(super) struct ProbedTableAttributes<'a> {
    position: usize,
    next_position: usize,
    content: &'a str,
    content_offset: usize,
    attributes: ParsedAttributes<'a>,
}

impl<'a> Parser<'a> {
    #[inline(never)]
    pub(super) fn probe_table_attributes_line(
        &self,
        position: usize,
    ) -> Option<Box<'a, Option<ProbedTableAttributes<'a>>>> {
        let (raw_line, next_position) = self.line_and_next(position);
        let line = caption_line(raw_line, true)?;
        // A brace-free colon line in the middle of a table is still a
        // data row, including when its only pipe is escaped. A plain
        // caption must terminate the table; an explicit attribute block
        // is unambiguous even before another row.
        let valid_boundary = line.tokens.is_some()
            || (!has_unescaped_pipe(line.content)
                && (next_position >= self.source.len()
                    || self
                        .line_at(next_position)
                        .trim_matches([' ', '\t'])
                        .is_empty()));
        if !valid_boundary {
            return None;
        }
        let attributes = if let Some(tokens) = line.tokens {
            self.parse_attributes(tokens, self.options.extended_attributes)?
        } else {
            self.empty_attributes()
        };
        Some(self.allocator.boxed(Some(ProbedTableAttributes {
            position,
            next_position,
            content: line.content,
            content_offset: line.content_offset,
            attributes,
        })))
    }

    /// Attach one following metadata line, optionally separated by one blank
    /// line. Failed lookahead never consumes text or allocates AST metadata.
    pub(super) fn parse_table_attributes(
        &mut self,
        immediate: Option<Box<'a, Option<ProbedTableAttributes<'a>>>>,
    ) -> ParseResult<Option<Box<'a, TableAttributes<'a>>>> {
        if let Some(mut probed) = immediate
            && let Some(probed) = probed.take()
        {
            let caption =
                self.parse_inline_block(probed.content, probed.position + probed.content_offset)?;
            self.position = probed.next_position;
            return Ok(Some(self.allocator.boxed(TableAttributes {
                id: probed.attributes.id,
                classes: probed.attributes.classes,
                attributes: probed.attributes.values,
                caption,
            })));
        }
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
