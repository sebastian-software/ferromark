//! Document-wide definition discovery.
//!
//! A cheap syntax-shape filter keeps ordinary documents out of collection.
//! Link definitions are collected by the real block grammar, so root and
//! container definitions share context and precedence rules. Footnote labels
//! retain their existing raw-line policy independently of link definitions.

use std::rc::Rc;
use std::sync::LazyLock;

use memchr::{memchr, memmem, memrchr2};

use super::Parser;
use super::footnote::{FootnoteLabels, normalize_footnote_label, parse_footnote_opener};
use super::line_scan::{line_end as scan_line_end, line_terminator_end};
use super::reference::{ReferenceMap, fence_open, is_fence_close};

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
fn definition_candidates(source: &str, footnotes: bool) -> (bool, bool) {
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
        if block_prefix {
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
            definition_candidates(self.source, self.options.footnotes);
        if !has_candidate {
            return (None, None);
        }
        let definitions = if self.options.allow_link_refs && has_link_candidate {
            self.collect_references()
        } else {
            ReferenceMap::default()
        };
        let mut labels = FootnoteLabels::default();
        if self.options.footnotes && self.source.contains("[^") {
            // Preserve the existing footnote scope. Every physical line was
            // already inspected independently of the reference scan's fence
            // and paragraph state; no link-grammar approximation is needed.
            let bytes = self.source.as_bytes();
            let mut pos = 0;
            let mut fence = None;
            while pos < bytes.len() {
                let end = scan_line_end(bytes, pos);
                footnote_scan_line(&self.source[pos..end], bytes[pos], &mut fence, &mut labels);
                pos = line_terminator_end(bytes, end);
            }
        }
        (
            (!definitions.is_empty()).then(|| Rc::new(definitions)),
            (!labels.is_empty()).then(|| Rc::new(labels)),
        )
    }
}

/// One line of the footnote-label scan: raw-line fence tracking plus the
/// `[^label]:` opener check. Mirrors the former standalone footnote
/// pre-pass exactly (no quote stripping).
fn footnote_scan_line(
    line: &str,
    first: u8,
    foot_fence: &mut Option<(u8, usize)>,
    labels: &mut FootnoteLabels,
) {
    let raw_trimmed = line.trim_start_matches([' ', '\t']);
    if let Some((fence_byte, fence_len)) = *foot_fence {
        if is_fence_close(raw_trimmed, fence_byte, fence_len) {
            *foot_fence = None;
        }
    } else if let Some(open) = fence_open(raw_trimmed) {
        *foot_fence = Some(open);
    } else if matches!(first, b'[' | b' ') {
        // An opener starts with `[^` after at most three spaces, so only
        // these first bytes can begin one.
        if let Some((label, _)) = parse_footnote_opener(line) {
            labels.insert(normalize_footnote_label(label));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::definition_candidates;

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
