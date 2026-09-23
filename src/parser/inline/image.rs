//! Inline images and their plain-text alternative content.

use crate::allocator::Vec;
use crate::ast::{Image, Node, Span};

use super::Parser;
use crate::parser::error::ParseResult;
use crate::parser::short_scan;

impl<'a> Parser<'a> {
    pub(super) fn parse_image(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let bytes = content.as_bytes();
        if *pos + 1 >= content.len() || bytes[*pos + 1] != b'[' {
            Self::push_text(children, "!", offset + *pos, offset + *pos + 1);
            *pos += 1;
            return Ok(());
        }

        let image_start = *pos;

        // Nothing can close this bracket, so skip the balanced scan that
        // would walk to the end of the content to reach the same verdict.
        // Same fallback as below, reached without the walk.
        if !self.has_closer_from(content, *pos + 2, b']') {
            Self::push_text(
                children,
                "![",
                offset + image_start,
                offset + image_start + 2,
            );
            *pos = image_start + 2;
            return Ok(());
        }

        *pos += 2;
        let alt_start = *pos;
        *pos = self.scan_balanced_matched(content, *pos).0;

        if *pos < content.len() && bytes[*pos] == b']' {
            let close = *pos;
            let raw_alt = &content[alt_start..close];
            let alt = self.flatten_image_alt(raw_alt, offset + alt_start)?;

            if bytes.get(close + 1) == Some(&b'(')
                && let Some(target) = self.parse_link_target(content, close + 1)
            {
                children.push(Node::Image(self.allocator.boxed(Image {
                    url: target.url,
                    alt,
                    title: target.title,
                    span: Span::new((offset + image_start) as u32, (offset + target.end) as u32),
                })));
                *pos = target.end;
                return Ok(());
            }

            let mut well_formed_reference = false;
            if self.options.allow_link_refs
                && bytes.get(close + 1) == Some(&b'[')
                && self.has_closer_from(content, close + 2, b']')
            {
                let label_start = close + 2;
                let (label_end, _) = self.scan_balanced_matched(content, label_start);
                if label_end < content.len() && bytes[label_end] == b']' {
                    well_formed_reference = true;
                    let raw_label = &content[label_start..label_end];
                    #[allow(
                        clippy::disallowed_methods,
                        reason = "blank-label checks must agree with `normalize_reference_label`"
                    )]
                    let key = if raw_label.trim().is_empty() {
                        raw_alt
                    } else {
                        raw_label
                    };
                    if let Some(reference) = self.lookup_reference(key) {
                        children.push(Node::Image(self.allocator.boxed(Image {
                            url: reference.url,
                            alt,
                            title: reference.title,
                            span: Span::new(
                                (offset + image_start) as u32,
                                (offset + label_end + 1) as u32,
                            ),
                        })));
                        *pos = label_end + 1;
                        return Ok(());
                    }
                }
            }

            if !well_formed_reference && let Some(reference) = self.lookup_reference(raw_alt) {
                children.push(Node::Image(self.allocator.boxed(Image {
                    url: reference.url,
                    alt,
                    title: reference.title,
                    span: Span::new((offset + image_start) as u32, (offset + close + 1) as u32),
                })));
                *pos = close + 1;
                return Ok(());
            }
        }

        // No valid inline image here: `![` is literal text and the rest of
        // the bracketed run is re-parsed for other inline markup.
        Self::push_text(
            children,
            "![",
            offset + image_start,
            offset + image_start + 2,
        );
        *pos = image_start + 2;
        Ok(())
    }

    /// Builds an image's `alt` attribute: the bracket text parsed as
    /// inlines and flattened to plain text (links contribute their text,
    /// code its literal content). Plain text stays zero-copy.
    fn flatten_image_alt(&self, raw: &'a str, offset: usize) -> ParseResult<&'a str> {
        // Alt text is usually a handful of words, so these three probes are
        // short-slice searches (see `short_scan`) rather than vector ones.
        if short_scan::find3(b'[', b'*', b'_', raw.as_bytes()).is_none()
            && short_scan::find3(b'`', b'\\', b'&', raw.as_bytes()).is_none()
            && short_scan::find(b'<', raw.as_bytes()).is_none()
        {
            return Ok(raw);
        }
        let nodes = self.parse_inline(raw, offset)?;
        let mut out = self.allocator.new_string();
        flatten_inline_text(&nodes, &mut out);
        Ok(out.into_bump_str())
    }
}

/// Flattens inline nodes to their plain-text content (image `alt` rules).
fn flatten_inline_text(nodes: &[Node<'_>], out: &mut crate::allocator::String<'_>) {
    for node in nodes {
        match node {
            Node::Text(n) => out.push_str(n.value),
            Node::InlineCode(n) => out.push_str(n.value),
            Node::Emphasis(n) => flatten_inline_text(&n.children, out),
            Node::Strong(n) => flatten_inline_text(&n.children, out),
            Node::Highlight(n) => flatten_inline_text(&n.children, out),
            Node::Delete(n) => flatten_inline_text(&n.children, out),
            Node::Superscript(n) => flatten_inline_text(&n.children, out),
            Node::Subscript(n) => flatten_inline_text(&n.children, out),
            Node::Link(n) => flatten_inline_text(&n.children, out),
            Node::Image(n) => out.push_str(n.alt),
            Node::Break(_) => out.push('\n'),
            _ => {}
        }
    }
}
