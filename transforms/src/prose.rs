//! Shared traversal of prose-bearing block and inline AST children.

use ferromark::ast::{Document, Node, Span};

use crate::{BoxError, TextRun, TransformContext};

pub type NodeList<'arena> = ferromark::allocator::Vec<'arena, Node<'arena>>;

/// Visits each prose-bearing sibling list, then its nested block or inline lists.
pub fn visit_document_prose<'arena, F>(
    document: &mut Document<'arena>,
    context: &TransformContext<'arena>,
    visit_link_labels: bool,
    visit_text_runs: &mut F,
) -> Result<(), BoxError>
where
    F: FnMut(&mut NodeList<'arena>, &TransformContext<'arena>, &[Span]) -> Result<(), BoxError>,
{
    let raw_html_spans = raw_html_text_spans(document, visit_link_labels);
    visit_block_children(
        &mut document.children,
        context,
        visit_link_labels,
        &raw_html_spans,
        visit_text_runs,
    )
}

pub fn is_in_raw_html(run: &TextRun<'_, '_>, raw_html_spans: &[Span]) -> bool {
    let run_span = run.source_span();
    !run_span.is_empty()
        && raw_html_spans
            .iter()
            .any(|span| run_span.start < span.end && span.start < run_span.end)
}

fn visit_block_children<'arena, F>(
    nodes: &mut NodeList<'arena>,
    context: &TransformContext<'arena>,
    visit_link_labels: bool,
    raw_html_spans: &[Span],
    visit_text_runs: &mut F,
) -> Result<(), BoxError>
where
    F: FnMut(&mut NodeList<'arena>, &TransformContext<'arena>, &[Span]) -> Result<(), BoxError>,
{
    visit_text_runs(nodes, context, raw_html_spans)?;

    for node in nodes {
        match node {
            Node::Paragraph(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Heading(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::BlockQuote(node) => {
                visit_block_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::List(node) => {
                for item in &mut node.children {
                    visit_block_children(
                        &mut item.children,
                        context,
                        visit_link_labels,
                        raw_html_spans,
                        visit_text_runs,
                    )?;
                }
            }
            Node::ListItem(node) => {
                visit_block_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Table(node) => {
                if let Some(attributes) = &mut node.attributes {
                    visit_inline_children(
                        &mut attributes.caption,
                        context,
                        visit_link_labels,
                        raw_html_spans,
                        visit_text_runs,
                    )?;
                }
                for row in &mut node.children {
                    for cell in &mut row.children {
                        visit_inline_children(
                            &mut cell.children,
                            context,
                            visit_link_labels,
                            raw_html_spans,
                            visit_text_runs,
                        )?;
                    }
                }
            }
            Node::DefinitionList(node) => {
                visit_block_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::DefinitionListTerm(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::DefinitionListDefinition(node) => {
                visit_block_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::FootnoteDefinition(node) => {
                visit_block_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::MdxJsxFlowElement(node) => {
                visit_block_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Emphasis(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Strong(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Highlight(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Delete(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Superscript(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Subscript(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::MdxJsxTextElement(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Link(node) if visit_link_labels && !is_url_label(node.url, &node.children) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::ThematicBreak(_)
            | Node::CodeBlock(_)
            | Node::MathBlock(_)
            | Node::Html(_)
            | Node::Text(_)
            | Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Break(_)
            | Node::Link(_)
            | Node::Image(_)
            | Node::FootnoteReference(_)
            | Node::Definition(_)
            | Node::MdxjsEsm(_)
            | Node::MdxFlowExpression(_)
            | Node::MdxTextExpression(_) => {}
        }
    }

    Ok(())
}

fn visit_inline_children<'arena, F>(
    nodes: &mut NodeList<'arena>,
    context: &TransformContext<'arena>,
    visit_link_labels: bool,
    raw_html_spans: &[Span],
    visit_text_runs: &mut F,
) -> Result<(), BoxError>
where
    F: FnMut(&mut NodeList<'arena>, &TransformContext<'arena>, &[Span]) -> Result<(), BoxError>,
{
    visit_text_runs(nodes, context, raw_html_spans)?;

    for node in nodes {
        match node {
            Node::Emphasis(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Strong(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Highlight(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Delete(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Superscript(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Subscript(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::MdxJsxTextElement(node) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::Link(node) if visit_link_labels && !is_url_label(node.url, &node.children) => {
                visit_inline_children(
                    &mut node.children,
                    context,
                    visit_link_labels,
                    raw_html_spans,
                    visit_text_runs,
                )?;
            }
            Node::ThematicBreak(_)
            | Node::Paragraph(_)
            | Node::Heading(_)
            | Node::BlockQuote(_)
            | Node::List(_)
            | Node::ListItem(_)
            | Node::CodeBlock(_)
            | Node::MathBlock(_)
            | Node::Html(_)
            | Node::Text(_)
            | Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Break(_)
            | Node::Link(_)
            | Node::Image(_)
            | Node::Table(_)
            | Node::DefinitionList(_)
            | Node::DefinitionListTerm(_)
            | Node::DefinitionListDefinition(_)
            | Node::FootnoteReference(_)
            | Node::Definition(_)
            | Node::FootnoteDefinition(_)
            | Node::MdxJsxFlowElement(_)
            | Node::MdxjsEsm(_)
            | Node::MdxFlowExpression(_)
            | Node::MdxTextExpression(_) => {}
        }
    }

    Ok(())
}

pub fn raw_html_text_spans(document: &Document<'_>, visit_link_labels: bool) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut open_tags = Vec::new();
    collect_raw_html_text_spans(
        &document.children,
        visit_link_labels,
        &mut open_tags,
        &mut spans,
    );
    spans
}

fn collect_raw_html_text_spans(
    nodes: &[Node<'_>],
    visit_link_labels: bool,
    open_tags: &mut Vec<String>,
    spans: &mut Vec<Span>,
) {
    for node in nodes {
        match node {
            Node::Text(text) => {
                if !open_tags.is_empty() && !text.span.is_empty() {
                    spans.push(text.span);
                }
            }
            Node::Html(html) => update_open_html_tags(html.value, open_tags),
            Node::Paragraph(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::Heading(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::BlockQuote(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::List(node) => {
                for item in &node.children {
                    collect_raw_html_text_spans(
                        &item.children,
                        visit_link_labels,
                        open_tags,
                        spans,
                    );
                }
            }
            Node::ListItem(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::Table(node) => {
                if let Some(attributes) = &node.attributes {
                    collect_raw_html_text_spans(
                        &attributes.caption,
                        visit_link_labels,
                        open_tags,
                        spans,
                    );
                }
                for row in &node.children {
                    for cell in &row.children {
                        collect_raw_html_text_spans(
                            &cell.children,
                            visit_link_labels,
                            open_tags,
                            spans,
                        );
                    }
                }
            }
            Node::DefinitionList(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::DefinitionListTerm(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::DefinitionListDefinition(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::FootnoteDefinition(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::MdxJsxFlowElement(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::Emphasis(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::Strong(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::Highlight(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::Delete(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::Superscript(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::Subscript(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::MdxJsxTextElement(node) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::Link(node) if visit_link_labels && !is_url_label(node.url, &node.children) => {
                collect_raw_html_text_spans(&node.children, visit_link_labels, open_tags, spans);
            }
            Node::ThematicBreak(_)
            | Node::CodeBlock(_)
            | Node::MathBlock(_)
            | Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Break(_)
            | Node::Link(_)
            | Node::Image(_)
            | Node::FootnoteReference(_)
            | Node::Definition(_)
            | Node::MdxjsEsm(_)
            | Node::MdxFlowExpression(_)
            | Node::MdxTextExpression(_) => {}
        }
    }
}

fn update_open_html_tags(value: &str, open_tags: &mut Vec<String>) {
    // Block HTML nodes contain whole lines or whole blocks, not a single
    // inline tag. Their contents are already opaque Html nodes in the AST.
    if value.ends_with('\n') {
        return;
    }
    let Some(tag) = parse_html_tag(value) else {
        return;
    };
    if tag.closing {
        if let Some(index) = open_tags.iter().rposition(|open| open == &tag.name) {
            open_tags.truncate(index);
        }
    } else if !tag.self_closing && !is_void_html_tag(&tag.name) {
        open_tags.push(tag.name);
    }
}

struct HtmlTag {
    name: String,
    closing: bool,
    self_closing: bool,
}

fn parse_html_tag(value: &str) -> Option<HtmlTag> {
    if !value.starts_with('<') || !value.ends_with('>') {
        return None;
    }
    let closing = value.starts_with("</");
    let start = if closing { 2 } else { 1 };
    let bytes = value.as_bytes();
    let mut end = start;
    while bytes
        .get(end)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
    {
        end += 1;
    }
    if end == start {
        return None;
    }
    Some(HtmlTag {
        name: value[start..end].to_ascii_lowercase(),
        closing,
        self_closing: !closing && value[..value.len() - 1].trim_end().ends_with('/'),
    })
}

fn is_void_html_tag(name: &str) -> bool {
    matches!(
        name,
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

fn is_url_label(url: &str, children: &[Node<'_>]) -> bool {
    matches!(children, [Node::Text(text)] if text.value == url)
}
