//! Block-level HTML visitor helpers.
//!
//! The trait implementation delegates here for paragraphs, headings, lists, tables,
//! raw HTML, block quotes, and fenced code. Separating block rendering from the trait
//! glue keeps each file small while preserving the visitor behavior exactly.

use crate::ast::{
    AlignKind, BlockQuote, CodeBlock, Heading, Html, List, ListItem, MathBlock, Paragraph, Table,
    TableCell, TableRow, ThematicBreak,
};

use super::super::code_annotations::{normalize_code_block_language, plain_code_block_language};
use super::HtmlRenderer;

/// The bytes a plain fence writes around its language and its body.
const PLAIN_FENCE_MARKUP_LEN: usize =
    "<pre><code class=\"language-".len() + "\">".len() + "</code></pre>\n".len();

/// How a fence's info token reaches the `language-…` class attribute.
///
/// Splitting the decision from the writing keeps the resolved language borrowed
/// from the source — no owned language string is ever built — and lets the
/// caller size its reservation before emitting anything.
enum FenceLanguage<'a> {
    /// The fence carries no usable language; it gets no class attribute.
    None,
    /// Already normalized and free of bytes the escaper rewrites.
    Plain(&'a str),
    /// Normalized, but still has to go through the HTML escaper.
    Escaped(&'a str),
}

impl<'a> FenceLanguage<'a> {
    fn from_escaped(language: Option<&'a str>) -> Self {
        match language {
            Some(language) => Self::Escaped(language),
            None => Self::None,
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Plain(language) | Self::Escaped(language) => language.len(),
        }
    }
}

impl HtmlRenderer {
    pub(in crate::renderer::html::renderer) fn render_paragraph(
        &mut self,
        paragraph: &Paragraph<'_>,
    ) {
        self.output.push_str("<p");
        self.write_source_span_attr(paragraph.span);
        self.output.push('>');
        for child in &paragraph.children {
            self.visit_inline_node(child);
        }
        self.output.push_str("</p>\n");
    }

    pub(in crate::renderer::html::renderer) fn render_heading(&mut self, heading: &Heading<'_>) {
        // Avoid the heading.depth -> &str match per call: heading depth is
        // 1..=6 by construction, and "h%d" is a fixed shape we can splat
        // directly. Saves a branch and a `write` call.
        let depth = super::super::super::map_heading_level(
            heading.depth,
            self.options.heading_level_offset,
        );
        self.output.push_str("<h");
        self.output.push((b'0' + depth) as char);
        if self.options.heading_ids {
            self.output.push_str(" id=\"");
            // Heading ids are slugified: lowercase alnum + '-' separators. None
            // of those bytes need HTML escaping, so the unconditional
            // `write_escaped` pass over the id was pure overhead. We also
            // skip materializing the id as a return-value `String`; it's
            // written straight into `self.output`.
            self.write_heading_id(heading);
            self.output.push('"');
        }
        if !heading.classes.is_empty() {
            self.output.push_str(" class=\"");
            for (index, class_name) in heading.classes.iter().enumerate() {
                if index > 0 {
                    self.output.push(' ');
                }
                self.write_attribute_escaped(class_name);
            }
            self.output.push('"');
        }
        self.write_source_span_attr(heading.span);
        self.output.push('>');
        for child in &heading.children {
            self.visit_inline_node(child);
        }
        self.write_heading_permalink_if_needed(heading);
        self.output.push_str("</h");
        self.output.push((b'0' + depth) as char);
        self.output.push_str(">\n");
    }

    pub(in crate::renderer::html::renderer) fn render_thematic_break(
        &mut self,
        thematic_break: &ThematicBreak,
    ) {
        self.write("<hr");
        self.write_source_span_attr(thematic_break.span);
        if self.options.xhtml {
            self.write(" />\n");
        } else {
            self.write(">\n");
        }
    }

    pub(in crate::renderer::html::renderer) fn render_block_quote(
        &mut self,
        block_quote: &BlockQuote<'_>,
    ) {
        if self.options.callouts && self.render_callout_block_quote(block_quote) {
            return;
        }

        self.write("<blockquote");
        self.write_source_span_attr(block_quote.span);
        self.write(">\n");
        for child in &block_quote.children {
            self.render_node(child);
        }
        self.write("</blockquote>\n");
    }

    pub(in crate::renderer::html::renderer) fn render_list(&mut self, list: &List<'_>) {
        if list.ordered {
            if let Some(start) = list.start {
                if start != 1 {
                    self.write("<ol start=\"");
                    self.write_display(start);
                    self.write("\"");
                } else {
                    self.write("<ol");
                }
            } else {
                self.write("<ol");
            }
        } else {
            self.write("<ul");
        }
        self.write_source_span_attr(list.span);
        self.write(">\n");

        // A tight list renders item paragraphs without <p> wrappers
        // (CommonMark "Lists": loose lists are the ones whose items are
        // separated by blank lines or contain multiple blocks).
        let tight = !list.spread;
        for child in &list.children {
            self.render_list_item_with_tightness(child, tight);
        }

        if list.ordered {
            self.write("</ol>\n");
        } else {
            self.write("</ul>\n");
        }
    }

    pub(in crate::renderer::html::renderer) fn render_list_item(
        &mut self,
        list_item: &ListItem<'_>,
    ) {
        self.render_list_item_with_tightness(list_item, false);
    }

    pub(in crate::renderer::html::renderer) fn render_list_item_with_tightness(
        &mut self,
        list_item: &ListItem<'_>,
        tight: bool,
    ) {
        self.write("<li");
        self.write_source_span_attr(list_item.span);
        self.write(">");

        if let Some(checked) = list_item.checked {
            if checked {
                self.write("<input type=\"checkbox\" checked disabled> ");
            } else {
                self.write("<input type=\"checkbox\" disabled> ");
            }
        }

        for child in &list_item.children {
            if tight {
                if let crate::ast::Node::Paragraph(paragraph) = child {
                    for inline in &paragraph.children {
                        self.visit_inline_node(inline);
                    }
                    continue;
                }
                // Keep nested blocks on their own lines even when the
                // preceding paragraph was rendered inline.
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.write("\n");
                }
            }
            self.render_node(child);
        }

        self.write("</li>\n");
    }

    pub(in crate::renderer::html::renderer) fn render_code_block(
        &mut self,
        code_block: &CodeBlock<'_>,
    ) {
        if !self.options.code_annotations || !self.options.code_fence_metadata {
            self.write_plain_code_block(code_block);
            return;
        }

        self.code_block_index += 1;
        let state = self.build_code_block_state(code_block, self.code_block_index);
        self.write_code_block_state(code_block, &state, None);
    }

    pub(in crate::renderer::html::renderer) fn write_code_block_state(
        &mut self,
        code_block: &CodeBlock<'_>,
        state: &super::super::code_annotations::CodeBlockRenderState,
        highlighted_lines: Option<&[String]>,
    ) {
        let block_classes = state.block_classes();

        self.write("<pre");
        if !block_classes.is_empty() {
            self.write(" class=\"");
            self.write(&block_classes.join(" "));
            self.write("\"");
        }
        self.write_source_span_attr(code_block.span);
        if let Some(title) = state.title.as_deref() {
            self.write(" data-code-title=\"");
            self.write_attribute_escaped(title);
            self.write("\"");
        }
        if let Some(prefix) = state.line_link_prefix.as_deref() {
            self.write(" data-line-link-prefix=\"");
            self.write_attribute_escaped(prefix);
            self.write("\"");
        }
        if let Some(start) = state.line_numbers_start {
            self.write(" data-line-numbers=\"true\" data-line-number-start=\"");
            self.write_display(start);
            self.write("\"");
        }
        if let Some(source) = state.copy_source.as_deref() {
            self.write(" data-ox-code-source=\"");
            self.write_attribute_escaped(source);
            self.write("\"");
        }
        self.write("><code");
        if let Some(lang) = state.language.as_deref() {
            self.write(" class=\"language-");
            self.write_escaped(lang);
            self.write("\"");
        }
        self.write(">");
        if state.needs_line_wrappers() {
            self.write_code_lines(state, highlighted_lines);
        } else if let Some(lines) = highlighted_lines {
            self.write_highlighted_lines(lines);
        } else {
            self.write_escaped(code_block.value);
        }
        self.write("</code></pre>\n");
    }

    /// Writes a fence that carries no annotation markup.
    ///
    /// Everything around the language and the body is fixed, so the opening tag
    /// pair and the class attribute name go out as whole literals rather than as
    /// a tag, a delimiter, and an attribute name pushed separately. The source
    /// span is the only optional part, and it is off in every default profile.
    ///
    /// One reservation up front covers the fixed markup, the class, and a body
    /// that needs no escaping, so none of the pushes below has to grow the
    /// buffer in the common case.
    fn write_plain_code_block(&mut self, code_block: &CodeBlock<'_>) {
        let language = self.code_block_language(code_block.lang);
        self.output
            .reserve(PLAIN_FENCE_MARKUP_LEN + language.len() + code_block.value.len());

        if self.options.source_spans {
            self.write("<pre");
            self.write_source_span_attr(code_block.span);
            self.write("><code");
        } else {
            self.write("<pre><code");
        }

        match language {
            FenceLanguage::None => self.write(">"),
            FenceLanguage::Plain(language) => {
                self.write(" class=\"language-");
                self.write(language);
                self.write("\">");
            }
            FenceLanguage::Escaped(language) => {
                self.write(" class=\"language-");
                self.write_escaped(language);
                self.write("\">");
            }
        }

        self.write_escaped(code_block.value);
        self.write("</code></pre>\n");
    }

    /// Resolves a fence's info token into the `language-…` class body.
    ///
    /// A bare language name is the overwhelmingly common info string, and
    /// `plain_code_block_language` proves with one table lookup per byte that
    /// such a token is already its own normalized, escaped form. That skips the
    /// VitePress metadata tokenizer and the escape scan for it; anything the
    /// proof does not cover falls back to the general route, which produces the
    /// same bytes. Either way the result borrows from the source, so resolving
    /// a language never allocates.
    fn code_block_language<'a>(&self, lang: Option<&'a str>) -> FenceLanguage<'a> {
        if !self.options.code_fence_metadata {
            return FenceLanguage::from_escaped(
                lang.map(str::trim).filter(|lang| !lang.is_empty()),
            );
        }

        if let Some(plain) = lang.and_then(plain_code_block_language) {
            return FenceLanguage::Plain(plain);
        }

        FenceLanguage::from_escaped(normalize_code_block_language(lang))
    }

    pub(in crate::renderer::html::renderer) fn render_math_block(&mut self, math: &MathBlock<'_>) {
        self.write("<div class=\"ox-math ox-math-block\"");
        self.write_source_span_attr(math.span);
        self.write(" data-ox-tex=\"");
        self.write_attribute_escaped(math.value);
        self.write("\"><math display=\"block\"><mtext>");
        self.write_escaped(math.value);
        self.write("</mtext></math></div>\n");
    }

    pub(in crate::renderer::html::renderer) fn render_html(&mut self, html: &Html<'_>) {
        self.write_html_value(html.value);
        // Block-level HTML values captured from full source lines already
        // end with their newline; don't double it.
        if !self.output.ends_with('\n') {
            self.write("\n");
        }
    }

    pub(in crate::renderer::html::renderer) fn render_table(&mut self, table: &Table<'_>) {
        self.write_table_opening(table);
        self.render_table_caption(table);
        self.write_table_colgroup(table);
        for (i, row) in table.children.iter().enumerate() {
            if i == 0 {
                self.write("<thead>\n");
            } else if i == 1 {
                self.write("<tbody>\n");
            }
            self.visit_table_row_with_header(row, i == 0, &table.align);
            if i == 0 {
                self.write("</thead>\n");
            }
        }
        if table.children.len() > 1 {
            self.write("</tbody>\n");
        }
        self.write("</table>\n");
    }
    pub(in crate::renderer::html::renderer) fn visit_table_row_with_header(
        &mut self,
        row: &TableRow<'_>,
        is_header: bool,
        align: &crate::allocator::Vec<'_, crate::ast::AlignKind>,
    ) {
        self.write("<tr");
        self.write_source_span_attr(row.span);
        self.write(">\n");
        let tag = if is_header { "th" } else { "td" };
        let mut column_index = 0usize;
        for cell in &row.children {
            self.write_table_cell_open(tag, cell, align.get(column_index).copied());
            self.visit_table_cell(cell);
            self.write("</");
            self.write(tag);
            self.write(">\n");
            column_index = column_index.saturating_add(Self::normalized_table_colspan(cell));
        }
        self.write("</tr>\n");
    }

    /// Writes the part of a table opening shared by the default and hook paths.
    pub(in crate::renderer::html::renderer) fn write_table_opening(&mut self, table: &Table<'_>) {
        self.write("<table");
        if let Some(attributes) = &table.attributes {
            if let Some(id) = attributes.id {
                self.write(" id=\"");
                self.write_attribute_escaped(id);
                self.write("\"");
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
        self.write_source_span_attr(table.span);
        self.write(">\n");
    }

    pub(in crate::renderer::html::renderer) fn render_table_caption(&mut self, table: &Table<'_>) {
        let Some(attributes) = &table.attributes else {
            return;
        };
        if attributes.caption.is_empty() {
            return;
        }
        self.write("<caption>");
        for child in &attributes.caption {
            self.visit_inline_node(child);
        }
        self.write("</caption>\n");
    }

    pub(in crate::renderer::html::renderer) fn normalized_table_colspan(
        cell: &TableCell<'_>,
    ) -> usize {
        cell.colspan.max(1)
    }

    pub(in crate::renderer::html::renderer) fn write_table_cell_open(
        &mut self,
        tag: &str,
        cell: &TableCell<'_>,
        alignment: Option<AlignKind>,
    ) {
        self.write("<");
        self.write(tag);
        match alignment.unwrap_or(AlignKind::None) {
            AlignKind::Left => self.write(" align=\"left\""),
            AlignKind::Center => self.write(" align=\"center\""),
            AlignKind::Right => self.write(" align=\"right\""),
            AlignKind::None => {}
        }
        let colspan = Self::normalized_table_colspan(cell);
        if colspan > 1 {
            self.write(" colspan=\"");
            self.write_display(colspan);
            self.write("\"");
        }
        self.write_source_span_attr(cell.span);
        self.write(">");
    }

    pub(in crate::renderer::html::renderer) fn visit_table_cell(&mut self, cell: &TableCell<'_>) {
        for child in &cell.children {
            self.visit_inline_node(child);
        }
    }
}

#[cfg(test)]
mod tests;
