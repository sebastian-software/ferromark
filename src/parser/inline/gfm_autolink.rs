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
    ) {
        // Entity, escape, and unpaired-delimiter handling fragment plain
        // prose into adjacent text nodes; autolinks must see the joined
        // run (`...?q=x&hl=en` is one URL despite the `&` split).
        self.coalesce_adjacent_text(children);
        let mut rebuilt: Option<Vec<'a, Node<'a>>> = None;
        for index in 0..children.len() {
            let first_candidate = match &mut children[index] {
                Node::Span(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan);
                    None
                }
                Node::Emphasis(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan);
                    None
                }
                Node::Strong(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan);
                    None
                }
                Node::Highlight(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan);
                    None
                }
                Node::Delete(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan);
                    None
                }
                Node::Insertion(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan);
                    None
                }
                Node::Superscript(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan);
                    None
                }
                Node::Subscript(node) => {
                    self.apply_gfm_autolinks(&mut node.children, scan);
                    None
                }
                Node::Text(text) => find_candidate(text.value, scan),
                _ => None,
            };

            let Some(first_candidate) = first_candidate else {
                if let Some(out) = &mut rebuilt {
                    let placeholder = Node::Text(Text {
                        value: "",
                        span: Span::empty(),
                    });
                    out.push(std::mem::replace(&mut children[index], placeholder));
                }
                continue;
            };

            let Node::Text(text) = &children[index] else {
                unreachable!();
            };
            let full_value = text.value;
            let full_span = text.span;

            // Fast path when a single autolink covers the entire text node and
            // we have not needed to rebuild the sibling list yet.
            if rebuilt.is_none()
                && first_candidate.start == 0
                && first_candidate.end == full_value.len()
            {
                let link_span = Span::new(
                    full_span.start,
                    full_span.start + first_candidate.end as u32,
                );
                children[index] =
                    self.build_gfm_autolink_node(full_value, link_span, &first_candidate);
                continue;
            }

            let out = rebuilt.get_or_insert_with(|| {
                let mut out = self
                    .allocator
                    .new_vec_with_capacity(children.len().saturating_add(2));
                for prior in &mut children[..index] {
                    let placeholder = Node::Text(Text {
                        value: "",
                        span: Span::empty(),
                    });
                    out.push(std::mem::replace(prior, placeholder));
                }
                out
            });

            let mut cursor = 0usize;
            let mut pending = Some(first_candidate);
            while let Some(candidate) = pending
                .take()
                .or_else(|| find_candidate(&full_value[cursor..], scan))
            {
                let start = cursor + candidate.start;
                let end = cursor + candidate.end;
                if start > cursor {
                    out.push(Node::Text(Text {
                        value: &full_value[cursor..start],
                        span: Span::new(
                            full_span.start + cursor as u32,
                            full_span.start + start as u32,
                        ),
                    }));
                }
                let link_span =
                    Span::new(full_span.start + start as u32, full_span.start + end as u32);
                out.push(self.build_gfm_autolink_node(
                    &full_value[cursor..],
                    link_span,
                    &candidate,
                ));
                cursor = end;
            }
            if cursor < full_value.len() {
                out.push(Node::Text(Text {
                    value: &full_value[cursor..],
                    span: Span::new(full_span.start + cursor as u32, full_span.end),
                }));
            }
        }
        if let Some(rebuilt) = rebuilt {
            *children = rebuilt;
        }
    }

    fn build_gfm_autolink_node(
        &self,
        slice: &'a str,
        link_span: Span,
        candidate: &Candidate,
    ) -> Node<'a> {
        let link_value = &slice[candidate.start..candidate.end];
        let url: &'a str = if candidate.href_prefix.is_empty() {
            link_value
        } else {
            let mut url = crate::allocator::String::with_capacity_in(
                candidate.href_prefix.len() + link_value.len(),
                self.allocator.bump(),
            );
            url.push_str(candidate.href_prefix);
            url.push_str(link_value);
            url.into_bump_str()
        };

        let mut link_children = self.allocator.new_vec_with_capacity(1);
        link_children.push(Node::Text(Text {
            value: link_value,
            span: link_span,
        }));
        Node::Link(self.allocator.boxed(Link {
            url,
            title: None,
            attributes: None,
            children: link_children,
            span: link_span,
        }))
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
                    let mut total_len = 0usize;
                    for node in &children[run_start..read] {
                        if let Node::Text(text) = node {
                            total_len += text.value.len();
                        }
                    }
                    let mut merged = crate::allocator::String::with_capacity_in(
                        total_len,
                        self.allocator.bump(),
                    );
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
