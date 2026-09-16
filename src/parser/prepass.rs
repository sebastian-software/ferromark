//! Document-wide definition discovery.
//!
//! A cheap syntax-shape filter keeps ordinary documents out of collection.
//! Link definitions and footnote labels are collected by the real block
//! grammar, so root and container definitions share context and scope.
//!
//! The filter reports *where* the possible definition openers are, not just
//! that one exists. [`segments`] turns those offsets into the byte ranges the
//! structural pass has to parse — each one bounded by a line start where the
//! real parser is provably at the document root — so an ordinary document no
//! longer block-parses itself twice because of a single `]:`. The planner
//! falls back to the whole body whenever it cannot prove a bound, and the
//! block grammar remains the only authority on what a definition is. See
//! `docs/decisions/2026-09-16-segmented-definition-pass.md`.

use std::rc::Rc;
use std::sync::LazyLock;

use memchr::{memchr, memmem, memrchr, memrchr2};

use super::Parser;
use super::footnote::FootnoteLabels;
use super::line_scan::is_line_ending_byte;
use super::reference::ReferenceMap;

mod segments;

/// The segmented pass is proven against the unsegmented one, over the bundled
/// specification fixtures, the frozen measurement corpora and generated token
/// soup. `gzip` only exists so the corpora can be read without a compression
/// dependency; both modules are test-only.
#[cfg(test)]
mod equivalence;
#[cfg(test)]
mod gzip;

pub(super) use segments::{
    CandidateOpeners, DENSITY_SAMPLE, DefinitionPlan, MIN_PLANNED_BYTES, SEGMENT_COST_BYTES,
    plan_definition_pass,
};

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

/// What the shape filter learned about a source that holds both a `[` and a
/// `]:`.
pub(super) struct CandidateScan {
    /// Some opener could begin a definition.
    pub any: bool,
    /// Some opener could begin a *link reference* definition, rather than only
    /// an enabled footnote label.
    pub link: bool,
    /// The openers sit too close together for planning to pay for itself, so
    /// the walk stopped early and `openers` is deliberately incomplete. Only a
    /// full-body pass may follow.
    pub dense: bool,
}

/// [`definition_candidates`] with the two probes [`Parser::build_prepass`]
/// runs for it, for tests that start from a bare source.
#[cfg(test)]
pub(super) fn scan_definition_candidates(
    source: &str,
    footnotes: bool,
    mdx: bool,
    openers: &mut CandidateOpeners,
) -> CandidateScan {
    let bytes = source.as_bytes();
    let absent = CandidateScan {
        any: false,
        link: false,
        dense: false,
    };
    if memchr(b'[', bytes).is_none() {
        return absent;
    }
    let Some(first_closer) = DEFINITION_CLOSER.find(bytes) else {
        return absent;
    };
    definition_candidates(source, footnotes, mdx, first_closer, openers)
}

/// Where every possible definition opener sits.
///
/// A bare `]:` anywhere is not enough: ordinary prose can mention the token
/// and force the much more expensive structural pre-pass. The opening `[` of
/// either a reference or footnote definition must begin a block line after
/// optional container prefixes. Reference labels are capped at 1,000 bytes;
/// footnote labels are line-bounded but have no length cap. This scanner only
/// proves that necessary shape exists; the full pre-pass remains responsible
/// for validating syntax and block context.
///
/// The caller has already established that the source holds a `[` and has
/// located the first `]:` at `first_closer`, so neither probe is repeated
/// here: those two searches are the whole cost for a document without
/// definitions, and they belong on the caller's lean frame.
///
/// Every positively judged opener is appended to `openers` in ascending order,
/// which is what [`segments`] needs to bound the structural pass. Overreport-
/// ing is safe there — an extra opener only widens a segment — but missing one
/// is not, so the walk judges link and footnote openers alike and no longer
/// stops at the first link candidate.
///
/// The scan is driven from the `]:` side. A document that holds one anywhere
/// used to make every `[` in it pay for a backwards line search and a forward
/// label search, which is the expensive direction on link-dense prose: `[` is
/// the common byte and `]:` the rare one. Walking the `]:` occurrences instead
/// bounds the work per occurrence to the label window before it, and the
/// window start only moves forwards, so an opener is judged once no matter how
/// many `]:` follow it.
fn definition_candidates(
    source: &str,
    footnotes: bool,
    mdx: bool,
    first_closer: usize,
    openers: &mut CandidateOpeners,
) -> CandidateScan {
    let bytes = source.as_bytes();
    let mut found = false;
    let mut link = false;
    // Openers before this offset have already been judged. Their verdict
    // cannot improve for a later `]:`: the length window only moves forwards,
    // and a footnote label that already failed to share a line with one `]:`
    // shares even less with the next.
    let mut judged = 0;
    let mut closer = first_closer;
    loop {
        // `]:` cannot overlap itself, so the next search starts past it.
        let from = closer + 2;
        let label_start = closer.saturating_sub(MAX_LABEL_SPAN);
        // Footnote labels cannot span lines, but unlike reference labels their
        // parser deliberately has no length cap, so their window is the line
        // rather than the label span. Only openers at or past `judged` are
        // still undecided, so the backwards search stops there: the two
        // searches then partition the source and cost one pass in total.
        let line_start = footnotes.then(|| bounded_line_start(bytes, judged, closer));
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
                    openers.push(open as u32);
                }
            } else if open >= label_start {
                link = true;
                openers.push(open as u32);
            }
        }
        judged = closer;
        // Stop as soon as the openers are provably too dense to plan: the
        // span they cover cannot pay for one segment each. Everything after
        // this point would only be collected to be thrown away.
        if link
            && openers.len() >= DENSITY_SAMPLE
            && closer - (openers[0] as usize) < openers.len() * SEGMENT_COST_BYTES
        {
            return CandidateScan {
                any: true,
                link: true,
                dense: true,
            };
        }
        let Some(offset) = DEFINITION_CLOSER.find(&bytes[from..]) else {
            break;
        };
        closer = from + offset;
    }
    // A link candidate settles both flags: every link definition goes through
    // the block grammar, whatever the footnote scan found.
    CandidateScan {
        any: link || found,
        link,
        dense: false,
    }
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

/// [`previous_line_start`] clamped to `from`.
///
/// Callers that only compare the result against offsets at or past `from` see
/// the same verdicts: when no terminator lies in `from..before`, the real line
/// start is at or before `from`, and every such comparison holds either way.
fn bounded_line_start(bytes: &[u8], from: usize, before: usize) -> usize {
    memrchr2(b'\n', b'\r', &bytes[from..before]).map_or(from, |off| from + off + 1)
}

impl<'a> Parser<'a> {
    /// Collects document-wide facts before resolving inline references.
    /// Absent maps stay `None`, avoiding shared allocations on ordinary input.
    ///
    /// Two byte searches settle the overwhelming majority of documents: one for
    /// `[` and one for `]:`. They are the whole cost of the pre-pass there, so
    /// they stay on this frame, and everything that needs the opener buffer
    /// lives behind a call that is never inlined — otherwise an ordinary parse
    /// pays for a buffer it never fills.
    pub(super) fn build_prepass(
        &self,
    ) -> (Option<Rc<ReferenceMap<'a>>>, Option<Rc<FootnoteLabels>>) {
        if !self.options.allow_link_refs && !self.options.footnotes {
            return (None, None);
        }
        let bytes = self.source.as_bytes();
        if memchr(b'[', bytes).is_none() {
            return (None, None);
        }
        let Some(first_closer) = DEFINITION_CLOSER.find(bytes) else {
            return (None, None);
        };
        self.discover_definitions(first_closer)
    }

    /// The rest of the pre-pass, for a source that holds a `[` and a `]:`.
    #[inline(never)]
    fn discover_definitions(
        &self,
        first_closer: usize,
    ) -> (Option<Rc<ReferenceMap<'a>>>, Option<Rc<FootnoteLabels>>) {
        // Inline capacity covers every document the openers are collected for.
        let mut openers = CandidateOpeners::new();
        let scan = definition_candidates(
            self.source,
            self.options.footnotes,
            self.options.mdx,
            first_closer,
            &mut openers,
        );
        if !scan.any
            || !((self.options.allow_link_refs && scan.link)
                || (self.options.footnotes && self.source.contains("[^")))
        {
            return (None, None);
        }
        // Two heuristics decide against planning before the planner runs.
        // Both only ever choose the pass this code replaced, so neither can
        // change what is collected.
        let plan = if scan.dense || self.source.len() < MIN_PLANNED_BYTES {
            DefinitionPlan::Fallback
        } else {
            plan_definition_pass(self.source, &self.options, &openers)
        };
        let (definitions, labels) = match plan {
            DefinitionPlan::Fallback => self.collect_definitions(&[(0, self.source.len())]),
            DefinitionPlan::Segments(segments) => {
                // Every candidate sat inside code or raw HTML: no arena
                // and no parser are built for this document.
                if segments.is_empty() {
                    return (None, None);
                }
                self.collect_definitions(&segments)
            }
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
                let mut openers = super::CandidateOpeners::new();
                let scan = super::scan_definition_candidates(source, footnotes, mdx, &mut openers);
                assert_eq!(
                    (scan.any, scan.link),
                    oracle(source, footnotes, mdx),
                    "footnotes {footnotes}, mdx {mdx}, source {source:?}"
                );
                assert!(
                    openers.windows(2).all(|pair| pair[0] < pair[1]),
                    "openers must be strictly ascending: {openers:?} for {source:?}"
                );
                assert!(
                    openers
                        .iter()
                        .all(|&open| source.as_bytes().get(open as usize) == Some(&b'[')),
                    "every reported opener must be a `[`: {openers:?} for {source:?}"
                );
            }
        }
    }

    fn definition_candidates(source: &str, footnotes: bool) -> (bool, bool) {
        let mut openers = super::CandidateOpeners::new();
        let scan = super::scan_definition_candidates(source, footnotes, false, &mut openers);
        (scan.any, scan.link)
    }

    fn has_definition_candidate(source: &str, footnotes: bool) -> bool {
        definition_candidates(source, footnotes).0
    }

    fn is_dense(source: &str) -> bool {
        let mut openers = super::CandidateOpeners::new();
        super::scan_definition_candidates(source, true, false, &mut openers).dense
    }

    /// `count` reference definitions, each padded to `spacing` bytes.
    fn reference_run(count: usize, spacing: usize) -> String {
        let mut source = String::new();
        for index in 0..count {
            let line = format!("[r{index}]: /u{index}\n");
            source.push_str(&line);
            for _ in line.len()..spacing {
                source.push('x');
            }
            source.push('\n');
        }
        source
    }

    #[test]
    fn a_reference_dense_document_stops_the_scan_early() {
        // Definition after definition: the plan could never skip enough to pay
        // for one segment each, so the scan gives up and the full pass runs.
        assert!(is_dense(&reference_run(40, 0)));
        // The same definitions spread thin are worth planning.
        assert!(!is_dense(&reference_run(40, 4 * super::SEGMENT_COST_BYTES)));
    }

    #[test]
    fn a_short_run_of_definitions_is_never_called_dense() {
        // Below the sample size the average proves nothing, however tight.
        assert!(!is_dense(&reference_run(super::DENSITY_SAMPLE - 1, 0)));
    }

    #[test]
    fn a_dense_scan_still_reports_a_link_candidate() {
        let mut openers = super::CandidateOpeners::new();
        let source = reference_run(40, 0);
        let scan = super::scan_definition_candidates(&source, true, false, &mut openers);
        assert!(scan.dense && scan.any && scan.link);
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
