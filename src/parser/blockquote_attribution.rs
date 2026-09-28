//! Optional visible attributions attached to block quotes.

use crate::allocator::Box;
use crate::ast::{ElementAttributes, Figure, Node, Span};

use super::Parser;
use super::attributes::caption_line;
use crate::parser::error::ParseResult;

pub(super) struct AttributionLine<'a> {
    pub(super) content: &'a str,
    pub(super) content_offset: usize,
    pub(super) attributes: Option<Box<'a, ElementAttributes<'a>>>,
}

impl<'a> Parser<'a> {
    /// Attach an immediate source line or one source line after one blank line.
    /// If no nonempty attribution is found, preserve the cursor for normal parsing.
    pub(super) fn attach_blockquote_attribution(
        &mut self,
        quote: Node<'a>,
        start: usize,
        immediate: Option<AttributionLine<'a>>,
        is_callout: bool,
    ) -> ParseResult<Node<'a>> {
        if !self.options.blockquote_attributions
            || is_callout
            || matches!(&quote, Node::BlockQuote(block) if block.children.is_empty())
        {
            return Ok(quote);
        }

        let mut position = self.position;
        let parsed = if let Some(parsed) = immediate {
            parsed
        } else {
            if position >= self.source.len() {
                return Ok(quote);
            }
            let (first, after_first) = self.line_and_next(position);
            if first.trim_matches([' ', '\t']).is_empty() {
                position = after_first;
            }
            if position >= self.source.len() {
                return Ok(quote);
            }
            let (line, _) = self.line_and_next(position);
            let Some(parsed) = self.parse_blockquote_attribution_line(line) else {
                return Ok(quote);
            };
            parsed
        };
        let (_, next) = self.line_and_next(position);
        let caption = self.parse_inline_block(parsed.content, position + parsed.content_offset)?;
        if caption.is_empty() {
            return Ok(quote);
        }

        self.position = next;
        Ok(Node::Figure(self.allocator.boxed(Figure {
            content: quote,
            caption,
            attributes: parsed.attributes,
            span: Span::new(start as u32, next as u32),
        })))
    }

    pub(super) fn parse_blockquote_attribution_line(
        &self,
        line: &'a str,
    ) -> Option<AttributionLine<'a>> {
        let candidate = caption_line(line, false)?;
        let attributes =
            if let Some(tokens) = candidate.tokens {
                Some(self.boxed_attributes(
                    self.parse_attributes(tokens, self.options.extended_attributes)?,
                ))
            } else {
                None
            };
        Some(AttributionLine {
            content: candidate.content,
            content_offset: candidate.content_offset,
            attributes,
        })
    }
}
