//! Optional image captions, attached only to standalone image paragraphs.

use crate::allocator::Vec;
use crate::ast::{Attribute, ElementAttributes, Figure, Node, Span};

use super::{Parser, attributes::caption_line};
use crate::parser::error::ParseResult;

struct CaptionLine<'a> {
    content: &'a str,
    content_offset: usize,
    id: Option<&'a str>,
    classes: Vec<'a, &'a str>,
    values: Vec<'a, Attribute<'a>>,
}

impl<'a> Parser<'a> {
    /// Definition-list parsing runs before paragraph parsing. Look through a
    /// possible multiline image only when both extensions are enabled, so an
    /// eligible caption wins without reparsing ordinary image paragraphs.
    pub(super) fn image_caption_precedes_definition_list(
        &self,
        start: usize,
        mut position: usize,
    ) -> ParseResult<bool> {
        while position < self.source.len() {
            let (line, next) = self.line_and_next(position);
            if self.parse_image_caption_line(line).is_some() {
                return self.image_caption_interrupts_paragraph(start, position, position);
            }
            if line.trim_matches([' ', '\t']).is_empty() {
                if next >= self.source.len() {
                    return Ok(false);
                }
                let (following, _) = self.line_and_next(next);
                return if self.parse_image_caption_line(following).is_some() {
                    self.image_caption_interrupts_paragraph(start, position, next)
                } else {
                    Ok(false)
                };
            }
            position = next;
        }
        Ok(false)
    }

    /// Before ending a paragraph on an immediate `: Caption` line, verify
    /// that the paragraph is exactly one image. Other prose keeps CommonMark
    /// lazy continuation behavior even with the extension enabled.
    pub(super) fn image_caption_interrupts_paragraph(
        &self,
        start: usize,
        content_end: usize,
        caption_line_start: usize,
    ) -> ParseResult<bool> {
        if !self.options.image_captions
            || self
                .parse_image_caption_line(self.line_at(caption_line_start))
                .is_none()
        {
            return Ok(false);
        }
        let (content, leading) =
            super::whitespace::trim_with_leading(&self.source[start..content_end]);
        if !content.starts_with("![") {
            return Ok(false);
        }
        let children = self.parse_inline_block(content, start + leading)?;
        Ok(children.len() == 1 && matches!(children.first(), Some(Node::Image(_))))
    }

    /// Look at the following line, optionally after one blank line. Failure
    /// leaves the cursor untouched so the line remains ordinary Markdown.
    pub(super) fn attach_image_caption(
        &mut self,
        image: Node<'a>,
        start: usize,
    ) -> ParseResult<Node<'a>> {
        let mut position = self.position;
        if position >= self.source.len() {
            return Ok(image);
        }
        let (first, after_first) = self.line_and_next(position);
        if first.trim_matches([' ', '\t']).is_empty() {
            position = after_first;
        }
        if position >= self.source.len() {
            return Ok(image);
        }
        let (line, next) = self.line_and_next(position);
        let Some(parsed) = self.parse_image_caption_line(line) else {
            return Ok(image);
        };
        let caption = self.parse_inline_block(parsed.content, position + parsed.content_offset)?;
        if caption.is_empty() {
            return Ok(image);
        }
        let attributes = (parsed.id.is_some()
            || !parsed.classes.is_empty()
            || !parsed.values.is_empty())
        .then(|| {
            self.allocator.boxed(ElementAttributes {
                id: parsed.id,
                classes: parsed.classes,
                values: parsed.values,
            })
        });
        self.position = next;
        Ok(Node::Figure(self.allocator.boxed(Figure {
            content: image,
            caption,
            attributes,
            span: Span::new(start as u32, next as u32),
        })))
    }

    fn parse_image_caption_line(&self, line: &'a str) -> Option<CaptionLine<'a>> {
        let parsed = caption_line(line, false)?;
        let attributes = if let Some(tokens) = parsed.tokens {
            self.parse_attributes(tokens, self.options.extended_attributes)?
        } else {
            self.empty_attributes()
        };
        Some(CaptionLine {
            content: parsed.content,
            content_offset: parsed.content_offset,
            id: attributes.id,
            classes: attributes.classes,
            values: attributes.values,
        })
    }
}
