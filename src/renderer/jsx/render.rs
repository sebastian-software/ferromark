//! Direct AST-to-JSX renderer.

use std::collections::BTreeMap;

use crate::ast::{
    AlignKind, Attribute, BlockQuote, CodeBlock, DefinitionList, DefinitionListDefinition,
    DefinitionListTerm, Document, Figure, FootnoteDefinition, FootnoteReference, Heading, Html,
    Image, InlineSpan, Link, List, ListItem, MathBlock, MdxJsxAttributeEntry, MdxJsxAttributeValue,
    MdxJsxFlowElement, MdxJsxTextElement, Node, Paragraph, Span, Table, TableRow, ThematicBreak,
};
use crate::callout::{CalloutKind, detect_callout};
use crate::outline::OutlineEntry;
use crate::renderer::{
    HeadingIdPlanner, JsxRendererOptions, JsxSourceMapping, map_heading_level, slugify_heading,
};

use super::code_metadata::{CodeBlockMetadata, parse_code_block_metadata};
use super::line_index::LineIndex;
use super::{JsxCodeBlock, JsxCodeBlockInput, JsxModuleSource, JsxOutput, JsxRenderHooks};

pub(super) fn render_document<H: JsxRenderHooks>(
    document: &Document<'_>,
    source: &str,
    options: &JsxRendererOptions,
    hooks: &mut H,
) -> JsxOutput {
    render_document_with_references(document, source, options, hooks).0
}

/// Renders a document and also returns each component reference with its full
/// member path, such as `ui.Badge`, in first-reference order. Module assembly
/// needs the path; [`JsxOutput::components`] holds only the root identifiers.
pub(super) fn render_document_with_references<H: JsxRenderHooks>(
    document: &Document<'_>,
    source: &str,
    options: &JsxRendererOptions,
    hooks: &mut H,
) -> (JsxOutput, Vec<String>) {
    let mut renderer = Renderer::new(source, options, hooks);
    renderer.push("<>\n");
    let mut considered_first_h1 = false;
    for child in &document.children {
        if let Node::Heading(heading) = child
            && heading.depth == 1
            && !considered_first_h1
        {
            considered_first_h1 = true;
            if let Some(title) = options
                .omit_title_heading
                .as_deref()
                .map(str::trim)
                .filter(|title| !title.is_empty())
                && heading_text(&heading.children).trim() == title
            {
                renderer.output.omitted_title_heading = Some(heading.span);
                continue;
            }
        }
        renderer.render_node(child);
    }
    renderer.push("</>");
    (renderer.output, renderer.references)
}

struct Renderer<'a, 'options, 'hooks, H> {
    source: &'a str,
    line_index: LineIndex<'a>,
    options: &'options JsxRendererOptions,
    hooks: &'hooks mut H,
    output: JsxOutput,
    references: Vec<String>,
    generated_line: u32,
    generated_column: u32,
    generated_previous_cr: bool,
    id_planner: HeadingIdPlanner,
    footnote_targets: BTreeMap<String, String>,
    footnote_reference_ids: BTreeMap<String, String>,
    footnote_reference_counts: BTreeMap<String, usize>,
    raw_text_element: Option<String>,
    inline_context_depth: usize,
}

impl<'a, 'options, 'hooks, H: JsxRenderHooks> Renderer<'a, 'options, 'hooks, H> {
    fn new(source: &'a str, options: &'options JsxRendererOptions, hooks: &'hooks mut H) -> Self {
        Self {
            source,
            line_index: LineIndex::new(source),
            options,
            hooks,
            output: JsxOutput::default(),
            references: Vec::new(),
            generated_line: 0,
            generated_column: 0,
            generated_previous_cr: false,
            id_planner: HeadingIdPlanner::new(),
            footnote_targets: BTreeMap::new(),
            footnote_reference_ids: BTreeMap::new(),
            footnote_reference_counts: BTreeMap::new(),
            raw_text_element: None,
            inline_context_depth: 0,
        }
    }

    fn push(&mut self, value: &str) {
        self.output.body.push_str(value);
        for ch in value.chars() {
            match ch {
                '\r' => {
                    self.generated_line = self.generated_line.saturating_add(1);
                    self.generated_column = 0;
                    self.generated_previous_cr = true;
                }
                '\n' if self.generated_previous_cr => self.generated_previous_cr = false,
                '\n' => {
                    self.generated_line = self.generated_line.saturating_add(1);
                    self.generated_column = 0;
                    self.generated_previous_cr = false;
                }
                _ => {
                    self.generated_column =
                        self.generated_column.saturating_add(ch.len_utf16() as u32);
                    self.generated_previous_cr = false;
                }
            }
        }
    }

    fn push_js_string(&mut self, value: &str) {
        let mut escaped = String::with_capacity(value.len().saturating_add(2));
        escaped.push('"');
        for ch in value.chars() {
            match ch {
                '"' => escaped.push_str("\\\""),
                '\\' => escaped.push_str("\\\\"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                '\u{0008}' => escaped.push_str("\\b"),
                '\u{000c}' => escaped.push_str("\\f"),
                '\u{2028}' => escaped.push_str("\\u2028"),
                '\u{2029}' => escaped.push_str("\\u2029"),
                ch if ch.is_control() => {
                    use std::fmt::Write as _;
                    let _ = write!(escaped, "\\u{:04x}", ch as u32);
                }
                _ => escaped.push(ch),
            }
        }
        escaped.push('"');
        self.push(&escaped);
    }

    fn push_string_child(&mut self, value: &str) {
        self.push("{");
        self.push_js_string(value);
        self.push("}");
    }

    fn push_string_attribute(&mut self, name: &str, value: &str) {
        self.push(" ");
        self.push(name);
        self.push("={");
        self.push_js_string(value);
        self.push("}");
    }

    fn push_string_attribute_with_suffix(&mut self, name: &str, prefix: &str, suffix: &str) {
        let mut value = String::with_capacity(prefix.len() + suffix.len());
        value.push_str(prefix);
        value.push_str(suffix);
        self.push_string_attribute(name, &value);
    }

    fn push_boolean_attribute(&mut self, name: &str, value: bool) {
        self.push(" ");
        self.push(name);
        self.push("={");
        self.push(if value { "true}" } else { "false}" });
    }

    fn add_mapping(&mut self, span: Span) {
        self.add_mapping_at(span.start as usize);
    }

    fn add_mapping_at(&mut self, source_offset: usize) {
        let (source_line, source_column) = self.line_index.position(source_offset);
        self.output.mappings.push(JsxSourceMapping {
            generated_line: self.generated_line,
            generated_column: self.generated_column,
            source_line,
            source_column,
        });
    }

    fn push_source(&mut self, value: &str, source_offset: usize) {
        let mut cursor = 0usize;
        self.add_mapping_at(source_offset);
        while cursor < value.len() {
            let next_line_break = value[cursor..]
                .char_indices()
                .find(|(_, ch)| matches!(ch, '\r' | '\n'));
            let Some((relative, ch)) = next_line_break else {
                self.push(&value[cursor..]);
                return;
            };
            let break_start = cursor + relative;
            if break_start > cursor {
                self.push(&value[cursor..break_start]);
            }
            let break_len = if ch == '\r' && value.as_bytes().get(break_start + 1) == Some(&b'\n') {
                2
            } else {
                ch.len_utf8()
            };
            self.push(&value[break_start..break_start + break_len]);
            cursor = break_start + break_len;
            self.add_mapping_at(source_offset.saturating_add(cursor));
        }
    }

    fn render_node(&mut self, node: &Node<'_>) {
        if let Node::MdxjsEsm(esm) = node {
            self.output.esm.push(JsxModuleSource {
                value: esm.value.to_owned(),
                span: esm.span,
            });
            return;
        }
        if matches!(node, Node::Definition(_)) {
            return;
        }

        self.add_mapping(node.span());
        let planned_id = if !matches!(node, Node::Heading(_)) {
            node.explicit_element_id()
                .map(|id| self.id_planner.plan(id))
        } else {
            None
        };

        match node {
            Node::Paragraph(value) => self.render_paragraph(value),
            Node::Heading(value) => self.render_heading(value),
            Node::ThematicBreak(value) => self.render_thematic_break(value),
            Node::BlockQuote(value) => self.render_block_quote(value),
            Node::List(value) => self.render_list(value),
            Node::ListItem(value) => self.render_list_item(value, false),
            Node::CodeBlock(value) => self.render_code_block(value),
            Node::MathBlock(value) => self.render_math_block(value),
            Node::Html(value) => self.render_html(value),
            Node::Table(value) => self.render_table(value, planned_id.as_deref()),
            Node::Figure(value) => self.render_figure(value, planned_id.as_deref()),
            Node::DefinitionList(value) => self.render_definition_list(value),
            Node::DefinitionListTerm(value) => self.render_definition_list_term(value),
            Node::DefinitionListDefinition(value) => self.render_definition_list_definition(value),
            Node::Text(value) => self.push_string_child(value.value),
            Node::Span(value) => self.render_span(value, planned_id.as_deref()),
            Node::Emphasis(value) => self.render_wrapped_inline("em", &value.children),
            Node::Strong(value) => self.render_wrapped_inline("strong", &value.children),
            Node::InlineCode(value) => self.render_inline_code(value.value),
            Node::InlineMath(value) => self.render_inline_math(value.value),
            Node::Break(_) => self.render_void("br"),
            Node::Link(value) => self.render_link(value, planned_id.as_deref()),
            Node::Image(value) => self.render_image(value, planned_id.as_deref()),
            Node::Highlight(value) => self.render_wrapped_inline("mark", &value.children),
            Node::Delete(value) => self.render_wrapped_inline("del", &value.children),
            Node::Insertion(value) => self.render_wrapped_inline("ins", &value.children),
            Node::Superscript(value) => self.render_wrapped_inline("sup", &value.children),
            Node::Subscript(value) => self.render_wrapped_inline("sub", &value.children),
            Node::FootnoteReference(value) => self.render_footnote_reference(value),
            Node::FootnoteDefinition(value) => self.render_footnote_definition(value),
            Node::MdxJsxFlowElement(value) => self.render_mdx_flow_element(value),
            Node::MdxJsxTextElement(value) => self.render_mdx_text_element(value),
            Node::MdxFlowExpression(value) => {
                self.push("{");
                self.push_source(value.value, value.span.start.saturating_add(1) as usize);
                self.push("}");
            }
            Node::MdxTextExpression(value) => {
                self.push("{");
                self.push_source(value.value, value.span.start.saturating_add(1) as usize);
                self.push("}");
            }
            Node::Definition(_) | Node::MdxjsEsm(_) => {}
        }
    }

    fn generated_tag_name(&mut self, tag: &str) -> String {
        if !self.output.elements.iter().any(|element| element == tag) {
            self.output.elements.push(tag.to_owned());
        }
        match self.options.component_prefix.as_deref() {
            Some(prefix) if !prefix.is_empty() => {
                let mut rendered = String::with_capacity(prefix.len() + 1 + tag.len());
                rendered.push_str(prefix);
                rendered.push('.');
                rendered.push_str(tag);
                rendered
            }
            _ => tag.to_owned(),
        }
    }

    fn start_generated(&mut self, tag: &str) -> String {
        let rendered = self.generated_tag_name(tag);
        self.push("<");
        self.push(&rendered);
        rendered
    }

    fn end_generated(&mut self, tag: &str) {
        self.push("</");
        self.push(tag);
        self.push(">");
    }

    fn render_void(&mut self, tag: &str) {
        let _ = self.start_generated(tag);
        self.push(" />");
    }

    fn add_component(&mut self, name: &str) {
        let root = name.split('.').next().unwrap_or(name);
        let member = name.contains('.');
        let intrinsic = root.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
            || name.contains(['-', ':']);
        if root.is_empty() || (!member && intrinsic) {
            return;
        }
        if !self
            .output
            .components
            .iter()
            .any(|component| component == root)
        {
            self.output.components.push(root.to_owned());
        }
        if !self.references.iter().any(|reference| reference == name) {
            self.references.push(name.to_owned());
        }
    }

    fn render_paragraph(&mut self, paragraph: &Paragraph<'_>) {
        let rendered = self.start_generated("p");
        self.push(">");
        self.render_inline_children(&paragraph.children);
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_heading(&mut self, heading: &Heading<'_>) {
        let level = map_heading_level(heading.depth, self.options.heading_level_offset);
        let id = if self.options.heading_ids {
            let mut candidate = self.options.heading_id_prefix.clone();
            if let Some(explicit) = heading.explicit_id() {
                candidate.push_str(explicit);
            } else {
                candidate.push_str(&slugify_heading(&heading_text(&heading.children)));
            }
            Some(self.id_planner.plan(&candidate))
        } else {
            None
        };

        let tag = match level {
            1 => "h1",
            2 => "h2",
            3 => "h3",
            4 => "h4",
            5 => "h5",
            _ => "h6",
        };
        let rendered = self.start_generated(tag);
        if let Some(id) = id.as_deref() {
            self.push_string_attribute("id", id);
        }
        let classes = heading.classes();
        if !classes.is_empty() {
            self.push_string_attribute("className", &classes.join(" "));
        }
        if let Some(attributes) = &heading.attributes {
            self.write_element_attributes(&attributes.values, &["id", "class", "className"]);
        }
        self.push(">");
        self.output.headings.push(OutlineEntry {
            level,
            text: heading_text(&heading.children),
            id,
            span: heading.span,
        });
        self.render_inline_children(&heading.children);
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_thematic_break(&mut self, _break_node: &ThematicBreak) {
        self.render_void("hr");
        self.push("\n");
    }

    fn render_block_quote(&mut self, block_quote: &BlockQuote<'_>) {
        if self.options.callouts
            && let Some(Node::Paragraph(paragraph)) = block_quote.children.first()
            && let Some((kind, consumed)) = detect_callout(paragraph)
        {
            self.render_callout(kind, paragraph, consumed, &block_quote.children[1..]);
            return;
        }

        let rendered = self.start_generated("blockquote");
        self.push(">\n");
        for child in &block_quote.children {
            self.render_node(child);
        }
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_callout(
        &mut self,
        kind: CalloutKind,
        paragraph: &Paragraph<'_>,
        consumed: usize,
        remaining: &[Node<'_>],
    ) {
        let key = kind.class_name();
        let component = self.options.callout_components.get(key).cloned();
        let callout_tag = if let Some(component) = component.as_deref() {
            self.add_component(component);
            self.push("<");
            self.push(component);
            self.push(" kind=");
            self.push("{");
            self.push_js_string(key);
            self.push("} title=");
            self.push("{");
            self.push_js_string(kind.label());
            self.push("}>");
            component.to_owned()
        } else {
            let tag = self.start_generated("blockquote");
            self.push_string_attribute_with_suffix("className", "ox-callout ox-callout--", key);
            self.push(">");
            self.push("\n");
            let title = self.start_generated("p");
            self.push_string_attribute("className", "ox-callout-title");
            self.push(">");
            self.push_string_child(kind.label());
            self.end_generated(&title);
            self.push("\n");
            tag
        };

        if callout_paragraph_has_body(paragraph, consumed) {
            self.add_mapping(paragraph.span);
            if component.is_none() {
                let p = self.start_generated("p");
                self.push(">");
                self.render_callout_paragraph_body(paragraph, consumed);
                self.end_generated(&p);
                self.push("\n");
            } else {
                self.render_callout_paragraph_body(paragraph, consumed);
            }
        }
        for child in remaining {
            self.render_node(child);
        }
        self.end_generated(&callout_tag);
        self.push("\n");
    }

    fn render_callout_paragraph_body(&mut self, paragraph: &Paragraph<'_>, consumed: usize) {
        let mut remaining = consumed;
        let mut before_body = true;
        for child in &paragraph.children {
            if let Node::Text(text) = child {
                let mut value = text.value;
                if remaining > 0 {
                    let skip = remaining.min(value.len());
                    value = &value[skip..];
                    remaining -= skip;
                }
                if before_body {
                    value = value.trim_start();
                }
                if value.is_empty() {
                    continue;
                }
                before_body = false;
                self.push_string_child(value);
            } else {
                before_body = false;
                self.render_node(child);
            }
        }
    }

    fn render_list(&mut self, list: &List<'_>) {
        let tag = if list.ordered { "ol" } else { "ul" };
        let rendered = self.start_generated(tag);
        if let Some(start) = list.start.filter(|start| *start != 1)
            && list.ordered
        {
            self.push(" start={");
            self.push(&start.to_string());
            self.push("}");
        }
        self.push(">\n");
        let tight = !list.spread;
        for item in &list.children {
            self.render_list_item(item, tight);
        }
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_list_item(&mut self, item: &ListItem<'_>, tight: bool) {
        let rendered = self.start_generated("li");
        self.push(">");
        if let Some(checked) = item.checked {
            let input = self.start_generated("input");
            let _ = input;
            self.push_string_attribute("type", "checkbox");
            self.push_boolean_attribute("checked", checked);
            self.push_boolean_attribute("disabled", true);
            self.push(" /> ");
        }
        for child in &item.children {
            if tight {
                if let Node::Paragraph(paragraph) = child {
                    self.add_mapping(paragraph.span);
                    self.render_inline_children(&paragraph.children);
                    continue;
                }
                if !self.output.body.ends_with('\n') {
                    self.push("\n");
                }
            }
            self.render_node(child);
        }
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_code_block(&mut self, code: &CodeBlock<'_>) {
        let input = JsxCodeBlockInput {
            code: code.value,
            language: code.lang,
            meta: code.meta,
            span: code.span,
        };
        let hooked = self.hooks.render_code_block(input);
        let metadata = parse_code_block_metadata(
            code.lang,
            code.meta,
            code.value,
            self.options.show_line_numbers,
        );
        let language_component = if hooked.is_none() {
            metadata
                .language
                .as_deref()
                .and_then(|language| self.code_component_for(language))
        } else {
            None
        };
        let component = if hooked.is_none() {
            language_component
                .clone()
                .or_else(|| self.options.code_block_component.clone())
        } else {
            None
        };
        let highlighted = if hooked.is_none() && language_component.is_none() {
            self.hooks.highlight_code_block(JsxCodeBlockInput {
                language: metadata.language.as_deref(),
                ..input
            })
        } else {
            None
        }
        .filter(|highlighted| is_valid_highlighted_code(highlighted, code.value));

        self.output.code_blocks.push(JsxCodeBlock {
            value: code.value.to_owned(),
            language: code.lang.map(str::to_owned),
            meta: code.meta.map(str::to_owned),
            span: code.span,
            component: component.clone(),
        });

        if let Some(snippet) = hooked {
            self.push(&snippet);
        } else if let Some(component) = language_component.as_deref() {
            self.add_component(component);
            self.push("<");
            self.push(component);
            if let Some(language) = metadata.language.as_deref() {
                self.push_string_attribute("language", language);
            }
            if let Some(meta) = code.meta {
                self.push_string_attribute("meta", meta);
            }
            self.push(">");
            self.push_string_child(code.value);
            self.push("</");
            self.push(component);
            self.push(">");
        } else {
            let global_component = component.as_deref();
            if let Some(component) = global_component {
                self.add_component(component);
                self.push("<");
                self.push(component);
                self.push_string_attribute("code", code.value);
                if let Some(language) = metadata.language.as_deref() {
                    self.push_string_attribute("language", language);
                }
                if let Some(title) = metadata.title.as_deref() {
                    self.push_string_attribute("title", title);
                }
                if let Some(label) = metadata.label.as_deref() {
                    self.push_string_attribute("data-label", label);
                }
                self.push_boolean_attribute("lineNumbers", metadata.line_number_start.is_some());
                self.push(">");
            }
            self.render_code_markup(
                code.value,
                metadata.language.as_deref(),
                &metadata,
                highlighted.as_ref(),
            );
            if let Some(component) = global_component {
                self.push("</");
                self.push(component);
                self.push(">");
            }
        }
        if !self.output.body.ends_with('\n') {
            self.push("\n");
        }
    }

    fn code_component_for(&self, language: &str) -> Option<String> {
        self.options
            .code_block_components
            .get(language)
            .or_else(|| {
                let normalized = language.to_ascii_lowercase();
                self.options.code_block_components.get(&normalized)
            })
            .cloned()
    }

    fn render_code_markup(
        &mut self,
        code: &str,
        language: Option<&str>,
        metadata: &CodeBlockMetadata,
        highlighted: Option<&super::JsxHighlightedCodeBlock>,
    ) {
        let line_numbers = metadata.line_number_start.is_some();
        let has_ranges = metadata.highlighted_lines.iter().any(|selected| *selected);
        let has_metadata =
            line_numbers || has_ranges || metadata.title.is_some() || metadata.label.is_some();
        let wrap_lines = highlighted.is_some() || line_numbers || has_ranges;

        let pre = self.start_generated("pre");
        let mut classes = Vec::new();
        if highlighted.is_some() {
            classes.push("shiki");
        }
        if has_metadata {
            classes.push("ox-code-block");
        }
        if line_numbers {
            classes.push("ox-code-block--line-numbers");
            classes.push("line-numbers-mode");
        }
        if has_ranges {
            classes.push("ox-code-block--annotated");
            classes.push("has-highlighted");
        }
        if !classes.is_empty() {
            let classes = classes.join(" ");
            self.push_string_attribute("className", &classes);
        }
        if let Some(highlighted) = highlighted {
            self.write_highlight_colors(highlighted);
        }
        if let Some(title) = metadata.title.as_deref() {
            self.push_string_attribute("data-code-title", title);
        }
        if let Some(label) = metadata.label.as_deref() {
            self.push_string_attribute("data-label", label);
        }
        if let Some(line_number_start) = metadata.line_number_start {
            self.push_string_attribute("data-line-numbers", "true");
            self.push_string_attribute("data-line-number-start", &line_number_start.to_string());
        }
        self.push(">");

        let code_tag = self.start_generated("code");
        if let Some(language) = language.filter(|language| !language.is_empty()) {
            self.push_string_attribute_with_suffix("className", "language-", language);
        }
        self.push(">");
        if wrap_lines {
            let lines: Vec<&str> = code.split('\n').collect();
            for (index, line) in lines.iter().enumerate() {
                if index > 0 {
                    self.push_string_child("\n");
                }
                let mut line_classes = Vec::with_capacity(3);
                line_classes.push("line");
                if line_numbers || has_ranges {
                    line_classes.push("ox-code-line");
                }
                let highlighted_line = metadata
                    .highlighted_lines
                    .get(index)
                    .copied()
                    .unwrap_or(false);
                if highlighted_line {
                    line_classes.push("ox-code-line--highlight");
                    line_classes.push("highlighted");
                }
                let line_tag = self.start_generated("span");
                let classes = line_classes.join(" ");
                self.push_string_attribute("className", &classes);
                self.push_string_attribute("data-line", &(index + 1).to_string());
                if let Some(start) = metadata.line_number_start {
                    let number = start.saturating_add(index).to_string();
                    self.push_string_attribute("data-ln", &number);
                }
                self.push(">");
                if let Some(fragment) = highlighted.and_then(|value| value.lines.get(index)) {
                    self.write_raw_html(fragment);
                } else {
                    self.push_string_child(line);
                }
                self.end_generated(&line_tag);
            }
        } else {
            self.push_string_child(code);
        }
        self.end_generated(&code_tag);
        self.end_generated(&pre);
    }

    fn write_highlight_colors(&mut self, highlighted: &super::JsxHighlightedCodeBlock) {
        let has_colors = highlighted.foreground.is_some()
            || highlighted.background.is_some()
            || highlighted.dark_foreground.is_some()
            || highlighted.dark_background.is_some();
        if !has_colors {
            return;
        }
        self.push(" style={{");
        let mut emitted = false;
        if let Some(foreground) = highlighted.foreground.as_deref() {
            self.write_jsx_style_property("color", foreground, &mut emitted);
        }
        if let Some(background) = highlighted.background.as_deref() {
            self.write_jsx_style_property("backgroundColor", background, &mut emitted);
        }
        if highlighted.dark_foreground.is_some() || highlighted.foreground.is_some() {
            if let Some(foreground) = highlighted.foreground.as_deref() {
                self.write_jsx_style_property("--shiki-light", foreground, &mut emitted);
            }
            if let Some(foreground) = highlighted.dark_foreground.as_deref() {
                self.write_jsx_style_property("--shiki-dark", foreground, &mut emitted);
            }
        }
        if highlighted.dark_background.is_some() || highlighted.background.is_some() {
            if let Some(background) = highlighted.background.as_deref() {
                self.write_jsx_style_property("--shiki-light-bg", background, &mut emitted);
            }
            if let Some(background) = highlighted.dark_background.as_deref() {
                self.write_jsx_style_property("--shiki-dark-bg", background, &mut emitted);
            }
        }
        self.push("}}");
    }

    fn write_jsx_style_property(&mut self, name: &str, value: &str, emitted: &mut bool) {
        if *emitted {
            self.push(", ");
        }
        if name.starts_with("--") {
            self.push_js_string(name);
        } else {
            self.push(name);
        }
        self.push(": ");
        self.push_js_string(value);
        *emitted = true;
    }

    fn render_math_block(&mut self, math: &MathBlock<'_>) {
        let div = self.start_generated("div");
        self.push_string_attribute("className", "ox-math ox-math-block");
        self.push_string_attribute("data-ox-tex", math.value);
        self.push(">");
        let math_tag = self.start_generated("math");
        self.push(" display={\"block\"}>");
        let mtext = self.start_generated("mtext");
        self.push(">");
        self.push_string_child(math.value);
        self.end_generated(&mtext);
        self.end_generated(&math_tag);
        self.end_generated(&div);
        self.push("\n");
    }

    fn render_html(&mut self, html: &Html<'_>) {
        self.write_raw_html(html.value);
        if self.inline_context_depth == 0
            && !html.value.ends_with('\n')
            && !self.output.body.ends_with('\n')
        {
            self.push("\n");
        }
    }

    fn write_raw_html(&mut self, raw: &str) {
        let bytes = raw.as_bytes();
        let mut cursor = 0usize;
        while cursor < bytes.len() {
            if let Some(tag) = self.raw_text_element.as_deref() {
                let closing = find_raw_text_closer(raw, cursor, tag);
                let end = closing.unwrap_or(bytes.len());
                if cursor < end {
                    let content = &raw[cursor..end];
                    let decoded = if matches!(tag, "script" | "style") {
                        content.to_owned()
                    } else {
                        decode_html_entities(content)
                    };
                    self.push_string_child(&decoded);
                    cursor = end;
                }
                if cursor == bytes.len() {
                    break;
                }
            }

            let Some(relative_open) = raw[cursor..].find('<') else {
                let tail = &raw[cursor..];
                if !tail.is_empty() {
                    self.push_string_child(&decode_html_entities(tail));
                }
                break;
            };
            let open = cursor + relative_open;
            if open > cursor {
                let text = &raw[cursor..open];
                let decoded = match self.raw_text_element.as_deref() {
                    Some("script" | "style") => text.to_owned(),
                    _ => decode_html_entities(text),
                };
                self.push_string_child(&decoded);
            }

            if raw[open..].starts_with("<!--")
                && let Some(end) = raw[open + 4..].find("-->")
            {
                // JSX cannot contain HTML comments. Retain the comment as
                // a source-only JSX comment, which has no runtime child.
                let comment = raw[open + 4..open + 4 + end].replace("*/", "* /");
                self.push("{/*");
                self.push(&comment);
                self.push("*/}");
                cursor = open + 4 + end + 3;
                continue;
            }

            if (raw[open..].starts_with("<!") || raw[open..].starts_with("<?"))
                && let Some(end) = raw[open..].find('>')
            {
                self.push_string_child(&raw[open..open + end + 1]);
                cursor = open + end + 1;
                continue;
            }

            if let Some((parsed, end)) = parse_raw_tag(raw, open) {
                if parsed.name.contains(':') {
                    // Namespaced XML tags are not accepted by all JSX
                    // compilers. Preserve the original source visibly.
                    self.push_string_child(&raw[open..end]);
                } else {
                    self.write_raw_tag(&parsed);
                }
                if parsed.closing {
                    if self
                        .raw_text_element
                        .as_deref()
                        .is_some_and(|name| name.eq_ignore_ascii_case(parsed.name))
                    {
                        self.raw_text_element = None;
                    }
                } else if !parsed.self_closing && !is_void_html_element(parsed.name) {
                    self.raw_text_element = ["script", "style", "textarea", "title"]
                        .iter()
                        .any(|name| parsed.name.eq_ignore_ascii_case(name))
                        .then(|| parsed.name.to_ascii_lowercase());
                }
                cursor = end;
                continue;
            }

            self.push_string_child("<");
            cursor = open + 1;
        }
    }

    fn write_raw_tag(&mut self, tag: &RawTag<'_>) {
        if tag.closing {
            self.push("</");
            self.push(&raw_tag_name(tag.name));
            self.push(">");
            return;
        }

        self.push("<");
        self.push(&raw_tag_name(tag.name));
        for attribute in &tag.attributes {
            let normalized_name = attribute.name.to_ascii_lowercase();
            let name = jsx_attribute_name(&normalized_name);
            if attribute.name.eq_ignore_ascii_case("style") {
                if let Some(value) = attribute.value {
                    self.write_style_attribute(&decode_html_entities(value));
                } else {
                    self.write_style_attribute("");
                }
            } else {
                self.push(" ");
                self.push(name);
                if let Some(value) = attribute.value {
                    self.push("=");
                    if attribute.quoted {
                        self.push(&attribute.quote.to_string());
                        self.push(value);
                        self.push(&attribute.quote.to_string());
                    } else {
                        self.push("{");
                        self.push_js_string(&decode_html_entities(value));
                        self.push("}");
                    }
                }
            }
        }
        if tag.self_closing || is_void_html_element(tag.name) {
            self.push(" />");
        } else {
            self.push(">");
        }
    }

    fn write_style_attribute(&mut self, style: &str) {
        self.push(" style={{");
        let mut emitted = false;
        for declaration in css_declarations(style) {
            let Some((property, value)) = declaration.split_once(':') else {
                continue;
            };
            let property = property.trim();
            let value = value.trim();
            if property.is_empty() || value.is_empty() {
                continue;
            }
            if emitted {
                self.push(", ");
            }
            if property.starts_with("--") {
                self.push_js_string(property);
            } else {
                let property = css_property_to_js(property);
                if is_js_identifier(&property) {
                    self.push(&property);
                } else {
                    self.push_js_string(&property);
                }
            }
            self.push(": ");
            self.push_js_string(value);
            emitted = true;
        }
        self.push("}}");
    }

    fn render_table(&mut self, table: &Table<'_>, planned_id: Option<&str>) {
        let rendered = self.start_generated("table");
        if let Some(attributes) = &table.attributes {
            if let Some(id) = planned_id {
                self.push_string_attribute("id", id);
            }
            if !attributes.classes.is_empty() {
                self.push_string_attribute("className", &attributes.classes.join(" "));
            }
            self.write_element_attributes(&attributes.attributes, &["id", "class", "className"]);
        }
        self.push(">");
        self.render_table_caption(table);

        for (row_index, row) in table.children.iter().enumerate() {
            if row_index == 0 {
                self.push("\n");
                let thead = self.start_generated("thead");
                self.push(">");
                self.render_table_row(row, true, table.align.as_slice());
                self.end_generated(&thead);
            } else if row_index == 1 {
                self.push("\n");
                let tbody = self.start_generated("tbody");
                self.push(">");
                self.render_table_row(row, false, table.align.as_slice());
                // Close tbody after all remaining rows.
                for later in table.children.iter().skip(row_index + 1) {
                    self.render_table_row(later, false, table.align.as_slice());
                }
                self.end_generated(&tbody);
                break;
            }
        }
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_table_caption(&mut self, table: &Table<'_>) {
        let Some(attributes) = &table.attributes else {
            return;
        };
        if attributes.caption.is_empty() {
            return;
        }
        self.push("\n");
        let caption = self.start_generated("caption");
        self.push(">");
        self.render_inline_children(&attributes.caption);
        self.end_generated(&caption);
    }

    fn render_table_row(&mut self, row: &TableRow<'_>, header: bool, align: &[AlignKind]) {
        self.push("\n");
        let tr = self.start_generated("tr");
        self.push(">");
        let mut column_index = 0usize;
        for cell in &row.children {
            self.push("\n");
            let tag = if header { "th" } else { "td" };
            let cell_tag = self.start_generated(tag);
            if let Some(alignment) = align.get(column_index) {
                let alignment = match alignment {
                    AlignKind::None => None,
                    AlignKind::Left => Some("left"),
                    AlignKind::Center => Some("center"),
                    AlignKind::Right => Some("right"),
                };
                if let Some(alignment) = alignment {
                    self.push_string_attribute("align", alignment);
                }
            }
            if cell.colspan > 1 {
                self.push(" colSpan={");
                self.push(&cell.colspan.to_string());
                self.push("}");
            }
            self.push(">");
            self.render_inline_children(&cell.children);
            self.end_generated(&cell_tag);
            column_index = column_index.saturating_add(cell.colspan.max(1));
        }
        self.end_generated(&tr);
    }

    fn render_figure(&mut self, figure: &Figure<'_>, planned_id: Option<&str>) {
        let rendered = self.start_generated("figure");
        if let Some(attributes) = &figure.attributes {
            if let Some(id) = planned_id {
                self.push_string_attribute("id", id);
            }
            if !attributes.classes.is_empty() {
                self.push_string_attribute("className", &attributes.classes.join(" "));
            }
            self.write_element_attributes(&attributes.values, &["id", "class", "className"]);
        }
        self.push(">");
        self.render_node(&figure.content);
        self.push("\n");
        let caption = self.start_generated("figcaption");
        self.push(">");
        self.render_inline_children(&figure.caption);
        self.end_generated(&caption);
        self.push("\n");
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_definition_list(&mut self, list: &DefinitionList<'_>) {
        let rendered = self.start_generated("dl");
        self.push_string_attribute("className", "ox-definition-list");
        self.push(">");
        for child in &list.children {
            self.render_node(child);
        }
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_definition_list_term(&mut self, term: &DefinitionListTerm<'_>) {
        let rendered = self.start_generated("dt");
        self.push(">");
        self.render_inline_children(&term.children);
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_definition_list_definition(&mut self, definition: &DefinitionListDefinition<'_>) {
        let rendered = self.start_generated("dd");
        self.push(">");
        if definition.children.len() == 1
            && let Some(Node::Paragraph(paragraph)) = definition.children.first()
        {
            self.add_mapping(paragraph.span);
            self.render_inline_children(&paragraph.children);
        } else {
            self.push("\n");
            for child in &definition.children {
                self.render_node(child);
            }
        }
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn render_wrapped_inline(&mut self, tag: &str, children: &[Node<'_>]) {
        let rendered = self.start_generated(tag);
        self.push(">");
        self.render_inline_children(children);
        self.end_generated(&rendered);
    }

    fn render_inline_code(&mut self, value: &str) {
        let rendered = self.start_generated("code");
        self.push(">");
        self.push_string_child(value);
        self.end_generated(&rendered);
    }

    fn render_inline_math(&mut self, value: &str) {
        let span = self.start_generated("span");
        self.push_string_attribute("className", "ox-math ox-math-inline");
        self.push_string_attribute("data-ox-tex", value);
        self.push(">");
        let math = self.start_generated("math");
        self.push(">");
        let mtext = self.start_generated("mtext");
        self.push(">");
        self.push_string_child(value);
        self.end_generated(&mtext);
        self.end_generated(&math);
        self.end_generated(&span);
    }

    fn render_inline_children(&mut self, children: &[Node<'_>]) {
        self.inline_context_depth = self.inline_context_depth.saturating_add(1);
        for child in children {
            self.render_node(child);
        }
        self.inline_context_depth = self.inline_context_depth.saturating_sub(1);
    }

    fn render_link(&mut self, link: &Link<'_>, planned_id: Option<&str>) {
        let rendered = self.start_generated("a");
        self.push_string_attribute("href", link.url);
        if let Some(title) = link.title {
            self.push_string_attribute("title", title);
        }
        if let Some(attributes) = &link.attributes {
            if let Some(id) = planned_id {
                self.push_string_attribute("id", id);
            }
            if !attributes.classes.is_empty() {
                self.push_string_attribute("className", &attributes.classes.join(" "));
            }
            self.write_element_attributes(
                &attributes.values,
                &["href", "title", "id", "class", "className"],
            );
        }
        self.push(">");
        self.render_inline_children(&link.children);
        self.end_generated(&rendered);
    }

    fn render_image(&mut self, image: &Image<'_>, planned_id: Option<&str>) {
        self.start_generated("img");
        self.push_string_attribute("src", image.url);
        self.push_string_attribute("alt", image.alt);
        if let Some(title) = image.title {
            self.push_string_attribute("title", title);
        }
        if let Some(attributes) = &image.attributes {
            if let Some(id) = planned_id {
                self.push_string_attribute("id", id);
            }
            if !attributes.classes.is_empty() {
                self.push_string_attribute("className", &attributes.classes.join(" "));
            }
            let reserved: &[&str] = if image.title.is_some() {
                &["src", "alt", "id", "class", "className", "title"]
            } else {
                &["src", "alt", "id", "class", "className"]
            };
            self.write_element_attributes(&attributes.values, reserved);
        }
        self.push(" />");
    }

    fn render_span(&mut self, span: &InlineSpan<'_>, planned_id: Option<&str>) {
        let rendered = self.start_generated("span");
        if let Some(id) = planned_id {
            self.push_string_attribute("id", id);
        }
        if !span.classes.is_empty() {
            self.push_string_attribute("className", &span.classes.join(" "));
        }
        self.write_element_attributes(&span.attributes, &["id", "class", "className"]);
        self.push(">");
        self.render_inline_children(&span.children);
        self.end_generated(&rendered);
    }

    fn render_mdx_flow_element(&mut self, element: &MdxJsxFlowElement<'_>) {
        self.render_mdx_element(
            element.name,
            &element.attributes,
            &element.children,
            element.self_closing,
            false,
        );
    }

    fn render_mdx_text_element(&mut self, element: &MdxJsxTextElement<'_>) {
        self.render_mdx_element(
            element.name,
            &element.attributes,
            &element.children,
            element.self_closing,
            true,
        );
    }

    fn render_mdx_element(
        &mut self,
        name: Option<&str>,
        attributes: &[MdxJsxAttributeEntry<'_>],
        children: &[Node<'_>],
        self_closing: bool,
        inline: bool,
    ) {
        let Some(name) = name else {
            self.push("<>");
            if inline {
                self.render_inline_children(children);
            } else {
                for child in children {
                    self.render_node(child);
                }
            }
            self.push("</>");
            return;
        };

        self.add_component(name);
        self.push("<");
        self.push(name);
        for attribute in attributes {
            match attribute {
                MdxJsxAttributeEntry::Attribute(attribute) => {
                    self.push(" ");
                    self.push(attribute.name);
                    match &attribute.value {
                        None => {}
                        Some(MdxJsxAttributeValue::Literal(value)) => {
                            if let Some((source_value, source_offset)) =
                                self.original_literal_attribute(attribute.span)
                            {
                                self.push("=");
                                self.push_source(&source_value, source_offset);
                            } else {
                                self.push("={");
                                self.push_js_string(value);
                                self.push("}");
                            }
                        }
                        Some(MdxJsxAttributeValue::Expression(expression)) => {
                            self.push("={");
                            self.push_source(
                                expression.value,
                                expression.span.start.saturating_add(1) as usize,
                            );
                            self.push("}");
                        }
                    }
                }
                MdxJsxAttributeEntry::Expression(expression) => {
                    self.push(" ");
                    self.push("{");
                    self.push_source(
                        expression.value,
                        expression.span.start.saturating_add(1) as usize,
                    );
                    self.push("}");
                }
            }
        }
        if self_closing {
            self.push(" />");
            return;
        }
        self.push(">");
        if inline {
            self.render_inline_children(children);
        } else {
            for child in children {
                self.render_node(child);
            }
        }
        self.push("</");
        self.push(name);
        self.push(">");
    }

    fn render_footnote_reference(&mut self, footnote: &FootnoteReference<'_>) {
        let target = self.footnote_target_id(footnote.identifier);
        let occurrence = self
            .footnote_reference_counts
            .entry(footnote.identifier.to_owned())
            .or_default();
        *occurrence += 1;
        let occurrence = *occurrence;
        let reference = self.footnote_reference_id(footnote.identifier, occurrence);

        let sup = self.start_generated("sup");
        self.push(">");
        let anchor = self.start_generated("a");
        self.push_string_attribute_with_suffix("href", "#", &target);
        self.push_string_attribute("id", &reference);
        self.push(">");
        self.push_string_child(footnote.identifier);
        self.end_generated(&anchor);
        self.end_generated(&sup);
    }

    fn render_footnote_definition(&mut self, footnote: &FootnoteDefinition<'_>) {
        let target = self.footnote_target_id(footnote.identifier);
        let reference = self.footnote_reference_id(footnote.identifier, 1);
        let rendered = self.start_generated("div");
        self.push_string_attribute("id", &target);
        self.push_string_attribute("className", "footnote");
        self.push(">");
        self.push("\n");
        for child in &footnote.children {
            self.render_node(child);
        }
        let anchor = self.start_generated("a");
        self.push_string_attribute_with_suffix("href", "#", &reference);
        self.push(">");
        self.push_string_child("↩");
        self.end_generated(&anchor);
        self.push("\n");
        self.end_generated(&rendered);
        self.push("\n");
    }

    fn footnote_target_id(&mut self, identifier: &str) -> String {
        if let Some(target) = self.footnote_targets.get(identifier) {
            return target.clone();
        }
        let mut candidate = String::with_capacity("fn-".len() + identifier.len());
        candidate.push_str("fn-");
        candidate.push_str(identifier);
        let target = self.id_planner.plan(&candidate);
        self.footnote_targets
            .insert(identifier.to_owned(), target.clone());
        target
    }

    fn footnote_reference_id(&mut self, identifier: &str, occurrence: usize) -> String {
        if occurrence == 1
            && let Some(reference) = self.footnote_reference_ids.get(identifier)
        {
            return reference.clone();
        }
        let mut candidate = String::with_capacity("fnref-".len() + identifier.len() + 12);
        candidate.push_str("fnref-");
        candidate.push_str(identifier);
        if occurrence > 1 {
            candidate.push('-');
            candidate.push_str(&occurrence.to_string());
        }
        let reference = self.id_planner.plan(&candidate);
        if occurrence == 1 {
            self.footnote_reference_ids
                .insert(identifier.to_owned(), reference.clone());
        }
        reference
    }

    fn write_element_attributes(&mut self, attributes: &[Attribute<'_>], reserved: &[&str]) {
        for attribute in attributes {
            if reserved.contains(&attribute.name)
                || attributes.iter().any(|other| {
                    other.name == "data-".to_owned() + attribute.name
                        && !attribute.name.starts_with("data-")
                })
            {
                continue;
            }
            let mut name = attribute.name.to_owned();
            if !is_standard_html_attribute(attribute.name)
                && !attribute.name.starts_with("data-")
                && !attribute.name.starts_with("aria-")
            {
                name.insert_str(0, "data-");
            }
            if attribute.name == "style" {
                self.write_style_attribute(attribute.value);
                continue;
            }
            self.push(" ");
            self.push(jsx_attribute_name(&name));
            self.push("={");
            self.push_js_string(attribute.value);
            self.push("}");
        }
    }

    fn original_literal_attribute(&self, span: Span) -> Option<(String, usize)> {
        let raw = self.source.get(span.start as usize..span.end as usize)?;
        let equals = raw.find('=')?;
        let after_equals = &raw[equals + 1..];
        let leading_space = after_equals.len() - after_equals.trim_start().len();
        let value = after_equals[leading_space..].trim_end();
        matches!(value.as_bytes().first(), Some(b'\'' | b'"')).then_some((
            value.to_owned(),
            (span.start as usize)
                .saturating_add(equals + 1)
                .saturating_add(leading_space),
        ))
    }
}

fn is_valid_highlighted_code(highlighted: &super::JsxHighlightedCodeBlock, code: &str) -> bool {
    highlighted.lines.len() == code.split('\n').count()
        && highlighted
            .lines
            .iter()
            .all(|line| !line.contains(['\n', '\r']))
}

fn callout_paragraph_has_body(paragraph: &Paragraph<'_>, consumed: usize) -> bool {
    let mut remaining = consumed;
    for child in &paragraph.children {
        if let Node::Text(text) = child {
            let skip = remaining.min(text.value.len());
            remaining -= skip;
            if !text.value[skip..].trim().is_empty() {
                return true;
            }
        } else {
            return true;
        }
    }
    false
}

fn heading_text(nodes: &[Node<'_>]) -> String {
    let mut text = String::new();
    for node in nodes {
        append_heading_text(node, &mut text);
    }
    text
}

fn append_heading_text(node: &Node<'_>, text: &mut String) {
    match node {
        Node::Text(value) => text.push_str(value.value),
        Node::InlineCode(value) => text.push_str(value.value),
        Node::Emphasis(value) => append_heading_children(&value.children, text),
        Node::Strong(value) => append_heading_children(&value.children, text),
        Node::Highlight(value) => append_heading_children(&value.children, text),
        Node::Delete(value) => append_heading_children(&value.children, text),
        Node::Insertion(value) => append_heading_children(&value.children, text),
        Node::Superscript(value) => append_heading_children(&value.children, text),
        Node::Subscript(value) => append_heading_children(&value.children, text),
        Node::Link(value) => append_heading_children(&value.children, text),
        Node::Span(value) => append_heading_children(&value.children, text),
        Node::MdxJsxFlowElement(value) => append_heading_children(&value.children, text),
        Node::MdxJsxTextElement(value) => append_heading_children(&value.children, text),
        _ => {}
    }
}

fn append_heading_children(nodes: &[Node<'_>], text: &mut String) {
    for node in nodes {
        append_heading_text(node, text);
    }
}

fn jsx_attribute_name(name: &str) -> &str {
    match name {
        "class" => "className",
        "for" => "htmlFor",
        "accept-charset" => "acceptCharset",
        "http-equiv" => "httpEquiv",
        "tabindex" => "tabIndex",
        "colspan" => "colSpan",
        "rowspan" => "rowSpan",
        "maxlength" => "maxLength",
        "minlength" => "minLength",
        "readonly" => "readOnly",
        "crossorigin" => "crossOrigin",
        "referrerpolicy" => "referrerPolicy",
        "autofocus" => "autoFocus",
        "autoplay" => "autoPlay",
        "contenteditable" => "contentEditable",
        "spellcheck" => "spellCheck",
        "srcset" => "srcSet",
        "usemap" => "useMap",
        "datetime" => "dateTime",
        "formaction" => "formAction",
        "inputmode" => "inputMode",
        "playsinline" => "playsInline",
        "frameborder" => "frameBorder",
        "viewbox" => "viewBox",
        "preserveaspectratio" => "preserveAspectRatio",
        "xlink:href" => "xlinkHref",
        "xml:lang" => "xmlLang",
        _ => name,
    }
}

fn is_standard_html_attribute(name: &str) -> bool {
    const STANDARD: &str = "abbr accept accept-charset accesskey action align alt as async autocapitalize autocomplete autofocus autoplay capture charset checked cite class cols colspan content contenteditable controls coords crossorigin data datetime decoding default defer dir dirname disabled download draggable enctype enterkeyhint fetchpriority for form formaction headers height hidden high href hreflang http-equiv id inert inputmode integrity ismap kind label lang list loading loop low max maxlength media method min minlength multiple muted name nonce open optimum pattern placeholder playsinline poster preload readonly referrerpolicy rel required reversed role rows rowspan sandbox scope selected shape size sizes slot span spellcheck src srcdoc srcset start step style tabindex target title translate type usemap value width wrap";
    STANDARD
        .split_ascii_whitespace()
        .any(|candidate| candidate == name)
}

struct RawTag<'a> {
    name: &'a str,
    closing: bool,
    self_closing: bool,
    attributes: Vec<RawAttribute<'a>>,
}

struct RawAttribute<'a> {
    name: &'a str,
    value: Option<&'a str>,
    quoted: bool,
    quote: char,
}

fn parse_raw_tag(raw: &str, start: usize) -> Option<(RawTag<'_>, usize)> {
    let bytes = raw.as_bytes();
    if bytes.get(start) != Some(&b'<') {
        return None;
    }
    let mut cursor = start + 1;
    let closing = bytes.get(cursor) == Some(&b'/');
    if closing {
        cursor += 1;
    }
    let name_start = cursor;
    while let Some(byte) = bytes.get(cursor) {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.') {
            cursor += 1;
        } else {
            break;
        }
    }
    if cursor == name_start || !bytes[name_start].is_ascii_alphabetic() {
        return None;
    }
    let name = &raw[name_start..cursor];
    let mut attributes = Vec::new();
    let mut self_closing = false;

    loop {
        skip_ascii_space(bytes, &mut cursor);
        match bytes.get(cursor) {
            Some(b'>') => {
                cursor += 1;
                break;
            }
            Some(b'/') if bytes.get(cursor + 1) == Some(&b'>') => {
                self_closing = true;
                cursor += 2;
                break;
            }
            None => return None,
            _ => {}
        }

        if closing {
            return None;
        }
        let attribute_start = cursor;
        while let Some(byte) = bytes.get(cursor) {
            if byte.is_ascii_whitespace() || matches!(byte, b'=' | b'/' | b'>') {
                break;
            }
            cursor += 1;
        }
        if cursor == attribute_start {
            return None;
        }
        let attribute_name = &raw[attribute_start..cursor];
        skip_ascii_space(bytes, &mut cursor);
        if bytes.get(cursor) != Some(&b'=') {
            attributes.push(RawAttribute {
                name: attribute_name,
                value: None,
                quoted: false,
                quote: '"',
            });
            continue;
        }
        cursor += 1;
        skip_ascii_space(bytes, &mut cursor);
        if let quote @ (b'\'' | b'"') = bytes.get(cursor).copied()? {
            cursor += 1;
            let value_start = cursor;
            while let Some(byte) = bytes.get(cursor) {
                if *byte == quote {
                    break;
                }
                cursor += 1;
            }
            if bytes.get(cursor) != Some(&quote) {
                return None;
            }
            let value = &raw[value_start..cursor];
            cursor += 1;
            attributes.push(RawAttribute {
                name: attribute_name,
                value: Some(value),
                quoted: true,
                quote: quote as char,
            });
        } else {
            let value_start = cursor;
            while let Some(byte) = bytes.get(cursor) {
                if byte.is_ascii_whitespace() || matches!(byte, b'>' | b'/') {
                    break;
                }
                cursor += 1;
            }
            if cursor == value_start {
                return None;
            }
            attributes.push(RawAttribute {
                name: attribute_name,
                value: Some(&raw[value_start..cursor]),
                quoted: false,
                quote: '"',
            });
        }
    }

    Some((
        RawTag {
            name,
            closing,
            self_closing,
            attributes,
        },
        cursor,
    ))
}

fn skip_ascii_space(bytes: &[u8], cursor: &mut usize) {
    while bytes.get(*cursor).is_some_and(u8::is_ascii_whitespace) {
        *cursor += 1;
    }
}

fn is_void_html_element(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn find_raw_text_closer(raw: &str, from: usize, name: &str) -> Option<usize> {
    let bytes = raw.as_bytes();
    let pattern = name.as_bytes();
    if pattern.is_empty() || from >= bytes.len() {
        return None;
    }
    let mut cursor = from;
    while cursor + pattern.len() + 2 <= bytes.len() {
        if bytes.get(cursor..cursor + 2) == Some(b"</")
            && raw
                .get(cursor + 2..cursor + 2 + pattern.len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name))
            && bytes
                .get(cursor + 2 + pattern.len())
                .is_some_and(|next| next.is_ascii_whitespace() || matches!(next, b'>' | b'/'))
        {
            return Some(cursor);
        }
        cursor += 1;
    }
    None
}

fn css_declarations(style: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0usize;
    let mut quote = None;
    let mut depth = 0usize;
    for (index, ch) in style.char_indices() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
        } else {
            match ch {
                '\'' | '"' => quote = Some(ch),
                '(' => depth = depth.saturating_add(1),
                ')' => depth = depth.saturating_sub(1),
                ';' if depth == 0 => {
                    result.push(&style[start..index]);
                    start = index + ch.len_utf8();
                }
                _ => {}
            }
        }
    }
    if start < style.len() {
        result.push(&style[start..]);
    }
    result
}

fn css_property_to_js(property: &str) -> String {
    let mut output = String::with_capacity(property.len());
    let mut uppercase_next = false;
    for ch in property.chars() {
        if ch == '-' {
            uppercase_next = !output.is_empty();
        } else if uppercase_next {
            output.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            output.push(ch);
        }
    }
    if property.starts_with("-ms-") {
        output
    } else if property.starts_with('-') {
        let mut chars = output.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => output,
        }
    } else {
        output
    }
}

fn decode_html_entities(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    while cursor < value.len() {
        let Some(relative_ampersand) = value[cursor..].find('&') else {
            output.push_str(&value[cursor..]);
            break;
        };
        let ampersand = cursor + relative_ampersand;
        output.push_str(&value[cursor..ampersand]);
        let candidate = &value[ampersand..];
        let semicolon = candidate
            .char_indices()
            .take_while(|(index, _)| *index <= 33)
            .find_map(|(index, ch)| (ch == ';').then_some(index));
        let Some(semicolon) = semicolon else {
            output.push('&');
            cursor = ampersand + 1;
            continue;
        };
        let entity = &candidate[1..semicolon];
        if let Some(decoded) = decode_entity(entity) {
            output.push_str(&decoded);
            cursor = ampersand + semicolon + 1;
        } else {
            output.push('&');
            cursor = ampersand + 1;
        }
    }
    output
}

fn decode_entity(entity: &str) -> Option<String> {
    if let Some(numeric) = entity.strip_prefix('#') {
        let (digits, radix) = if let Some(hex) = numeric.strip_prefix(['x', 'X']) {
            (hex, 16)
        } else {
            (numeric, 10)
        };
        let code = u32::from_str_radix(digits, radix).ok()?;
        let ch = char::from_u32(code)
            .filter(|ch| *ch != '\0')
            .unwrap_or('\u{fffd}');
        return Some(ch.to_string());
    }
    html_entities()
        .get(entity)
        .map(|decoded| (*decoded).to_owned())
}

fn html_entities() -> &'static std::collections::HashMap<&'static str, &'static str> {
    use std::sync::OnceLock;
    static ENTITIES: OnceLock<std::collections::HashMap<&'static str, &'static str>> =
        OnceLock::new();
    ENTITIES.get_or_init(|| {
        include_str!("../../parser/inline/entities.txt")
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .collect()
    })
}

fn raw_tag_name(name: &str) -> String {
    if is_html_element_name(name) || !is_svg_element_name(name) {
        name.to_ascii_lowercase()
    } else {
        name.to_owned()
    }
}

fn is_js_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || matches!(ch, '_' | '$'))
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$'))
}

fn is_html_element_name(name: &str) -> bool {
    const ELEMENTS: &str = "a abbr address area article aside audio b base bdi bdo blockquote body br button canvas caption cite code col colgroup data datalist dd del details dfn dialog div dl dt em embed fieldset figcaption figure footer form h1 h2 h3 h4 h5 h6 head header hgroup hr html i iframe img input ins kbd label legend li link main map mark menu meta meter nav noscript object ol optgroup option output p picture pre progress q rp rt ruby s samp script search section select slot small source span strong style sub summary sup table tbody td template textarea tfoot th thead time title tr track u ul var video wbr";
    ELEMENTS
        .split_ascii_whitespace()
        .any(|candidate| candidate.eq_ignore_ascii_case(name))
}

fn is_svg_element_name(name: &str) -> bool {
    const ELEMENTS: &str = "svg animate circle clipPath defs desc ellipse feBlend feColorMatrix feComponentTransfer feComposite feConvolveMatrix feDiffuseLighting feDisplacementMap feDistantLight feDropShadow feFlood feFuncA feFuncB feFuncG feFuncR feGaussianBlur feImage feMerge feMergeNode feMorphology feOffset fePointLight feSpecularLighting feSpotLight feTile feTurbulence filter foreignObject g image line linearGradient marker mask metadata mpath path pattern polygon polyline radialGradient rect set stop switch symbol text textPath tspan use view";
    ELEMENTS
        .split_ascii_whitespace()
        .any(|candidate| candidate.eq_ignore_ascii_case(name))
}
