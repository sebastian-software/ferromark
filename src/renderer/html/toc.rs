//! Inline table-of-contents discovery.
//!
//! A standalone `[[toc]]` paragraph is rendered as a flat navigation list. This module
//! pre-collects headings only when such a marker exists, preserving duplicate-heading
//! ID behavior while avoiding allocation in documents without TOC markers.

use crate::ast::{Document, ListItem, Node, Paragraph};

use super::heading::collect_heading_text;
use super::heading_ids::{HeadingIdPlanner, heading_id_base};

#[derive(Debug, Clone)]
pub(super) struct InlineTocEntry {
    pub(super) depth: u8,
    pub(super) text: String,
    pub(super) id: String,
}

/// Cheap document-level facts needed at `HtmlRenderer::render` entry.
///
/// Rendering already scans for `[[toc]]` before deciding whether to collect
/// TOC entries. Counting headings in the same traversal lets the renderer
/// reserve the heading-id map once, avoiding incremental hash-map growth
/// while preserving the lazy TOC behavior for documents without a marker.
pub(super) struct DocumentRenderScan {
    pub(super) has_toc_marker: bool,
    pub(super) has_footnotes: bool,
    pub(super) heading_count: usize,
}

impl DocumentRenderScan {
    /// The result of a walk that found nothing, and the starting point of one
    /// that has not run. A renderer whose options read neither fact uses this
    /// instead of walking: `reserve(0)` is a no-op and "no marker" is what a
    /// disabled inline TOC would have concluded anyway.
    pub(super) const NONE: Self = Self {
        has_toc_marker: false,
        has_footnotes: false,
        heading_count: 0,
    };
}

pub(super) fn collect_inline_toc_entries(
    document: &Document<'_>,
    max_depth: u8,
    ids: &mut HeadingIdPlanner,
    entries: &mut Vec<InlineTocEntry>,
) {
    for node in &document.children {
        collect_inline_toc_node(node, max_depth, ids, entries);
    }
}

/// Cheap, allocation-free scan for renderer setup.
///
/// The TOC directive is recognized only when a paragraph's text content trims
/// to exactly `[[toc]]`, which is also the only form `visit_paragraph` will
/// render as a TOC. `detect_toc_marker` is false when the renderer would
/// ignore a marker anyway — an inline TOC needs heading IDs to link to, so
/// both conveniences must be on — which drops the per-paragraph marker
/// predicate from the walk without changing what the renderer concludes.
///
/// The walk still descends into every container, because a marker paragraph
/// or heading nested in a block quote, list, footnote definition, or MDX
/// element counts exactly like a top-level one.
pub(super) fn scan_document_for_render(
    document: &Document<'_>,
    detect_toc_marker: bool,
) -> DocumentRenderScan {
    let mut scan = DocumentRenderScan::NONE;
    for node in &document.children {
        scan_node_for_render(node, detect_toc_marker, &mut scan);
    }
    scan
}

fn scan_node_for_render(node: &Node<'_>, detect_toc_marker: bool, scan: &mut DocumentRenderScan) {
    // This traversal intentionally collects only facts that are free to derive:
    // "does any paragraph equal the TOC marker?" and "how many headings exist?".
    // It does not slugify headings or collect text. That keeps the no-TOC
    // render path allocation-free while still giving `HtmlRenderer::render`
    // enough information to reserve the heading-id map up front.
    match node {
        Node::Heading(_) => scan.heading_count += 1,
        Node::Paragraph(p)
            if detect_toc_marker && !scan.has_toc_marker && is_toc_marker_paragraph(p) =>
        {
            scan.has_toc_marker = true;
        }
        Node::Paragraph(_) => {}
        Node::BlockQuote(bq) => {
            for child in &bq.children {
                scan_node_for_render(child, detect_toc_marker, scan);
            }
        }
        Node::List(list) => {
            for item in &list.children {
                scan_list_item_for_render(item, detect_toc_marker, scan);
            }
        }
        Node::ListItem(item) => scan_list_item_for_render(item, detect_toc_marker, scan),
        Node::FootnoteDefinition(def) => {
            scan.has_footnotes = true;
            for child in &def.children {
                scan_node_for_render(child, detect_toc_marker, scan);
            }
        }
        Node::MdxJsxFlowElement(node) => {
            for child in &node.children {
                scan_node_for_render(child, detect_toc_marker, scan);
            }
        }
        Node::MdxJsxTextElement(node) => {
            for child in &node.children {
                scan_node_for_render(child, detect_toc_marker, scan);
            }
        }
        _ => {}
    }
}

fn scan_list_item_for_render(
    item: &ListItem<'_>,
    detect_toc_marker: bool,
    scan: &mut DocumentRenderScan,
) {
    for child in &item.children {
        scan_node_for_render(child, detect_toc_marker, scan);
    }
}

pub(super) fn is_toc_marker_paragraph(paragraph: &Paragraph<'_>) -> bool {
    // Equivalent to the prior
    // `collect_text_nodes_only(...).is_some_and(|t| t.trim() == "[[toc]]")`
    // check, but allocation-free: bails on the first non-Text child and
    // matches the marker byte-by-byte against the concatenated text. Note
    // that the inline parser emits the literal "[[toc]]" as three Text
    // nodes (`[`, `[`, `toc]]`) because the bracket-as-link path fails
    // open — so a "single Text child only" shortcut would miss it.
    //
    // The match is case-insensitive: the directive names itself, and a page
    // written `[[TOC]]` meant the same thing as one written `[[toc]]`. The
    // marker is ASCII, so lowercasing a byte at a time is sound.
    const MARKER: &[u8] = b"[[toc]]";
    let mut matched = 0usize;
    let mut after_marker_ws = false;

    for child in &paragraph.children {
        let Node::Text(text) = child else {
            return false;
        };
        for &byte in text.value.as_bytes() {
            let is_ws = matches!(byte, b' ' | b'\t' | b'\n' | b'\r');
            if is_ws {
                if matched > 0 {
                    after_marker_ws = true;
                }
                continue;
            }
            if after_marker_ws
                || matched == MARKER.len()
                || byte.to_ascii_lowercase() != MARKER[matched]
            {
                return false;
            }
            matched += 1;
        }
    }

    matched == MARKER.len()
}

fn collect_inline_toc_node(
    node: &Node<'_>,
    max_depth: u8,
    ids: &mut HeadingIdPlanner,
    entries: &mut Vec<InlineTocEntry>,
) {
    match node {
        Node::Heading(heading) => {
            let include_heading = heading.depth <= max_depth;
            let text = collect_heading_text(&heading.children);
            let base = heading_id_base(heading);
            let id = ids.unique_id(&base).to_string();
            if include_heading {
                entries.push(InlineTocEntry {
                    depth: heading.depth,
                    text,
                    id,
                });
            }
        }
        Node::BlockQuote(block_quote) => {
            for child in &block_quote.children {
                collect_inline_toc_node(child, max_depth, ids, entries);
            }
        }
        Node::List(list) => {
            for item in &list.children {
                for child in &item.children {
                    collect_inline_toc_node(child, max_depth, ids, entries);
                }
            }
        }
        Node::ListItem(item) => {
            for child in &item.children {
                collect_inline_toc_node(child, max_depth, ids, entries);
            }
        }
        Node::FootnoteDefinition(definition) => {
            for child in &definition.children {
                collect_inline_toc_node(child, max_depth, ids, entries);
            }
        }
        Node::MdxJsxFlowElement(node) => {
            for child in &node.children {
                collect_inline_toc_node(child, max_depth, ids, entries);
            }
        }
        Node::MdxJsxTextElement(node) => {
            for child in &node.children {
                collect_inline_toc_node(child, max_depth, ids, entries);
            }
        }
        _ => {}
    }
}
