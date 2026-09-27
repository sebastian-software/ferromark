//! Optional image captions, attached only to standalone image paragraphs.

use crate::allocator::Vec;
use crate::ast::{Figure, Node, Span};

use super::Parser;
use crate::parser::error::ParseResult;

struct CaptionLine<'a> {
    content: &'a str,
    content_offset: usize,
    id: Option<&'a str>,
    classes: Vec<'a, &'a str>,
}

impl<'a> Parser<'a> {
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
        self.position = next;
        Ok(Node::Figure(self.allocator.boxed(Figure {
            content: image,
            caption,
            id: parsed.id,
            classes: parsed.classes,
            span: Span::new(start as u32, next as u32),
        })))
    }

    fn parse_image_caption_line(&self, line: &'a str) -> Option<CaptionLine<'a>> {
        let trimmed = line.trim_start_matches(' ');
        let indent = line.len() - trimmed.len();
        if indent > 3 {
            return None;
        }
        let rest = trimmed.strip_prefix(':')?;
        if !rest.starts_with([' ', '\t']) {
            return None;
        }
        let content = rest.trim_start_matches([' ', '\t']);
        let offset = line.len() - content.len();
        let content = content.trim_end_matches([' ', '\t']);
        if content.is_empty() {
            return None;
        }
        if let Some(without_close) = content.strip_suffix('}')
            && let Some(open) = without_close.rfind('{')
            && (open == 0 || without_close[..open].ends_with([' ', '\t']))
        {
            let caption = without_close[..open].trim_end_matches([' ', '\t']);
            if caption.is_empty() {
                return None;
            }
            let (id, classes) = self.parse_id_classes(&without_close[open + 1..])?;
            return Some(CaptionLine {
                content: caption,
                content_offset: offset,
                id,
                classes,
            });
        }
        Some(CaptionLine {
            content,
            content_offset: offset,
            id: None,
            classes: self.allocator.new_vec(),
        })
    }
}
