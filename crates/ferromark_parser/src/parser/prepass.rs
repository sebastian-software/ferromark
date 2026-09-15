//! Document-wide definition discovery.
//!
//! A cheap syntax-shape filter keeps ordinary documents out of collection.
//! Link definitions and footnote labels are collected by the real block
//! grammar, so root and container definitions share context and scope.

use std::rc::Rc;
use std::sync::LazyLock;

use memchr::{memchr, memmem, memrchr2};

use super::Parser;
use super::footnote::FootnoteLabels;
use super::line_scan::line_end as scan_line_end;
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
fn definition_candidates(source: &str, footnotes: bool, mdx: bool) -> (bool, bool) {
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
        // A necessary shape, not a container grammar. False positives (for
        // example indented code) are rejected by the real block parser.
        let block_prefix = raw_prefix.bytes().all(|byte| {
            matches!(
                byte,
                b' ' | b'\t' | b'>' | b'-' | b'+' | b'*' | b'.' | b')' | b'0'..=b'9'
            )
        });
        // A flow component can begin Markdown children on its opening line.
        // Treat a preceding tag end as a candidate; the block grammar decides
        // whether it really belongs to JSX rather than raw HTML or prose.
        if block_prefix || (mdx && raw_prefix.contains('>')) {
            let candidate_end = if footnotes && bytes.get(open + 1) == Some(&b'^') {
                // Footnote labels cannot span lines, but unlike reference
                // labels their parser deliberately has no length cap.
                scan_line_end(bytes, open + 2)
            } else {
                // label_start..=closing bracket spans at most 1,001 bytes,
                // with one final byte needed for the colon after it.
                open.saturating_add(1003).min(bytes.len())
            };
            if memmem::find(&bytes[open + 1..candidate_end], b"]:").is_some() {
                if footnotes && bytes.get(open + 1) == Some(&b'^') {
                    found = true;
                } else {
                    // All link definitions use the block grammar, so there
                    // is no need to search for a later container candidate.
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
}
