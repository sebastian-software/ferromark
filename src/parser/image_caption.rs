//! Optional image captions, attached only to standalone image paragraphs.

use crate::allocator::{Box, Vec};
use crate::ast::{Attribute, ElementAttributes, Figure, Node, Span};

use super::{Parser, attributes::caption_line};
use crate::parser::error::ParseResult;

pub(super) struct CaptionLine<'a> {
    content: &'a str,
    content_offset: usize,
    id: Option<&'a str>,
    classes: Vec<'a, &'a str>,
    values: Vec<'a, Attribute<'a>>,
}

pub(super) struct ImmediateImageCaption<'a> {
    pub(super) image: Node<'a>,
    caption_line: CaptionLine<'a>,
    line_start: usize,
    line_next: usize,
}

impl<'a> Parser<'a> {
    /// Before ending a paragraph on an immediate `: Caption` line, verify
    /// that the paragraph is exactly one image. Other prose keeps CommonMark
    /// lazy continuation behavior even with the extension enabled.
    ///
    /// Returns the already-parsed image node and caption line so the caller
    /// does not re-parse both immediately after breaking out of the paragraph.
    #[inline(never)]
    pub(super) fn probe_immediate_image_caption(
        &self,
        start: usize,
        content_end: usize,
        caption_line_start: usize,
    ) -> ParseResult<Option<Box<'a, Option<ImmediateImageCaption<'a>>>>> {
        if !self.options.image_captions {
            return Ok(None);
        }
        let (content, leading) =
            super::whitespace::trim_with_leading(&self.source[start..content_end]);
        if !content.starts_with("![") {
            return Ok(None);
        }
        let (line, line_next) = self.line_and_next(caption_line_start);
        let Some(caption_line) = self.parse_image_caption_line(line) else {
            return Ok(None);
        };
        let mut children = self.parse_inline_block(content, start + leading)?;
        if children.len() == 1
            && matches!(children.first(), Some(Node::Image(_)))
            && let Some(image) = children.pop()
        {
            return Ok(Some(self.allocator.boxed(Some(ImmediateImageCaption {
                image,
                caption_line,
                line_start: caption_line_start,
                line_next,
            }))));
        }
        Ok(None)
    }

    /// Attaches a caption already probed at the paragraph boundary without
    /// reparsing the caption line or its attributes.
    pub(super) fn attach_immediate_image_caption(
        &mut self,
        mut immediate: Box<'a, Option<ImmediateImageCaption<'a>>>,
        start: usize,
    ) -> ParseResult<Option<Node<'a>>> {
        let Some(immediate) = immediate.take() else {
            return Ok(None);
        };
        self.attach_parsed_image_caption(
            immediate.image,
            start,
            immediate.caption_line,
            immediate.line_start,
            immediate.line_next,
        )
        .map(Some)
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
        self.attach_parsed_image_caption(image, start, parsed, position, next)
    }

    fn attach_parsed_image_caption(
        &mut self,
        image: Node<'a>,
        start: usize,
        parsed: CaptionLine<'a>,
        position: usize,
        next: usize,
    ) -> ParseResult<Node<'a>> {
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
