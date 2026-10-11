//! Conservative, source-preserving static Markdown projection.
//!
//! Projection removes parsed MDX constructs from the original source. It does
//! not evaluate JavaScript, render components, serialize transforms, or reparse
//! Markdown. Code and ordinary Markdown are copied from the source verbatim.

use crate::ast::{Document, MdxJsxAttributeEntry, Node, Span, Visit, walk_node};
use crate::renderer::{collect_jsx_heading_text, update_raw_text_context};

/// Choices for projecting original source as static Markdown.
#[derive(Debug, Clone, Default)]
pub struct MarkdownProjectionOptions {
    /// Omit recognized front matter, including its delimiter lines.
    ///
    /// Defaults to `false`. Front matter must have been enabled during parsing.
    pub omit_front_matter: bool,
    /// Omit the first top-level H1 only when its display text matches this title.
    ///
    /// Leading/trailing whitespace is ignored; empty titles omit nothing. This
    /// follows JSX title selection, without serializing transformed text.
    pub omit_title_heading: Option<String>,
}

/// Why an original source construct was removed or unwrapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownProjectionDiagnosticKind {
    /// Module source is omitted without importing or executing it.
    EsmOmitted,
    /// A JSX wrapper and all its attributes were discarded, retaining children.
    JsxWrapperUnwrapped,
    /// An expression was omitted without evaluating it.
    ExpressionOmitted,
    /// A self-closing element has no known static children and was omitted.
    SelfClosingElementOmitted,
    /// Raw HTML was omitted; block HTML can include its entire body.
    RawHtmlOmitted,
    /// JSX boundaries could not be verified in this source; the element was omitted.
    UnsupportedJsxOmitted,
    /// An intrinsic raw-text element (`script`, `style`, `title`, `textarea`) was omitted.
    RawTextElementOmitted,
    /// Recognized front matter was omitted by request.
    FrontMatterOmitted,
    /// A matching top-level title heading was omitted by request.
    TitleHeadingOmitted,
}

/// An omission or unwrapping, with an original half-open UTF-8 byte range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownProjectionDiagnostic {
    /// The reason for the change; no runtime component output is inferred.
    pub kind: MarkdownProjectionDiagnosticKind,
    /// Original source range `[start, end)`, measured in bytes, not characters.
    pub span: Span,
}

/// Owned Markdown and diagnostics derived without changing the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownProjection {
    /// Original Markdown with parsed MDX and raw HTML constructs removed.
    pub markdown: String,
    /// Changes in original source order, including requested metadata omissions.
    pub diagnostics: Vec<MarkdownProjectionDiagnostic>,
    /// Original source range of the title heading omitted by request, if any.
    pub omitted_title_heading: Option<Span>,
}

/// A document span does not identify a valid range in the supplied source.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("document span {span:?} is not a valid UTF-8 range in the supplied source")]
pub struct MarkdownProjectionError {
    /// The invalid original source range.
    pub span: Span,
}

impl Document<'_> {
    /// Projects static Markdown from the original source and this parsed tree.
    ///
    /// Supply the exact source used to parse the tree. Ordinary Markdown,
    /// reference definitions, code fences, inline code, indentation, and line
    /// endings retain their authored bytes. This is not a Markdown serializer:
    /// changes made by native passes are not written back to the source.
    ///
    /// Paired JSX components and fragments are unwrapped, regardless of their
    /// names or attributes. Expressions, ESM, self-closing elements, and raw
    /// HTML are omitted with diagnostics. Attributes and component-specific
    /// semantics are discarded. Unverifiable wrapper boundaries cause the
    /// whole element to be omitted conservatively with a diagnostic.
    ///
    /// Removal preserves line endings and container prefixes within tags. A
    /// space separates words that would otherwise be concatenated by removal.
    /// The method neither parses source again nor mutates the document.
    pub fn project_markdown(
        &self,
        source: &str,
        options: &MarkdownProjectionOptions,
    ) -> Result<MarkdownProjection, MarkdownProjectionError> {
        let lines = SourceLines::new(source);
        let mut collector = Collector {
            source,
            lines: &lines,
            removals: Vec::new(),
            diagnostics: Vec::new(),
            invalid: None,
            inline_context: 0,
            raw_text: None,
            raw_text_start: None,
        };
        collector.validate(self.span);
        if options.omit_front_matter
            && let Some(front_matter) = &self.front_matter
        {
            collector.omit(
                front_matter.span,
                MarkdownProjectionDiagnosticKind::FrontMatterOmitted,
            );
        }
        let mut considered_first_h1 = false;
        let mut omitted_title_heading = None;
        for child in &self.children {
            if let Node::Heading(heading) = child
                && heading.depth == 1
                && !considered_first_h1
            {
                considered_first_h1 = true;
                if let Some(title) = options.omit_title_heading.as_deref().map(str::trim)
                    && !title.is_empty()
                    && collect_jsx_heading_text(&heading.children).trim() == title
                {
                    collector.omit(
                        heading.span,
                        MarkdownProjectionDiagnosticKind::TitleHeadingOmitted,
                    );
                    omitted_title_heading = Some(heading.span);
                    continue;
                }
            }
            collector.visit_node(child);
        }
        if let Some(start) = collector.raw_text_start {
            collector.omit(
                Span::new(start, source.len() as u32),
                MarkdownProjectionDiagnosticKind::RawTextElementOmitted,
            );
        }
        if let Some(span) = collector.invalid {
            return Err(MarkdownProjectionError { span });
        }
        collector
            .diagnostics
            .sort_by_key(|diagnostic| (diagnostic.span.start, diagnostic.span.end));
        let markdown = apply_removals(source, &lines, &mut collector.removals);
        Ok(MarkdownProjection {
            markdown,
            diagnostics: collector.diagnostics,
            omitted_title_heading,
        })
    }
}

struct Collector<'source> {
    source: &'source str,
    lines: &'source SourceLines,
    removals: Vec<Removal>,
    diagnostics: Vec<MarkdownProjectionDiagnostic>,
    invalid: Option<Span>,
    inline_context: usize,
    raw_text: Option<String>,
    raw_text_start: Option<u32>,
}

struct Removal {
    span: Span,
    tight: bool,
}

impl Collector<'_> {
    fn validate(&mut self, span: Span) -> bool {
        let valid = span.start <= span.end
            && self
                .source
                .get(span.start as usize..span.end as usize)
                .is_some();
        if !valid && self.invalid.is_none() {
            self.invalid = Some(span);
        }
        valid
    }

    fn omit(&mut self, span: Span, kind: MarkdownProjectionDiagnosticKind) {
        if self.validate(span) {
            self.removals.push(Removal { span, tight: false });
            self.diagnostics
                .push(MarkdownProjectionDiagnostic { kind, span });
        }
    }

    fn unwrap(
        &mut self,
        span: Span,
        name: Option<&str>,
        attributes: &[MdxJsxAttributeEntry<'_>],
        self_closing: bool,
    ) -> bool {
        if matches!(name, Some("script" | "style" | "title" | "textarea")) {
            self.omit(
                span,
                MarkdownProjectionDiagnosticKind::RawTextElementOmitted,
            );
            return false;
        }
        if self_closing {
            self.omit(
                span,
                MarkdownProjectionDiagnosticKind::SelfClosingElementOmitted,
            );
            return false;
        }
        if !self.validate(span) {
            return false;
        }
        for attribute in attributes {
            let span = match attribute {
                MdxJsxAttributeEntry::Attribute(attribute) => attribute.span,
                MdxJsxAttributeEntry::Expression(expression) => expression.span,
            };
            if !self.validate(span) {
                return false;
            }
        }
        if let Some(spans) = wrapper_spans(self.source, self.lines, span, name, attributes) {
            self.removals
                .extend(<[Span; 2]>::from(spans).map(|span| Removal {
                    span,
                    tight: self.inline_context > 0,
                }));
            self.diagnostics.push(MarkdownProjectionDiagnostic {
                kind: MarkdownProjectionDiagnosticKind::JsxWrapperUnwrapped,
                span,
            });
            true
        } else {
            self.omit(
                span,
                MarkdownProjectionDiagnosticKind::UnsupportedJsxOmitted,
            );
            false
        }
    }
}

impl Visit<'_> for Collector<'_> {
    fn visit_node(&mut self, node: &Node<'_>) {
        if !self.validate(node.span()) {
            return;
        }
        if self.raw_text.is_some() && !matches!(node, Node::Html(_)) {
            walk_node(self, node);
            return;
        }
        match node {
            Node::MdxjsEsm(node) => {
                self.omit(node.span, MarkdownProjectionDiagnosticKind::EsmOmitted);
            }
            Node::MdxFlowExpression(node) => self.omit(
                node.span,
                MarkdownProjectionDiagnosticKind::ExpressionOmitted,
            ),
            Node::MdxTextExpression(node) => self.omit(
                node.span,
                MarkdownProjectionDiagnosticKind::ExpressionOmitted,
            ),
            Node::Html(node) => {
                let was_raw_text = self.raw_text.is_some();
                update_raw_text_context(node.value, &mut self.raw_text);
                if !was_raw_text && self.raw_text.is_some() {
                    self.raw_text_start = Some(node.span.start);
                } else if was_raw_text
                    && self.raw_text.is_none()
                    && let Some(start) = self.raw_text_start.take()
                {
                    self.omit(
                        Span::new(start, node.span.end),
                        MarkdownProjectionDiagnosticKind::RawTextElementOmitted,
                    );
                }
                self.omit(node.span, MarkdownProjectionDiagnosticKind::RawHtmlOmitted);
            }
            Node::MdxJsxFlowElement(node) => {
                if self.unwrap(node.span, node.name, &node.attributes, node.self_closing) {
                    for child in &node.children {
                        self.visit_node(child);
                    }
                }
            }
            Node::MdxJsxTextElement(node) => {
                if self.unwrap(node.span, node.name, &node.attributes, node.self_closing) {
                    for child in &node.children {
                        self.visit_node(child);
                    }
                }
            }
            _ => {
                let phrasing = matches!(
                    node,
                    Node::Emphasis(_)
                        | Node::Strong(_)
                        | Node::Delete(_)
                        | Node::Insertion(_)
                        | Node::Highlight(_)
                        | Node::Span(_)
                        | Node::Link(_)
                        | Node::Superscript(_)
                        | Node::Subscript(_)
                );
                self.inline_context += usize::from(phrasing);
                walk_node(self, node);
                self.inline_context -= usize::from(phrasing);
            }
        }
    }
}

// Only verify delimiters of an already parsed element. Attribute spans skip
// quotes, JavaScript, and spreads; no MDX syntax is parsed here.
fn wrapper_spans(
    source: &str,
    lines: &SourceLines,
    span: Span,
    name: Option<&str>,
    attributes: &[MdxJsxAttributeEntry<'_>],
) -> Option<(Span, Span)> {
    let start = span.start as usize;
    let end = span.end as usize;
    let opening_start = start + source.get(start..end)?.find('<')?;
    let name = name.unwrap_or("");
    if !source.get(opening_start + 1..end)?.starts_with(name) {
        return None;
    }
    let anchor = attributes
        .iter()
        .map(|attribute| match attribute {
            MdxJsxAttributeEntry::Attribute(attribute) => attribute.span.end as usize,
            MdxJsxAttributeEntry::Expression(expression) => expression.span.end as usize,
        })
        .max()
        .unwrap_or(opening_start + 1 + name.len());
    if anchor < opening_start || anchor > end {
        return None;
    }
    let quotes = lines.at(opening_start).quotes;
    let opening_end = tag_end(source, anchor, end, quotes)?;
    let closing_start = opening_end + source.get(opening_end..end)?.rfind("</")?;
    if !source.get(closing_start + 2..end)?.starts_with(name) {
        return None;
    }
    let closing_end = tag_end(source, closing_start + 2 + name.len(), end, quotes)?;
    if !source.get(closing_end..end)?.trim().is_empty() {
        return None;
    }
    Some((
        Span::new(opening_start as u32, opening_end as u32),
        Span::new(closing_start as u32, closing_end as u32),
    ))
}

struct SourceLine {
    start: usize,
    prefix_end: usize,
    quotes: usize,
}

struct SourceLines {
    lines: Vec<SourceLine>,
}

impl SourceLines {
    fn new(source: &str) -> Self {
        let mut lines = Vec::new();
        let mut start = 0;
        loop {
            let end = source[start..]
                .find(['\n', '\r'])
                .map_or(source.len(), |at| start + at);
            let (prefix, quotes) = container_prefix(&source.as_bytes()[start..end]);
            lines.push(SourceLine {
                start,
                prefix_end: start + prefix,
                quotes,
            });
            if end == source.len() {
                break;
            }
            start = end + 1;
            if source.as_bytes()[end] == b'\r' && source.as_bytes().get(start) == Some(&b'\n') {
                start += 1;
            }
        }
        Self { lines }
    }

    fn at(&self, position: usize) -> &SourceLine {
        let index = self.lines.partition_point(|line| line.start <= position);
        &self.lines[index.saturating_sub(1)]
    }
}

fn container_prefix(prefix: &[u8]) -> (usize, usize) {
    let mut at = 0;
    let mut quotes = 0;
    while at < prefix.len() {
        match prefix[at] {
            b' ' | b'\t' => at += 1,
            b'>' => {
                quotes += 1;
                at += 1;
            }
            b'-' | b'+' | b'*' if prefix.get(at + 1).is_some_and(u8::is_ascii_whitespace) => {
                at += 1;
            }
            byte if byte.is_ascii_digit() => {
                let number_start = at;
                while prefix.get(at).is_some_and(u8::is_ascii_digit) {
                    at += 1;
                }
                if !matches!(prefix.get(at), Some(b'.' | b')'))
                    || !prefix.get(at + 1).is_some_and(u8::is_ascii_whitespace)
                {
                    at = number_start;
                    break;
                }
                at += 1;
            }
            _ => break,
        }
    }
    (at, quotes)
}

fn prefix_end(bytes: &[u8], mut at: usize, quotes: usize) -> Option<usize> {
    for _ in 0..quotes {
        while matches!(bytes.get(at), Some(b' ' | b'\t')) {
            at += 1;
        }
        if bytes.get(at) != Some(&b'>') {
            return None;
        }
        at += 1;
    }
    while matches!(bytes.get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    Some(at)
}

fn tag_end(source: &str, mut at: usize, end: usize, quotes: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    while at < end {
        match bytes[at] {
            b' ' | b'\t' => at += 1,
            b'\r' | b'\n' => {
                if bytes[at] == b'\r' && bytes.get(at + 1) == Some(&b'\n') {
                    at += 1;
                }
                at = prefix_end(bytes, at + 1, quotes)?;
            }
            b'>' => return Some(at + 1),
            _ => return None,
        }
    }
    None
}

fn apply_removals(source: &str, lines: &SourceLines, removals: &mut [Removal]) -> String {
    removals.sort_unstable_by_key(|removal| (removal.span.start, removal.span.end));
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut index = 0;
    while index < removals.len() {
        let start = removals[index].span.start as usize;
        let mut end = removals[index].span.end as usize;
        let mut tight = removals[index].tight;
        index += 1;
        while index < removals.len() && removals[index].span.start as usize <= end {
            end = end.max(removals[index].span.end as usize);
            tight = false;
            index += 1;
        }
        output.push_str(&source[cursor..start]);
        let before = output.chars().next_back();
        let before_len = output.len();
        preserve_line_boundaries(source, lines, start, end, &mut output);
        if output.len() == before_len
            && before.is_some_and(|before| !before.is_whitespace())
            && source[end..].chars().next().is_some_and(|after| {
                !after.is_whitespace()
                    && (!tight
                        || before.is_some_and(char::is_alphanumeric) && after.is_alphanumeric())
            })
        {
            output.push(' ');
        }
        cursor = end;
    }
    output.push_str(&source[cursor..]);
    output
}

fn preserve_line_boundaries(
    source: &str,
    lines: &SourceLines,
    start: usize,
    end: usize,
    output: &mut String,
) {
    let bytes = source.as_bytes();
    let line = lines.at(start);
    let quotes = line.quotes;
    let prefix = line.prefix_end;
    let mut at = if start < prefix && prefix <= end {
        output.push_str(&source[start..prefix]);
        prefix
    } else {
        start
    };
    while at < end {
        if matches!(bytes[at], b'\r' | b'\n') {
            output.push(bytes[at] as char);
            if bytes[at] == b'\r' && bytes.get(at + 1) == Some(&b'\n') && at + 1 < end {
                at += 1;
                output.push('\n');
            }
            at += 1;
            if quotes > 0
                && let Some(prefix) = prefix_end(bytes, at, quotes)
                && prefix <= end
            {
                output.push_str(&source[at..prefix]);
                at = prefix;
            }
        } else {
            at += 1;
        }
    }
}
