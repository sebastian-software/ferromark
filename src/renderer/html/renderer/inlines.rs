//! Inline-level HTML visitor helpers.
//!
//! Inline nodes primarily write escaped text or small tags. This module also owns link
//! and image URL sanitization so URL escaping and Markdown link conversion happen in a
//! single place.

use crate::ast::{
    Break, Delete, Emphasis, Highlight, Image, InlineCode, InlineMath, InlineSpan, Insertion, Link,
    Strong, Subscript, Superscript, Text,
};

use super::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::renderer::html::renderer) fn render_text(&mut self, text: &Text<'_>) {
        self.write_inline_text(text.value);
    }

    pub(in crate::renderer::html::renderer) fn render_emphasis(&mut self, emphasis: &Emphasis<'_>) {
        self.write("<em>");
        self.render_inline_children(&emphasis.children);
        self.write("</em>");
    }

    pub(in crate::renderer::html::renderer) fn render_strong(&mut self, strong: &Strong<'_>) {
        self.write("<strong>");
        self.render_inline_children(&strong.children);
        self.write("</strong>");
    }

    pub(in crate::renderer::html::renderer) fn render_inline_code(
        &mut self,
        inline_code: &InlineCode<'_>,
    ) {
        self.write("<code>");
        self.write_escaped(inline_code.value);
        self.write("</code>");
    }

    pub(in crate::renderer::html::renderer) fn render_inline_math(
        &mut self,
        inline_math: &InlineMath<'_>,
    ) {
        self.write("<span class=\"ox-math ox-math-inline\" data-ox-tex=\"");
        self.write_attribute_escaped(inline_math.value);
        self.write("\"><math><mtext>");
        self.write_escaped(inline_math.value);
        self.write("</mtext></math></span>");
    }

    pub(in crate::renderer::html::renderer) fn render_break(&mut self, _break_node: &Break) {
        self.output.push_str(self.options.hard_break());
    }

    pub(in crate::renderer::html::renderer) fn render_link(&mut self, link: &Link<'_>) {
        self.write_link_open(link);
        // Suppress URL auto-linking inside the anchor — children text nodes
        // may contain literal URLs that we must not wrap in a nested <a>.
        let prev_in_link = self.in_link;
        self.in_link = true;
        self.render_inline_children(&link.children);
        self.in_link = prev_in_link;
        self.write("</a>");
    }

    pub(in crate::renderer::html::renderer) fn render_image(&mut self, image: &Image<'_>) {
        self.write("<img src=\"");
        let converted_url = if self.options.convert_md_links {
            self.convert_markdown_url(image.url)
        } else {
            None
        };
        let src = self.sanitized_url(converted_url.as_deref().unwrap_or(image.url), "");
        self.write_url_escaped(src);
        self.write("\" alt=\"");
        self.write_escaped(image.alt);
        self.write("\"");
        if let Some(title) = image.title {
            self.write(" title=\"");
            self.write_escaped(title);
            self.write("\"");
        }
        if let Some(attributes) = &image.attributes {
            if let Some(id) = attributes.id {
                self.write_explicit_element_id(id);
            }
            if !attributes.classes.is_empty() {
                self.write(" class=\"");
                for (index, class_name) in attributes.classes.iter().enumerate() {
                    if index > 0 {
                        self.write(" ");
                    }
                    self.write_attribute_escaped(class_name);
                }
                self.write("\"");
            }
        }
        if let Some(attributes) = &image.attributes {
            if image.title.is_some() {
                self.write_authored_attributes(
                    &attributes.values,
                    &["src", "alt", "title", "id", "class"],
                );
            } else {
                self.write_authored_attributes(&attributes.values, &["src", "alt", "id", "class"]);
            }
        }
        if self.options.xhtml {
            self.write(" />");
        } else {
            self.write(">");
        }
    }

    pub(in crate::renderer::html::renderer) fn render_span(&mut self, span: &InlineSpan<'_>) {
        self.write_span_open(span);
        self.render_inline_children(&span.children);
        self.write("</span>");
    }

    pub(in crate::renderer::html::renderer) fn write_span_open(&mut self, span: &InlineSpan<'_>) {
        self.write("<span");
        if let Some(id) = span.id {
            self.write_explicit_element_id(id);
        }
        if !span.classes.is_empty() {
            self.write(" class=\"");
            for (index, class_name) in span.classes.iter().enumerate() {
                if index > 0 {
                    self.write(" ");
                }
                self.write_attribute_escaped(class_name);
            }
            self.write("\"");
        }
        self.write_authored_attributes(&span.attributes, &["id", "class"]);
        self.write(">");
    }

    /// Visits highlighted text and its children.
    pub(in crate::renderer::html::renderer) fn render_highlight(
        &mut self,
        highlight: &Highlight<'_>,
    ) {
        self.write("<mark>");
        self.render_inline_children(&highlight.children);
        self.write("</mark>");
    }

    pub(in crate::renderer::html::renderer) fn render_delete(&mut self, delete: &Delete<'_>) {
        self.write("<del>");
        self.render_inline_children(&delete.children);
        self.write("</del>");
    }

    pub(in crate::renderer::html::renderer) fn render_insertion(
        &mut self,
        insertion: &Insertion<'_>,
    ) {
        self.write("<ins>");
        self.render_inline_children(&insertion.children);
        self.write("</ins>");
    }

    pub(in crate::renderer::html::renderer) fn render_superscript(
        &mut self,
        superscript: &Superscript<'_>,
    ) {
        self.write("<sup>");
        self.render_inline_children(&superscript.children);
        self.write("</sup>");
    }

    pub(in crate::renderer::html::renderer) fn render_subscript(
        &mut self,
        subscript: &Subscript<'_>,
    ) {
        self.write("<sub>");
        self.render_inline_children(&subscript.children);
        self.write("</sub>");
    }
}
