//! Read-only plain text and section projection from an existing document tree.
//!
//! Extraction never parses source, renders JSX, highlights code, or evaluates
//! MDX. See [`TextExtractionOptions`] for the static content policy.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{Document, Heading, Node, Span};
use crate::callout::detect_callout;
use crate::renderer::{
    HeadingIdPlanner, JsxRendererOptions, collect_jsx_heading_text, map_heading_level,
    slugify_heading, update_raw_text_context,
};

/// A half-open UTF-8 byte range in [`TextExtraction::text`].
///
/// This is a normalized text range, never an original source span. JavaScript
/// consumers must encode the text as UTF-8 before slicing with these offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    /// Inclusive byte offset.
    pub start: usize,
    /// Exclusive byte offset.
    pub end: usize,
}

/// One heading and the text up to the next heading, in document order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSection {
    /// Heading label, following JSX's static heading label rules.
    pub heading: String,
    /// Effective heading level, after offset and clamping to 1 through 6.
    pub level: u8,
    /// Index of the nearest earlier section with a lower effective level.
    pub parent_index: Option<usize>,
    /// Final JSX anchor ID; absent when heading IDs are disabled.
    pub id: Option<String>,
    /// Original heading syntax's half-open UTF-8 source span, when available.
    pub heading_span: Option<Span>,
    /// Original source from this heading to the next heading (or document end).
    ///
    /// Absent for generated headings or reordered/overlapping heading spans.
    /// This describes source boundaries, not a source map for normalized text.
    pub source_span: Option<Span>,
    /// Half-open UTF-8 byte range into [`TextExtraction::text`], including the heading.
    pub text_range: TextRange,
    /// Owned text for this range, excluding separators before the next heading.
    pub text: String,
}

/// Owned document text and its ordered, nonoverlapping heading sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextExtraction {
    /// Normalized static text. Content before the first heading is retained here.
    pub text: String,
    /// Sections in traversal order. Child text does not repeat in parent sections.
    pub sections: Vec<TextSection>,
    /// Original span of an omitted title heading, if one was omitted.
    pub omitted_title_heading: Option<Span>,
}

/// Static text policy and final heading planning choices.
///
/// Text includes decoded prose, link labels, image alt text, inline code,
/// literal TeX, and footnote bodies. Footnote reference labels/backlinks, link
/// destinations/titles, front matter, raw HTML nodes, MDX attributes, ESM, and
/// expressions are omitted. MDX wrappers contribute their parsed static children;
/// `script`, `style`, `textarea`, and `title` children are omitted. Runtime component output is unknown.
///
/// Prose whitespace collapses to ASCII spaces. Paragraphs/blocks use two LF
/// characters; list items and table rows use one LF; table cells use tabs.
/// Inline breaks and omitted runtime content use one space. Block code retains parser-normalized whitespace,
/// except final LF characters, and can be excluded while retaining inline code.
/// Empty omitted blocks do not produce leading or trailing separators.
///
/// Heading planning follows native JSX body/module output, including title
/// omission and the namespace shared with authored element and footnote IDs.
/// Use matching [`JsxRendererOptions`] when comparing final anchors.
#[derive(Debug, Clone)]
pub struct TextExtractionOptions {
    /// Include fenced and indented block code. Default: `true`.
    pub include_block_code: bool,
    /// Heading and callout settings shared with JSX rendering.
    ///
    /// Only `heading_ids`, `heading_level_offset`, `heading_id_prefix`,
    /// `omit_title_heading`, and `callouts` affect extraction. Other render
    /// settings and hooks have no effect on this static projection.
    pub rendering: JsxRendererOptions,
}

impl Default for TextExtractionOptions {
    fn default() -> Self {
        Self {
            include_block_code: true,
            rendering: JsxRendererOptions::default(),
        }
    }
}

impl Document<'_> {
    /// Projects static text and final JSX heading sections without reparsing.
    ///
    /// Extraction is opt-in, deterministic, and does not mutate the document.
    /// Native passes should run before this call; their resulting tree is the
    /// exact input. Empty source spans are treated as unavailable provenance.
    #[must_use]
    pub fn extract_text(&self, options: &TextExtractionOptions) -> TextExtraction {
        let mut projection = Projection::new(options);
        let mut considered_first_h1 = false;
        for node in &self.children {
            if let Node::Heading(heading) = node
                && heading.depth == 1
                && !considered_first_h1
            {
                considered_first_h1 = true;
                if let Some(title) = options
                    .rendering
                    .omit_title_heading
                    .as_deref()
                    .map(str::trim)
                    .filter(|title| !title.is_empty())
                    && collect_jsx_heading_text(&heading.children).trim() == title
                {
                    projection.omitted_title_heading =
                        (!heading.span.is_empty()).then_some(heading.span);
                    continue;
                }
            }
            projection.node(node, true);
        }
        projection.finish(self.span)
    }
}

struct Projection<'a> {
    options: &'a TextExtractionOptions,
    writer: TextWriter,
    planner: HeadingIdPlanner,
    footnote_targets: BTreeSet<String>,
    footnote_first_references: BTreeSet<String>,
    footnote_counts: BTreeMap<String, usize>,
    sections: Vec<TextSection>,
    ancestors: Vec<usize>,
    omitted_title_heading: Option<Span>,
    list_depth: usize,
    raw_text_context: Option<String>,
}

impl<'a> Projection<'a> {
    fn new(options: &'a TextExtractionOptions) -> Self {
        Self {
            options,
            writer: TextWriter::default(),
            planner: HeadingIdPlanner::new(),
            footnote_targets: BTreeSet::new(),
            footnote_first_references: BTreeSet::new(),
            footnote_counts: BTreeMap::new(),
            sections: Vec::new(),
            ancestors: Vec::new(),
            omitted_title_heading: None,
            list_depth: 0,
            raw_text_context: None,
        }
    }

    fn children(&mut self, children: &[Node<'_>], visible: bool) {
        for child in children {
            self.node(child, visible);
        }
    }

    // Inline raw HTML context belongs to one source run, never to a caption
    // visited before its authored table cells or to a later paragraph.
    fn inline_children(&mut self, children: &[Node<'_>], visible: bool) {
        let previous = self.raw_text_context.take();
        self.children(children, visible);
        self.raw_text_context = previous;
    }

    fn boundary(&mut self) {
        self.writer
            .separator(if self.list_depth == 0 { "\n\n" } else { "\n" });
    }

    fn footnote_ids(&mut self, identifier: &str, occurrence: usize) {
        if self.footnote_targets.insert(identifier.to_owned()) {
            let _ = self.planner.plan(&format!("fn-{identifier}"));
        }
        if occurrence != 1 || self.footnote_first_references.insert(identifier.to_owned()) {
            let candidate = if occurrence == 1 {
                format!("fnref-{identifier}")
            } else {
                format!("fnref-{identifier}-{occurrence}")
            };
            let _ = self.planner.plan(&candidate);
        }
    }

    fn heading(&mut self, heading: &Heading<'_>, visible: bool) {
        let label = collect_jsx_heading_text(&heading.children);
        let options = &self.options.rendering;
        let level = map_heading_level(heading.depth, options.heading_level_offset);
        let id = options.heading_ids.then(|| {
            let mut candidate = options.heading_id_prefix.clone();
            candidate.push_str(heading.explicit_id().unwrap_or_default());
            if heading.explicit_id().is_none() {
                candidate.push_str(&slugify_heading(&label));
            }
            self.planner.plan(&candidate)
        });
        if visible {
            // Keep inter-section separators outside either section's text range.
            if let Some(previous) = self.sections.last_mut() {
                previous.text_range.end = self.writer.text.len();
            }
            self.writer.separator("\n\n");
            self.writer.flush_separator();
            while self
                .ancestors
                .last()
                .is_some_and(|&index| self.sections[index].level >= level)
            {
                self.ancestors.pop();
            }
            let index = self.sections.len();
            self.sections.push(TextSection {
                heading: label,
                level,
                parent_index: self.ancestors.last().copied(),
                id,
                heading_span: (!heading.span.is_empty()).then_some(heading.span),
                source_span: None,
                text_range: TextRange {
                    start: self.writer.text.len(),
                    end: self.writer.text.len(),
                },
                text: String::new(),
            });
            self.ancestors.push(index);
        }
        self.inline_children(&heading.children, visible);
        if visible {
            self.boundary();
        }
    }

    fn node(&mut self, node: &Node<'_>, inherited_visible: bool) {
        // Raw HTML closers inside inline groups can restore visibility; only
        // inherited MDX suppression persists for the whole child subtree.
        let visible = inherited_visible && self.raw_text_context.is_none();
        if !matches!(
            node,
            Node::Heading(_) | Node::Definition(_) | Node::MdxjsEsm(_)
        ) && let Some(id) = node.explicit_element_id()
        {
            let _ = self.planner.plan(id);
        }
        match node {
            Node::Heading(heading) => self.heading(heading, visible),
            Node::Text(value) if visible => self.writer.prose(value.value),
            Node::InlineCode(value) if visible => self.writer.literal(value.value),
            Node::InlineMath(value) if visible => self.writer.prose(value.value),
            Node::Image(value) if visible => self.writer.prose(value.alt),
            Node::Break(_) if visible => self.writer.separator(" "),
            Node::CodeBlock(value) if visible && self.options.include_block_code => {
                self.boundary();
                self.writer.literal(value.value.trim_end_matches('\n'));
                self.boundary();
            }
            Node::MathBlock(value) if visible => {
                self.boundary();
                self.writer.prose(value.value);
                self.boundary();
            }
            Node::Paragraph(value) => {
                if visible {
                    self.boundary();
                }
                self.inline_children(&value.children, visible);
                if visible {
                    self.boundary();
                }
            }
            Node::BlockQuote(value) => {
                if visible {
                    self.boundary();
                }
                if self.options.rendering.callouts
                    && let Some(Node::Paragraph(paragraph)) = value.children.first()
                    && let Some((kind, consumed)) = detect_callout(paragraph)
                {
                    if visible {
                        self.writer.prose(kind.label());
                        self.boundary();
                    }
                    let previous_context = self.raw_text_context.take();
                    let mut remaining = consumed;
                    let mut before_body = true;
                    for child in &paragraph.children {
                        if let Node::Text(text) = child {
                            let skip = remaining.min(text.value.len());
                            remaining -= skip;
                            let mut value = &text.value[skip..];
                            if before_body {
                                value = value.trim_start();
                            }
                            if !value.is_empty() {
                                before_body = false;
                                if visible && self.raw_text_context.is_none() {
                                    self.writer.prose(value);
                                }
                            }
                        } else {
                            before_body = false;
                            self.node(child, visible);
                        }
                    }
                    self.raw_text_context = previous_context;
                    if visible {
                        self.boundary();
                    }
                    self.children(&value.children[1..], visible);
                } else {
                    self.children(&value.children, visible);
                }
                if visible {
                    self.boundary();
                }
            }
            Node::List(value) => {
                if visible {
                    self.boundary();
                }
                self.list_depth += 1;
                for item in &value.children {
                    self.children(&item.children, visible);
                    if visible {
                        self.writer.separator("\n");
                    }
                }
                self.list_depth -= 1;
                if visible {
                    self.boundary();
                }
            }
            Node::ListItem(value) => self.children(&value.children, visible),
            Node::Table(value) => {
                if visible {
                    self.boundary();
                }
                if let Some(attributes) = &value.attributes {
                    self.inline_children(&attributes.caption, visible);
                    if visible {
                        self.boundary();
                    }
                }
                for (row_index, row) in value.children.iter().enumerate() {
                    if visible {
                        if row_index == 0 {
                            self.writer.flush_separator();
                        } else {
                            self.writer.delimiter('\n');
                        }
                    }
                    for (cell_index, cell) in row.children.iter().enumerate() {
                        if visible && cell_index > 0 {
                            self.writer.delimiter('\t');
                        }
                        self.inline_children(&cell.children, visible);
                    }
                }
                if visible {
                    self.boundary();
                }
            }
            Node::Figure(value) => {
                if visible {
                    self.boundary();
                }
                self.node(&value.content, visible);
                if visible {
                    self.boundary();
                }
                self.inline_children(&value.caption, visible);
                if visible {
                    self.boundary();
                }
            }
            Node::DefinitionList(value) => self.children(&value.children, visible),
            Node::DefinitionListTerm(value) => {
                if visible {
                    self.boundary();
                }
                self.inline_children(&value.children, visible);
                if visible {
                    self.boundary();
                }
            }
            Node::DefinitionListDefinition(value) => self.children(&value.children, visible),
            Node::Span(value) => self.children(&value.children, inherited_visible),
            Node::Emphasis(value) => self.children(&value.children, inherited_visible),
            Node::Strong(value) => self.children(&value.children, inherited_visible),
            Node::Link(value) => self.children(&value.children, inherited_visible),
            Node::Highlight(value) => self.children(&value.children, inherited_visible),
            Node::Delete(value) => self.children(&value.children, inherited_visible),
            Node::Insertion(value) => self.children(&value.children, inherited_visible),
            Node::Superscript(value) => self.children(&value.children, inherited_visible),
            Node::Subscript(value) => self.children(&value.children, inherited_visible),
            Node::FootnoteReference(value) => {
                let count = self
                    .footnote_counts
                    .entry(value.identifier.to_owned())
                    .or_default();
                *count += 1;
                let count = *count;
                self.footnote_ids(value.identifier, count);
            }
            Node::FootnoteDefinition(value) => {
                self.footnote_ids(value.identifier, 1);
                if visible {
                    self.boundary();
                }
                self.children(&value.children, visible);
                if visible {
                    self.boundary();
                }
            }
            Node::MdxJsxFlowElement(value) => {
                if visible {
                    self.boundary();
                }
                let child_visible = visible && !is_hidden_element(value.name);
                if !value.self_closing {
                    self.children(&value.children, child_visible);
                }
                if visible {
                    self.boundary();
                }
            }
            Node::MdxJsxTextElement(value) => {
                let hidden = is_hidden_element(value.name);
                if visible && (hidden || value.self_closing) {
                    self.writer.separator(" ");
                }
                if !value.self_closing {
                    self.children(&value.children, inherited_visible && !hidden);
                }
                if visible && hidden {
                    self.writer.separator(" ");
                }
            }
            Node::Html(value) => {
                let was_hidden = self.raw_text_context.is_some();
                update_raw_text_context(value.value, &mut self.raw_text_context);
                if was_hidden || self.raw_text_context.is_some() {
                    self.writer.separator(" ");
                }
            }
            Node::MdxTextExpression(_) => {
                if visible {
                    self.writer.separator(" ");
                }
            }
            Node::MdxFlowExpression(_) => {
                if visible {
                    self.boundary();
                }
            }
            Node::ThematicBreak(_) if visible => self.boundary(),
            Node::Definition(_)
            | Node::MdxjsEsm(_)
            | Node::Text(_)
            | Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Image(_)
            | Node::Break(_)
            | Node::CodeBlock(_)
            | Node::MathBlock(_)
            | Node::ThematicBreak(_) => {}
        }
    }

    fn finish(mut self, document_span: Span) -> TextExtraction {
        if let Some(last) = self.sections.last_mut() {
            last.text_range.end = self.writer.text.len();
        }
        for index in 0..self.sections.len() {
            let next_span = self.sections.get(index + 1).map(|next| next.heading_span);
            let previous_span = index
                .checked_sub(1)
                .and_then(|previous| self.sections[previous].heading_span);
            let section = &mut self.sections[index];
            self.writer.text[section.text_range.start..section.text_range.end]
                .clone_into(&mut section.text);
            if let Some(heading_span) = section.heading_span {
                let source_end = match next_span {
                    None => Some(document_span.end),
                    Some(Some(span)) => Some(span.start),
                    Some(None) => None,
                };
                if let Some(end) = source_end
                    && document_span.contains_span(&heading_span)
                    && previous_span.is_none_or(|previous| previous.end <= heading_span.start)
                    && end >= heading_span.end
                    && end <= document_span.end
                {
                    section.source_span = Some(Span::new(heading_span.start, end));
                }
            }
        }
        TextExtraction {
            text: self.writer.text,
            sections: self.sections,
            omitted_title_heading: self.omitted_title_heading,
        }
    }
}

fn is_hidden_element(name: Option<&str>) -> bool {
    matches!(name, Some("script" | "style" | "textarea" | "title"))
}

#[derive(Default)]
struct TextWriter {
    text: String,
    pending: &'static str,
}

impl TextWriter {
    fn separator(&mut self, separator: &'static str) {
        if separator.len() > self.pending.len() || (separator != " " && self.pending == " ") {
            self.pending = separator;
        }
    }

    fn flush_separator(&mut self) {
        if !self.text.is_empty() {
            self.text.push_str(self.pending);
        }
        self.pending = "";
    }

    fn prose(&mut self, value: &str) {
        for ch in value.chars() {
            if ch.is_whitespace() {
                self.separator(" ");
            } else {
                self.flush_separator();
                self.text.push(ch);
            }
        }
    }

    fn literal(&mut self, value: &str) {
        if !value.is_empty() {
            self.flush_separator();
            self.text.push_str(value);
        }
    }

    fn delimiter(&mut self, delimiter: char) {
        self.pending = "";
        self.text.push(delimiter);
    }
}
