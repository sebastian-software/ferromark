//! Heading text extraction and slug generation.
//!
//! Heading IDs and heading-based tooling must agree on the same slug rules. This module
//! owns the shared text collector and slugifier so both code paths reuse the same
//! Unicode-aware normalization behavior.

use std::collections::hash_map::Entry;
use std::fmt;
use std::fmt::Write as _;
use std::hash::Hasher as _;

use crate::ast::{Link, Node};
use rustc_hash::{FxHashMap, FxHasher};

#[cfg(test)]
pub(super) mod planner_tests;
#[cfg(test)]
mod tests;

/// Class name on the opt-in heading permalink control.
///
/// Headings that already contain an `<a class="header-anchor">` or a `#`
/// link to the generated id do not receive a second marker.
pub const HEADING_PERMALINK_CLASS: &str = "header-anchor";

/// Maps a Markdown heading level to its rendered level after applying `offset`.
///
/// Levels are clamped to HTML's `h1` through `h6` range after the offset is
/// applied. Offsets outside that range therefore produce `h1` or `h6` rather
/// than an invalid heading element.
#[must_use]
pub fn map_heading_level(level: u8, offset: i32) -> u8 {
    let shifted = i64::from(level.clamp(1, 6)) + i64::from(offset);
    u8::try_from(shifted.clamp(1, 6)).unwrap_or(1)
}

/// Error returned when a configured heading ID prefix contains unsafe bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidHeadingIdPrefix;

impl fmt::Display for InvalidHeadingIdPrefix {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "heading ID prefixes may contain only ASCII letters, digits, underscores, and hyphens",
        )
    }
}

impl std::error::Error for InvalidHeadingIdPrefix {}

/// Assigns unique heading IDs in document order.
///
/// The first request for a base ID keeps it. Later requests try `-1`, `-2`,
/// and so on, skipping any ID already claimed by an earlier heading. Explicit
/// heading IDs use the same rule as generated slugs.
///
/// The planner is shared by HTML rendering and derived heading metadata so
/// their IDs stay in sync. Clear it before planning a new document; keep it
/// between incremental fragments when their IDs must remain unique together.
#[derive(Debug, Clone, Default)]
pub struct HeadingIdPlanner {
    /// Every claimed ID, back to back in claim order.
    ///
    /// The candidate for the next claim is written at the end of this buffer
    /// and stays there once claimed, so a claimed ID costs no allocation of
    /// its own, and `clear` keeps the capacity for the next document instead
    /// of freeing one key per heading. A claimed range is never moved or
    /// rewritten before `clear`.
    ids: String,
    /// One record per claimed ID, in claim order.
    claims: Vec<Claim>,
    /// Hash of a claimed ID to the newest claim with that hash.
    ///
    /// The ID bytes are hashed once per candidate, and the `u64` key needs no
    /// allocation. Claims whose IDs share a full 64-bit hash chain through
    /// `Claim::previous_with_hash`, so equality is always decided on the
    /// bytes.
    by_hash: FxHashMap<u64, usize>,
}

/// One claimed heading ID.
#[derive(Debug, Clone, Copy)]
struct Claim {
    /// Range of the ID in `HeadingIdPlanner::ids`.
    start: usize,
    end: usize,
    /// The next `-N` suffix to try when this ID is requested again as a base.
    next_suffix: usize,
    /// An older claim with the same hash, or `NO_CLAIM`.
    previous_with_hash: usize,
}

const NO_CLAIM: usize = usize::MAX;

/// Initial capacity of the planner's ID storage.
///
/// Replaces the two 64-byte scratch buffers the renderer used to reserve for
/// the slug and the unique ID, and covers the IDs of a typical document
/// without growing.
const ID_STORAGE_CAPACITY: usize = 256;

/// A claimed heading ID, valid until the planner is cleared.
///
/// Read it back with `HeadingIdPlanner::id`.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PlannedId {
    start: usize,
    end: usize,
}

impl HeadingIdPlanner {
    /// Creates an empty heading ID planner.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears IDs claimed for the previous document while retaining capacity.
    pub fn clear(&mut self) {
        self.ids.clear();
        self.claims.clear();
        self.by_hash.clear();
    }

    /// Plans a unique ID based on `base`, returning it as an owned string.
    #[must_use]
    pub fn plan(&mut self, base: &str) -> String {
        let id = self.claim(base);
        self.id(id).to_owned()
    }

    /// Writes a unique ID based on `base` into `output`.
    ///
    /// The buffer is cleared before writing and can be reused across headings.
    #[inline]
    pub fn plan_into(&mut self, base: &str, output: &mut String) {
        let id = self.claim(base);
        output.clear();
        output.push_str(self.id(id));
    }

    /// Plans a unique ID based on `base` and keeps it in the planner.
    pub(super) fn claim(&mut self, base: &str) -> PlannedId {
        let start = self.begin_candidate();
        self.ids.push_str(base);
        self.claim_candidate(start)
    }

    /// Plans a unique ID based on the slug of `text`.
    ///
    /// Equivalent to `claim(&slugify_heading(text))`, but the slug is written
    /// straight into the planner's ID storage: a heading whose slug is not
    /// taken yet is claimed where it was written, without a copy.
    pub(super) fn claim_slug(&mut self, text: &str) -> PlannedId {
        let start = self.begin_candidate();
        slugify_heading_into(text, &mut self.ids);
        self.claim_candidate(start)
    }

    /// Returns a claimed ID.
    #[inline]
    pub(super) fn id(&self, id: PlannedId) -> &str {
        &self.ids[id.start..id.end]
    }

    fn begin_candidate(&mut self) -> usize {
        if self.ids.capacity() == 0 {
            self.ids.reserve(ID_STORAGE_CAPACITY);
        }
        self.ids.len()
    }

    /// Claims the base written at `ids[start..]`, or the first free `-N`
    /// variant of it.
    fn claim_candidate(&mut self, start: usize) -> PlannedId {
        let taken = match self.claim_if_free(start) {
            Ok(id) => return id,
            Err(taken) => taken,
        };

        // `ids[start..base_end]` is a copy of the taken base. Each try
        // replaces the previous suffix after it, so the claimed variant ends
        // up where the base was written.
        let base_end = self.ids.len();
        let mut suffix = self.claims[taken].next_suffix;
        loop {
            self.ids.truncate(base_end);
            let _ = write!(self.ids, "-{suffix}");
            suffix = suffix.saturating_add(1);
            if let Ok(id) = self.claim_if_free(start) {
                // The base's own record says where the next duplicate starts
                // looking, exactly like the first duplicate did.
                self.claims[taken].next_suffix = suffix;
                return id;
            }
        }
    }

    /// Claims `ids[start..]` when no earlier claim holds the same ID.
    ///
    /// Returns the new claim, or the index of the claim that already holds
    /// the candidate. One hash and one table probe answer both "is it taken?"
    /// and "claim it" for a free candidate.
    fn claim_if_free(&mut self, start: usize) -> Result<PlannedId, usize> {
        let Self {
            ids,
            claims,
            by_hash,
        } = self;
        let candidate = &ids[start..];
        let hash = hash_id(candidate);
        let index = claims.len();
        let previous_with_hash = match by_hash.entry(hash) {
            Entry::Vacant(slot) => {
                slot.insert(index);
                NO_CLAIM
            }
            Entry::Occupied(mut slot) => {
                let newest = *slot.get();
                let mut cursor = newest;
                while cursor != NO_CLAIM {
                    let claim = &claims[cursor];
                    if &ids[claim.start..claim.end] == candidate {
                        return Err(cursor);
                    }
                    cursor = claim.previous_with_hash;
                }
                slot.insert(index);
                newest
            }
        };
        let end = ids.len();
        claims.push(Claim {
            start,
            end,
            next_suffix: 1,
            previous_with_hash,
        });
        Ok(PlannedId { start, end })
    }
}

fn hash_id(id: &str) -> u64 {
    let mut hasher = FxHasher::default();
    hasher.write(id.as_bytes());
    hasher.finish() & hash_mask()
}

/// Keeps every hash bit. Distinct IDs practically never share a 64-bit hash,
/// so tests narrow the mask to drive the collision chain on purpose.
#[cfg(not(test))]
const fn hash_mask() -> u64 {
    u64::MAX
}

#[cfg(test)]
fn hash_mask() -> u64 {
    planner_tests::HASH_MASK.with(std::cell::Cell::get)
}

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
/// `out` is **not** cleared by this function; the slug is appended, and the
/// trailing trim and the `section` fallback only look at the appended part.
/// `HeadingIdPlanner::claim_slug` relies on this to write each slug straight
/// after the IDs it has already claimed, and footnotes slugify into a reused
/// scratch buffer, so neither allocates a temporary slug string per call.
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

/// Whether the heading already links to the emitted ID `prefix` + `id`.
///
/// The emitted ID is passed in two parts so the renderer can check it without
/// concatenating the configured prefix and the planned ID.
pub(super) fn heading_has_permalink_marker(nodes: &[Node<'_>], prefix: &str, id: &str) -> bool {
    nodes
        .iter()
        .any(|node| node_has_permalink_marker(node, prefix, id))
}

fn node_has_permalink_marker(node: &Node<'_>, prefix: &str, id: &str) -> bool {
    match node {
        Node::Link(link) => {
            is_hash_permalink_link(link, prefix, id)
                || heading_has_permalink_marker(&link.children, prefix, id)
        }
        Node::Html(html) => html_has_header_anchor(html.value),
        Node::Emphasis(value) => heading_has_permalink_marker(&value.children, prefix, id),
        Node::Strong(value) => heading_has_permalink_marker(&value.children, prefix, id),
        Node::Highlight(value) => heading_has_permalink_marker(&value.children, prefix, id),
        Node::Delete(value) => heading_has_permalink_marker(&value.children, prefix, id),
        Node::Superscript(value) => heading_has_permalink_marker(&value.children, prefix, id),
        Node::Subscript(value) => heading_has_permalink_marker(&value.children, prefix, id),
        _ => false,
    }
}

fn is_hash_permalink_link(link: &Link<'_>, prefix: &str, id: &str) -> bool {
    // The URL is exactly `#` + prefix + id.
    let links_to_id = link
        .url
        .strip_prefix('#')
        .and_then(|fragment| fragment.strip_prefix(prefix))
        == Some(id);
    links_to_id && collect_heading_text(&link.children) == "#"
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
