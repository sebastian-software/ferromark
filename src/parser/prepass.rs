//! Document-wide definition discovery.
//!
//! A cheap syntax-shape filter keeps ordinary documents out of collection.
//! Link definitions and footnote labels are collected by the real block
//! grammar, so root and container definitions share context and scope.

use std::rc::Rc;
use std::sync::LazyLock;

use memchr::{memchr, memmem, memrchr, memrchr2};

use super::Parser;
use super::footnote::FootnoteLabels;
use super::line_scan::is_line_ending_byte;
use super::reference::ReferenceMap;

/// Three-byte fence-run searchers, built once for the process.
///
/// A closing fence line holds at least three of its fence byte in a row, so
/// the needle is a necessary condition for one — enough to skip straight to
/// the first line that could possibly close an open fence. The one-shot
/// `memmem::find` would rebuild its SIMD prefilter for every fence in the
/// document.
static BACKTICK_RUN: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("```"));
static TILDE_RUN: LazyLock<memmem::Finder<'static>> = LazyLock::new(|| memmem::Finder::new("~~~"));

/// Definition-closer searcher, built once for the process, for the same
/// reason: a short document spends more time building a one-shot finder's
/// prefilter than searching with it.
static DEFINITION_CLOSER: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("]:"));

/// Start of the first line at or after `from` holding a run of three
/// `fence_byte`s, or `None` when the rest of the source holds none.
pub(super) fn next_fence_run_line(bytes: &[u8], from: usize, fence_byte: u8) -> Option<usize> {
    // `from` is one past a newline, which is one past the end when the last
    // line of the document is unterminated.
    let from = from.min(bytes.len());
    let finder = if fence_byte == b'`' {
        &*BACKTICK_RUN
    } else {
        &*TILDE_RUN
    };
    let at = from + finder.find(&bytes[from..])?;
    // `from` is a line start, so the match's line starts at or after it.
    Some(memrchr2(b'\n', b'\r', &bytes[from..at]).map_or(from, |off| from + off + 1))
}

/// Furthest an opening `[` can sit before the `]:` that closes its label:
/// `[`, at most 1,000 label bytes, and `]` span 1,002 bytes, so the `]` of a
/// well-formed reference definition is at most 1,001 bytes past its `[`.
const MAX_LABEL_SPAN: usize = 1001;

/// Whether the source contains the minimum shape of a definition opener.
///
/// A bare `]:` anywhere is not enough: ordinary prose can mention the token
/// and force the much more expensive structural pre-pass. The opening `[` of
/// either a reference or footnote definition must begin a block line after
/// optional container prefixes. The returned flags indicate any candidate
/// and a possible link definition (rather than only enabled footnote labels).
/// Reference labels are capped at
/// 1,000 bytes; footnote labels are line-bounded but have no length cap. This
/// scanner only proves that necessary shape exists; the full pre-pass remains
/// responsible for validating syntax and block context.
///
/// The scan is driven from the `]:` side. A document that holds one anywhere
/// used to make every `[` in it pay for a backwards line search and a forward
/// label search, which is the expensive direction on link-dense prose: `[` is
/// the common byte and `]:` the rare one. Walking the `]:` occurrences instead
/// bounds the work per occurrence to the label window before it, and the
/// window start only moves forwards, so an opener is judged once no matter how
/// many `]:` follow it.
fn definition_candidates(source: &str, footnotes: bool, mdx: bool) -> (bool, bool) {
    let bytes = source.as_bytes();
    if memchr(b'[', bytes).is_none() {
        return (false, false);
    }

    let mut found = false;
    // Openers before this offset have already been judged. Their verdict
    // cannot improve for a later `]:`: the length window only moves forwards,
    // and a footnote label that already failed to share a line with one `]:`
    // shares even less with the next.
    let mut judged = 0;
    let mut from = 0;
    while let Some(offset) = DEFINITION_CLOSER.find(&bytes[from..]) {
        let closer = from + offset;
        // `]:` cannot overlap itself, so the next search starts past it.
        from = closer + 2;
        let label_start = closer.saturating_sub(MAX_LABEL_SPAN);
        // Footnote labels cannot span lines, but unlike reference labels their
        // parser deliberately has no length cap, so their window is the line
        // rather than the label span. Once one is found the wider window has
        // nothing left to prove.
        let line_start = (footnotes && !found).then(|| previous_line_start(bytes, closer));
        let window_start = line_start.map_or(label_start, |start| start.min(label_start));
        let mut at = window_start.max(judged);
        while let Some(offset) = memchr(b'[', &bytes[at..closer]) {
            let open = at + offset;
            at = open + 1;
            if !opens_block_line(bytes, open, mdx) {
                continue;
            }
            if footnotes && bytes.get(open + 1) == Some(&b'^') {
                if line_start.is_some_and(|start| open >= start) {
                    found = true;
                }
            } else if open >= label_start {
                // All link definitions use the block grammar, so there is no
                // need to search for a later container candidate.
                return (true, true);
            }
        }
        judged = closer;
    }
    (found, false)
}

/// Whether the bytes before `open` on its line could be a container prefix.
///
/// A necessary shape, not a container grammar. False positives (for example
/// indented code) are rejected by the real block parser. Under MDX a flow
/// component can begin Markdown children on its opening line, so a preceding
/// tag end also counts; the block grammar decides whether it really belongs to
/// JSX rather than raw HTML or prose.
///
/// The walk runs backwards from `open` and stops at the first byte that
/// settles the question, so ordinary prose pays for one byte.
fn opens_block_line(bytes: &[u8], open: usize, mdx: bool) -> bool {
    let mut at = open;
    while at > 0 {
        let byte = bytes[at - 1];
        if is_block_prefix_byte(byte) {
            if mdx && byte == b'>' {
                return true;
            }
            at -= 1;
            continue;
        }
        // A line ending means the whole prefix was container bytes.
        if is_line_ending_byte(byte) {
            return true;
        }
        // Everything between here and `open` was a container byte, and none of
        // it was a tag end, so only the rest of the line can still qualify.
        return mdx && memrchr(b'>', &bytes[previous_line_start(bytes, at)..at - 1]).is_some();
    }
    true
}

const fn is_block_prefix_byte(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\t' | b'>' | b'-' | b'+' | b'*' | b'.' | b')' | b'0'..=b'9'
    )
}

fn previous_line_start(bytes: &[u8], before: usize) -> usize {
    memrchr2(b'\n', b'\r', &bytes[..before]).map_or(0, |off| off + 1)
}

impl<'a> Parser<'a> {
    /// Collects document-wide facts before resolving inline references.
    /// Absent maps stay `None`, avoiding shared allocations on ordinary input.
    pub(super) fn build_prepass(
        &self,
    ) -> (Option<Rc<ReferenceMap<'a>>>, Option<Rc<FootnoteLabels>>) {
        if !self.options.allow_link_refs && !self.options.footnotes {
            return (None, None);
        }
        let (has_candidate, has_link_candidate) =
            definition_candidates(self.source, self.options.footnotes, self.options.mdx);
        if !has_candidate {
            return (None, None);
        }
        let (definitions, labels) = if (self.options.allow_link_refs && has_link_candidate)
            || (self.options.footnotes && self.source.contains("[^"))
        {
            self.collect_definitions()
        } else {
            (ReferenceMap::default(), FootnoteLabels::default())
        };
        (
            (!definitions.is_empty()).then(|| Rc::new(definitions)),
            (!labels.is_empty()).then(|| Rc::new(labels)),
        )
    }
}

#[cfg(test)]
mod tests {
    // Owned strings keep the test oracle independent of production storage.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use memchr::{memchr, memmem, memrchr2};

    use crate::parser::line_scan::line_end as scan_line_end;

    use super::previous_line_start;

    /// The original opener-driven walk, kept as an independent oracle for the
    /// `]:`-driven scan. It examines every `[` in the document and searches
    /// forward from each one for the `]:` that could close its label.
    fn oracle(source: &str, footnotes: bool, mdx: bool) -> (bool, bool) {
        let bytes = source.as_bytes();
        let Some(mut open) = memchr(b'[', bytes) else {
            return (false, false);
        };
        if memmem::find(bytes, b"]:").is_none() {
            return (false, false);
        }

        let mut found = false;
        let mut line_start = previous_line_start(bytes, open);
        loop {
            let raw_prefix = &source[line_start..open];
            let block_prefix = raw_prefix.bytes().all(|byte| {
                matches!(
                    byte,
                    b' ' | b'\t' | b'>' | b'-' | b'+' | b'*' | b'.' | b')' | b'0'..=b'9'
                )
            });
            if block_prefix || (mdx && raw_prefix.contains('>')) {
                let candidate_end = if footnotes && bytes.get(open + 1) == Some(&b'^') {
                    scan_line_end(bytes, open + 2)
                } else {
                    open.saturating_add(1003).min(bytes.len())
                };
                if memmem::find(&bytes[open + 1..candidate_end], b"]:").is_some() {
                    if footnotes && bytes.get(open + 1) == Some(&b'^') {
                        found = true;
                    } else {
                        return (true, true);
                    }
                }
            }

            let search_start = open + 1;
            let Some(next) = memchr(b'[', &bytes[search_start..]) else {
                return (found, false);
            };
            open = search_start + next;
            if memrchr2(b'\n', b'\r', &bytes[search_start..open]).is_some() {
                line_start = previous_line_start(bytes, open);
            }
        }
    }

    #[track_caller]
    fn check_against_oracle(source: &str) {
        for footnotes in [false, true] {
            for mdx in [false, true] {
                assert_eq!(
                    super::definition_candidates(source, footnotes, mdx),
                    oracle(source, footnotes, mdx),
                    "footnotes {footnotes}, mdx {mdx}, source {source:?}"
                );
            }
        }
    }

    fn definition_candidates(source: &str, footnotes: bool) -> (bool, bool) {
        super::definition_candidates(source, footnotes, false)
    }

    fn has_definition_candidate(source: &str, footnotes: bool) -> bool {
        definition_candidates(source, footnotes).0
    }

    #[test]
    fn footnote_only_candidates_do_not_require_link_collection() {
        assert_eq!(
            definition_candidates("[^note]: body\n\n[^note]", true),
            (true, false)
        );
        assert_eq!(
            definition_candidates("[^note]: body\n\n[link]: /url", true),
            (true, true)
        );
        assert_eq!(definition_candidates("[^note]: /url", false), (true, true));
    }

    #[test]
    fn definition_candidate_rejects_inline_prose_decoys() {
        let source = "Earlier [link](https://example.com).\n- Skip the scan when no `]:` exists.";
        assert!(!has_definition_candidate(source, false));
        assert_eq!(
            definition_candidates("    [indented]: /code", false),
            (true, true)
        );
    }

    #[test]
    fn definition_candidate_accepts_valid_block_prefixes() {
        for source in [
            "[plain]: /url",
            "   [indented]: /url",
            "> [quoted]: /url",
            "> > [nested]: /url",
            "[multi\nline]: /url",
            "[^footnote]: body",
        ] {
            assert!(has_definition_candidate(source, true), "missed {source:?}");
        }
    }

    #[test]
    fn definition_candidate_keeps_the_label_length_boundary() {
        fn definition_with_label_len(prefix: &str, len: usize) -> compact_str::CompactString {
            let mut source = compact_str::CompactString::with_capacity(prefix.len() + len + 7);
            source.push_str(prefix);
            source.extend(std::iter::repeat_n('a', len));
            source.push_str("]: /url");
            source
        }

        let max_label = definition_with_label_len("[", 1000);
        let too_long = definition_with_label_len("[", 1001);

        assert!(has_definition_candidate(&max_label, false));
        assert!(!has_definition_candidate(&too_long, false));

        let long_footnote = definition_with_label_len("[^", 1001);
        assert!(has_definition_candidate(&long_footnote, true));
        assert!(!has_definition_candidate(&long_footnote, false));
    }

    #[test]
    fn candidate_scan_matches_the_opener_walk_on_generated_mixes() {
        // Bracket, colon, caret, line ending, container prefix, and tag bytes
        // in every proportion: the shapes that decide a candidate, dense
        // enough that windows of neighbouring `]:` overlap.
        let tokens = [
            "[", "]", ":", "^", "]:", "[^", "\n", "\r", "\r\n", " ", "\t", ">", "-", "+", "*", ".",
            ")", "0", "9", "a", "word ", "<div>", "é", "🙂",
        ];
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            (state >> 33) as usize
        };
        for _ in 0..4000 {
            let len = next() % 60;
            let mut source = String::new();
            for _ in 0..len {
                source.push_str(tokens[next() % tokens.len()]);
            }
            check_against_oracle(&source);
        }
    }

    #[test]
    fn candidate_scan_matches_the_opener_walk_at_the_label_length_boundary() {
        for opener in ["[", "[^", "> [", "> [^", "   [", "<p>[", "x [", "-[^"] {
            for len in [0, 1, 998, 999, 1000, 1001, 1002, 1003, 1004, 2005] {
                let label = "a".repeat(len);
                check_against_oracle(&format!("{opener}{label}]: /url"));
                // A label that spans lines is line-bounded for footnotes and
                // only length-bounded for reference labels.
                check_against_oracle(&format!("{opener}{label}\nmore]: /url"));
                check_against_oracle(&format!("{opener}{label}\r\nmore]: /url"));
                // A second `]:` further along must not rescue an opener that
                // the first one already left out of range.
                check_against_oracle(&format!("{opener}{label}]x: /url\n]: tail"));
            }
        }
    }

    #[test]
    fn candidate_scan_matches_the_opener_walk_on_adversarial_shapes() {
        let mut sources = vec![
            String::new(),
            "]: no opener at all".to_owned(),
            "]:[".to_owned(),
            "[".to_owned(),
            "[]:".to_owned(),
            "[^]:".to_owned(),
            "]:]:]:]:[a]: x".to_owned(),
            "> [a]: x\n[^b]: y".to_owned(),
            "[^a]: y\n[b]: x".to_owned(),
            "x[^a]: y".to_owned(),
            "<span>[a]: x".to_owned(),
            "<span>[^a]: x".to_owned(),
            "\r[a]: x".to_owned(),
            "\r\n[^a]: x".to_owned(),
        ];
        // Thousands of `]:` around sparse openers: the window before each one
        // is bounded, and openers behind it are never revisited.
        sources.push("]:".repeat(5000));
        sources.push(format!("[{}", "]:".repeat(5000)));
        sources.push(format!("x [{}", "]:".repeat(5000)));
        sources.push(format!("{}[a]: x", "]:".repeat(5000)));
        sources.push("[^".repeat(600) + &"]:".repeat(600));
        sources.push(format!("{}\n{}", "[^".repeat(600), "]:".repeat(600)));
        // Many openers per line, and many `]:` per line.
        sources.push("[a][b][c]: x".to_owned());
        sources.push("[a] [b] [^c]: x".to_owned());
        sources.push("> [a] [^b]: x ]: y ]: z".to_owned());
        sources.push(format!("{}[^tail]: x", " ".repeat(2000)));
        sources.push(format!("> {}[tail]: x", "-".repeat(2000)));
        for source in &sources {
            check_against_oracle(source);
        }
    }
}
