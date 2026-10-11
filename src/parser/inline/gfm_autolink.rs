//! GFM autolink extension: bare `www.`, `http(s)://`, extended `mailto:`/
//! `xmpp:`, and email
//! addresses in plain text become links (GFM spec "Autolinks
//! (extension)").
//!
//! Runs as a post-pass over a parsed inline sequence so it cannot
//! interfere with emphasis pairing, and recurses into emphasis-like
//! containers while never descending into existing links.

use crate::allocator::Vec;
use crate::ast::{Link, Node, Span, Text};

mod candidate;
mod scan;

use self::candidate::find_candidate;
pub(super) use self::scan::may_contain_autolink;
use self::scan::valid_boundary;

/// Collects bare-link ranges so insertion delimiters inside a URL stay text.
///
/// This runs only when both insertions and GFM autolinks are enabled. The
/// normal autolink post-pass still creates the actual link nodes.
pub(super) fn collect_candidate_ranges<'arena>(
    value: &str,
    scan: AutolinkScan,
    ranges: &mut Vec<'arena, (usize, usize)>,
) {
    let mut from = 0;
    while from < value.len() {
        let Some(candidate) = find_candidate(&value[from..], scan) else {
            break;
        };
        let start = from + candidate.start;
        let end = from + candidate.end;
        if candidate.protect_plus_markers && valid_boundary(value, start) {
            // The inline parser can turn these bytes into nodes that the
            // autolink post-pass cannot see as one text run. Leave the range
            // before the first such byte, then search the rest of the raw
            // source for another scheme URL.
            let protected_end = value.as_bytes()[start..end]
                .iter()
                .position(|byte| matches!(byte, b'*' | b'_' | b'~' | b'`' | b'<' | b'[' | b'\\'))
                .map_or(end, |relative| start + relative);
            if protected_end > start {
                ranges.push((start, protected_end));
            }
            from = if protected_end < end {
                protected_end
            } else {
                end
            };
        } else {
            from = end;
        }
    }
}

use crate::parser::Parser;

pub(super) struct Candidate {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) href_prefix: &'static str,
    /// Only scheme URLs need raw-source protection before the inline pass.
    /// Fuzzy email and `www.` candidates are linkified after parsing, so
    /// insertion markers in those candidates must remain active.
    pub(super) protect_plus_markers: bool,
}

/// What the block-level pre-flight proved about a block's raw content, so
/// the per-node scan does not re-derive it.
///
/// `www.` is the only needle with no `:` and no `@` in it, which makes it
/// the only reason a node has to pay for a substring search when the cheap
/// `memchr2` comes back empty. Whether a block can hold one is a property
/// of its raw source, so [`may_contain_autolink`] answers it once for the
/// whole block instead of once per text node.
#[derive(Clone, Copy)]
pub(in crate::parser::inline) struct AutolinkScan {
    pub(super) may_have_www: bool,
    pub(super) may_have_extended: bool,
}

impl<'a> Parser<'a> {
    pub(in crate::parser::inline) fn apply_gfm_autolinks(
        &self,
        children: &mut Vec<'a, Node<'a>>,
        scan: AutolinkScan,
        content: &str,
        offset: usize,
    ) {
        // Entity, escape, and unpaired-delimiter handling fragment plain
        // prose into adjacent text nodes; autolinks must see the joined
        // run (`...?q=x&hl=en` is one URL despite the `&` split).
        self.coalesce_adjacent_text(children);
        let mut i = 0;
        while i < children.len() {
            match &mut children[i] {
                Node::Span(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan, content, offset);
                }
                Node::Emphasis(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan, content, offset);
                }
                Node::Strong(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan, content, offset);
                }
                Node::Highlight(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan, content, offset);
                }
                Node::Delete(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan, content, offset);
                }
                Node::Insertion(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan, content, offset);
                }
                Node::Superscript(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan, content, offset);
                }
                Node::Subscript(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan, content, offset);
                }
                Node::Text(text) => {
                    let value = text.value;
                    if let Some(candidate) = find_candidate(value, scan) {
                        let span_start = text.span.start;
                        let link_value = &value[candidate.start..candidate.end];
                        let url: &'a str = if candidate.href_prefix.is_empty() {
                            link_value
                        } else {
                            let mut url = self.allocator.new_string_from(candidate.href_prefix);
                            url.push_str(link_value);
                            url.into_bump_str()
                        };

                        // Decoding entities and punctuation escapes changes byte counts.
                        // Map through the original inline source before its container map.
                        let raw =
                            &content[span_start as usize - offset..text.span.end as usize - offset];
                        let source_start = decoded_boundary(raw, candidate.start, false);
                        let source_end = decoded_boundary(raw, candidate.end, true);
                        let link_span = Span::new(
                            span_start + source_start as u32,
                            span_start + source_end as u32,
                        );
                        let mut link_children = self.allocator.new_vec();
                        link_children.push(Node::Text(Text {
                            value: link_value,
                            span: link_span,
                        }));
                        let link_node = Node::Link(self.allocator.boxed(Link {
                            url,
                            title: None,
                            attributes: None,
                            children: link_children,
                            span: link_span,
                        }));

                        let after = &value[candidate.end..];
                        let after_node = (!after.is_empty()).then(|| {
                            Node::Text(Text {
                                value: after,
                                span: Span::new(link_span.end, text.span.end),
                            })
                        });

                        if candidate.start == 0 {
                            children[i] = link_node;
                        } else {
                            text.value = &value[..candidate.start];
                            text.span = Span::new(span_start, link_span.start);
                            i += 1;
                            children.insert(i, link_node);
                        }
                        if let Some(after_node) = after_node {
                            children.insert(i + 1, after_node);
                        }
                        // Continue scanning in the remainder text node.
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}

impl<'a> Parser<'a> {
    /// Merges runs of adjacent `Text` nodes into one.
    ///
    /// Each run used to be `drain`ed and the merged node `insert`ed, and both
    /// shift every node after the run. A paragraph that alternates text with
    /// something else is all runs — `@[a](b) ` repeated is exactly that — so
    /// the shifting cost O(n²): 256 KiB took 1.6 s and grew x19 for every x4
    /// of input. A write cursor touches each node once.
    fn coalesce_adjacent_text(&self, children: &mut Vec<'a, Node<'a>>) {
        let mut write = 0usize;
        let mut read = 0usize;

        while read < children.len() {
            let run_start = read;
            while read < children.len() && matches!(children[read], Node::Text(_)) {
                read += 1;
            }

            match read - run_start {
                // Not text at all: keep it, at the cursor.
                0 => {
                    children.swap(write, read);
                    read += 1;
                }
                // A lone text node has nothing to merge with.
                1 => children.swap(write, run_start),
                _ => {
                    let mut merged = self.allocator.new_string();
                    let mut span = Span::new(0, 0);
                    for (index, node) in children[run_start..read].iter().enumerate() {
                        if let Node::Text(text) = node {
                            if index == 0 {
                                span = text.span;
                            }
                            span = Span::new(span.start, text.span.end);
                            merged.push_str(text.value);
                        }
                    }
                    children[write] = Node::Text(Text {
                        value: merged.into_bump_str(),
                        span,
                    });
                }
            }
            write += 1;
        }

        children.truncate(write);
    }
}

// Maps a decoded text boundary to its authored byte boundary. An entity that
// expands to several bytes is indivisible in the source; partial ranges cover
// that complete entity. This changes only spans, never autolink recognition.
fn decoded_boundary(raw: &str, target: usize, end: bool) -> usize {
    use super::entity::{EntityValue, scan_entity};
    let mut authored = 0;
    let mut decoded = 0;
    while decoded < target && authored < raw.len() {
        let rest = &raw[authored..];
        let bytes = rest.as_bytes();
        let (source_len, value_len) = if bytes[0] == b'&'
            && let Some((entity, consumed)) = scan_entity(rest)
        {
            let len = match entity {
                EntityValue::Named(value) => value.len(),
                EntityValue::Char(value) => value.len_utf8(),
            };
            (consumed, len)
        } else if bytes[0] == b'\\' && bytes.get(1).is_some_and(u8::is_ascii_punctuation) {
            (2, 1)
        } else {
            let len = rest.chars().next().map_or(0, char::len_utf8);
            (len, len)
        };
        if decoded + value_len > target {
            return if end { authored + source_len } else { authored };
        }
        authored += source_len;
        decoded += value_len;
    }
    authored
}
