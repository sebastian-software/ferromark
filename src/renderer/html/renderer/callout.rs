//! Rendering support for GitHub-style callout block quotes.
//!
//! Callouts are encoded as normal block quotes in the AST. These helpers detect the
//! marker in the first paragraph, strip it from the body, and emit the themed wrapper
//! while leaving non-callout block quotes on the regular rendering path.

use crate::ast::{BlockQuote, Node, Paragraph};

use super::HtmlRenderer;
use crate::callout::detect_callout;

impl HtmlRenderer {
    fn render_paragraph_with_skipped_text_prefix<'a>(
        &mut self,
        paragraph: &Paragraph<'a>,
        mut skip_chars: usize,
    ) {
        let paragraph_start = self.output.len();
        self.write("<p");
        self.write_source_span_attr(paragraph.span);
        self.write(">");
        let body_start = self.output.len();

        // The former temporary renderer had no initialized autolink index,
        // so callout body text was escaped without bare-URL rewriting. Keep
        // that observable behavior while writing into the existing buffer.
        let autolink_index = self.autolink_index.take();
        // Whitespace-only text between the marker and the body (the parser
        // may emit the separating newline as its own Text node) is part of
        // the marker line, not body content.
        let mut before_body = true;

        for child in &paragraph.children {
            match child {
                Node::Text(text) if skip_chars > 0 || before_body => {
                    let mut value = text.value;
                    if skip_chars > 0 {
                        if skip_chars >= value.len() {
                            skip_chars -= value.len();
                            continue;
                        }
                        value = &value[skip_chars..];
                        skip_chars = 0;
                    }
                    value = value.trim_start();
                    if value.is_empty() {
                        continue;
                    }
                    before_body = false;
                    self.write_inline_text(value);
                }
                _ => {
                    before_body = false;
                    // Inline children of the marker paragraph, including raw
                    // inline HTML: the block path would append a line break
                    // after every HTML fragment.
                    self.visit_inline_node(child);
                }
            }
        }
        self.autolink_index = autolink_index;

        if self.output[body_start..].trim().is_empty() {
            self.output.truncate(paragraph_start);
        } else {
            self.write("</p>\n");
        }
    }

    pub(in crate::renderer::html::renderer) fn render_callout_block_quote<'a>(
        &mut self,
        block_quote: &BlockQuote<'a>,
    ) -> bool {
        let Some(Node::Paragraph(first_paragraph)) = block_quote.children.first() else {
            return false;
        };
        let Some((kind, consumed_chars)) = detect_callout(first_paragraph) else {
            return false;
        };

        self.write("<blockquote class=\"ox-callout ox-callout--");
        self.write(kind.class_name());
        self.write("\"");
        self.write_source_span_attr(block_quote.span);
        self.write(">\n");
        self.write("<p class=\"ox-callout-title\">");
        self.write(kind.label());
        self.write("</p>\n");

        self.render_paragraph_with_skipped_text_prefix(first_paragraph, consumed_chars);

        for child in block_quote.children.iter().skip(1) {
            self.render_node(child);
        }

        self.write("</blockquote>\n");
        true
    }
}
