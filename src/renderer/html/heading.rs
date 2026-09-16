//! Heading text extraction and slug generation.
//!
//! Heading IDs and inline TOCs must agree on the same slug rules. This module owns the
//! shared text collector and slugifier so both code paths reuse the same Unicode-aware
//! normalization behavior.

use crate::ast::{Link, Node};

#[cfg(test)]
mod tests;

/// Class name on the opt-in heading permalink control.
///
/// Headings that already contain an `<a class="header-anchor">` or a `#`
/// link to the generated id do not receive a second marker.
pub const HEADING_PERMALINK_CLASS: &str = "header-anchor";

/// Collects heading text using the same rules as generated HTML IDs.
#[must_use]
pub fn collect_heading_text(nodes: &[Node<'_>]) -> String {
    let mut text = String::new();
    collect_heading_text_into(nodes, &mut text);
    text
}

pub(super) fn collect_heading_text_into(nodes: &[Node<'_>], text: &mut String) {
    for node in nodes {
        collect_node_text(node, text);
    }
}

/// Returns the heading's text when a single `Text` child carries all of it.
///
/// `## Configuration options` parses to exactly one `Text` node, and
/// [`collect_heading_text_into`] would do nothing but copy that node's `value`
/// into the destination. Recognizing the shape lets a caller read the source
/// slice directly and skip the copy; anything else (emphasis, inline code,
/// links, several children) still has to be concatenated.
pub(super) fn single_text_child<'a>(nodes: &[Node<'a>]) -> Option<&'a str> {
    match nodes {
        [Node::Text(text)] => Some(text.value),
        _ => None,
    }
}

fn collect_node_text(node: &Node<'_>, text: &mut String) {
    match node {
        Node::Text(value) => text.push_str(value.value),
        Node::InlineCode(value) => text.push_str(value.value),
        Node::Emphasis(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Strong(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Highlight(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Delete(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Superscript(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Subscript(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Link(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        _ => {}
    }
}

/// Returns the canonical fragment identifier used for a rendered heading.
#[must_use]
pub fn slugify_heading(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    slugify_heading_into(text, &mut out);
    out
}

/// Slugify `text` into `out`.
///
/// `out` is **not** cleared by this function. Renderers keep a long-lived
/// scratch buffer for heading IDs, clear it at the call site, and pass it back
/// here on every heading. That avoids allocating one temporary slug string per
/// heading while still leaving ownership decisions, such as cloning the final
/// unique id into a hash map, with the caller.
///
/// # Slug alphabet
///
/// Every character appended here is either `-` or passes `is_alphanumeric`
/// (`is_ascii_alphanumeric` on the ASCII path, `is_alphanumeric` on the
/// lowercased Unicode path), plus the literal `section` fallback. None of
/// `&`, `<`, `>`, `"`, `'`, CR, or LF is alphanumeric, and a multi-byte UTF-8
/// encoding never contains an ASCII byte, so a generated slug never holds a
/// byte that HTML attribute escaping would replace. `write_prepared_heading_id`
/// relies on this to write generated ids to the output without escaping them;
/// `heading::tests` pins the guarantee.
pub(super) fn slugify_heading_into(text: &str, out: &mut String) {
    // Single-pass slugify. The hot path is the all-ASCII byte loop: no UTF-8
    // decode and no `char::to_lowercase` iterator allocation per character.
    // We switch to the Unicode-aware char iterator only for contiguous
    // non-ASCII runs, preserving Japanese and other non-Latin heading text
    // without slowing down the common ASCII API-doc heading.
    let bytes = text.as_bytes();
    // One reserve covers an all-ASCII heading, which is the whole input in the
    // common case: the ASCII path emits at most one byte per input byte.
    // `push_ascii_run` still asks for its own run below, because the Unicode
    // path can grow the slug past `text.len()` (U+0130 lowercases to two
    // scalars, three bytes, from a two-byte input) and would otherwise eat the
    // headroom a later ASCII run relies on.
    out.reserve(text.len());
    let start_len = out.len();
    let mut last_was_separator = true;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] < 0x80 {
            i = push_ascii_run(bytes, i, out, &mut last_was_separator);
        } else {
            // Find the next ASCII boundary and process the multi-byte run
            // through the char iterator (handles Unicode case folding /
            // alphanumeric classification correctly).
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] >= 0x80 {
                j += 1;
            }
            push_unicode_run(&text[i..j], out, &mut last_was_separator);
            i = j;
        }
    }

    while out.len() > start_len && out.ends_with('-') {
        out.pop();
    }

    if out.len() == start_len {
        out.push_str("section");
    }
}

/// Appends the slug bytes for the ASCII run starting at `from`, returning the
/// index of the first non-ASCII byte (or `bytes.len()`).
///
/// `String::push` re-checks capacity and re-encodes UTF-8 for every character,
/// which is the bulk of the work for a 20-40 byte ASCII heading. The run's
/// length bounds the output exactly — an alphanumeric byte emits its lowercase
/// self and any other byte emits at most one `-` — so the capacity can be
/// claimed once and the bytes written through a plain cursor, with a single
/// `set_len` publishing the run. No stack buffer and no `from_utf8` are
/// involved: the bytes go straight to their final address.
#[allow(unsafe_code)]
fn push_ascii_run(
    bytes: &[u8],
    from: usize,
    out: &mut String,
    last_was_separator: &mut bool,
) -> usize {
    let mut end = from;
    while end < bytes.len() && bytes[end] < 0x80 {
        end += 1;
    }
    let run = &bytes[from..end];
    out.reserve(run.len());
    let mut separator = *last_was_separator;

    // SAFETY: `reserve` above guarantees at least `run.len()` spare bytes past
    // the current length, and no reallocation can happen between it and
    // `set_len` because `out` is only touched through `dst` in between. The
    // loop writes at most one byte per byte of `run`, so `written` never
    // exceeds `run.len()` and every write lands inside that spare region;
    // `set_len` then covers exactly the bytes written. Every written byte is
    // ASCII (`-`, or an ASCII alphanumeric with bit 5 forced on for the
    // uppercase letters), appended at the end of a `String`, which is a char
    // boundary, so the buffer stays valid UTF-8.
    unsafe {
        let vec = out.as_mut_vec();
        let at = vec.len();
        let dst = vec.as_mut_ptr().add(at);
        let mut written = 0usize;
        for &byte in run {
            if byte.is_ascii_alphanumeric() {
                // Bit 5 is the ASCII case bit. Forcing it on lowercases a
                // letter and leaves a digit alone, with no branch.
                *dst.add(written) = byte | 0x20;
                written += 1;
                separator = false;
            } else if !separator {
                *dst.add(written) = b'-';
                written += 1;
                separator = true;
            }
        }
        debug_assert!(written <= run.len());
        debug_assert!(at + written <= vec.capacity());
        vec.set_len(at + written);
    }

    *last_was_separator = separator;
    end
}

/// Appends the slug for one run of non-ASCII bytes.
///
/// Case folding and `is_alphanumeric` need real scalars, and a lowercase
/// mapping may be several characters long, so this run keeps the safe
/// `String::push` path.
fn push_unicode_run(run: &str, out: &mut String, last_was_separator: &mut bool) {
    for ch in run.chars() {
        for lower in ch.to_lowercase() {
            if lower.is_alphanumeric() {
                out.push(lower);
                *last_was_separator = false;
            } else if !*last_was_separator {
                out.push('-');
                *last_was_separator = true;
            }
        }
    }
}

pub(super) fn heading_has_permalink_marker(nodes: &[Node<'_>], id: &str) -> bool {
    nodes.iter().any(|node| node_has_permalink_marker(node, id))
}

fn node_has_permalink_marker(node: &Node<'_>, id: &str) -> bool {
    match node {
        Node::Link(link) => {
            is_hash_permalink_link(link, id) || heading_has_permalink_marker(&link.children, id)
        }
        Node::Html(html) => html_has_header_anchor(html.value),
        Node::Emphasis(value) => heading_has_permalink_marker(&value.children, id),
        Node::Strong(value) => heading_has_permalink_marker(&value.children, id),
        Node::Highlight(value) => heading_has_permalink_marker(&value.children, id),
        Node::Delete(value) => heading_has_permalink_marker(&value.children, id),
        Node::Superscript(value) => heading_has_permalink_marker(&value.children, id),
        Node::Subscript(value) => heading_has_permalink_marker(&value.children, id),
        _ => false,
    }
}

fn is_hash_permalink_link(link: &Link<'_>, id: &str) -> bool {
    let url = link.url;
    if url.len() != id.len() + 1 || !url.starts_with('#') || &url[1..] != id {
        return false;
    }
    collect_heading_text(&link.children) == "#"
}

fn html_has_header_anchor(value: &str) -> bool {
    let mut rest = value;
    while let Some(start) = rest.find("<a") {
        let tag = &rest[start..];
        let Some(end) = tag.find('>') else {
            break;
        };
        if class_attr_contains(&tag[..end], HEADING_PERMALINK_CLASS) {
            return true;
        }
        rest = &tag[end + 1..];
    }
    false
}

fn class_attr_contains(tag: &str, class_name: &str) -> bool {
    for quote in ['"', '\''] {
        let needle = if quote == '"' { "class=\"" } else { "class='" };
        if let Some(index) = tag.find(needle) {
            let after = &tag[index + needle.len()..];
            if let Some(end) = after.find(quote) {
                return after[..end]
                    .split_whitespace()
                    .any(|class| class == class_name);
            }
        }
    }
    false
}
