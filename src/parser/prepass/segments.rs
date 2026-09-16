//! Bounding the structural definition pass to the parts of a document that
//! can hold a definition.
//!
//! The real block grammar stays the only authority on what a definition is
//! (see [`super`]), but it does not have to run over the whole document. This
//! planner turns the shape filter's candidate openers into byte ranges that
//! begin and end at *safe root boundaries*: line starts where the real parser
//! provably sits at the document root with every container and leaf block
//! closed. Parsing `&source[start..end]` with a fresh `Definitions`-phase
//! parser then yields exactly the definitions the full parse yields there.
//!
//! # Why a column-0 line start can be trusted
//!
//! The planner never models containers. It only ever calls a line a boundary
//! when the line begins at column 0 and the real parser demonstrably starts a
//! root block there: the first line, a line after a blank one, or a line after
//! an opaque region the planner itself tracked. Nothing carries block state
//! across that shape:
//!
//! * A list item's continuation indent is at least two columns, and after a
//!   blank line a list continues only through a line indented that far or
//!   through a sibling marker. A column-0 marker after a blank line opens a
//!   fresh list in a segment parse, and every item's content is a function of
//!   its own marker line, so no definition moves.
//! * A blank line ends a block quote outright, and lazy continuation — the one
//!   way a column-0 line joins a container — is blocked right after a blank
//!   line and while a fence is open inside the container.
//! * Indented code, footnote-definition bodies, definition-list bodies and
//!   tables all stop at a non-blank line indented fewer than four columns.
//! * Paragraphs, headings, thematic breaks and setext underlines end at the
//!   blank line.
//!
//! The one exception is a definition list: a column-0 `:` line after a blank
//! line continues the list above it, so such a line is never a boundary while
//! the option is on.
//!
//! # Opaque regions
//!
//! What is left are the constructs that really do span blank lines: fenced
//! code and HTML blocks of types 1-5. Those are recognized from column 0 with
//! the parser's own recognizers and closed with the parser's own closing rules
//! ([`fenced_close_bounds`], [`html_block_end`]), so a candidate inside one is
//! dropped and no boundary is claimed inside one.
//!
//! # Lines the planner cannot place
//!
//! Two shapes are genuinely ambiguous. An opener indented one to three columns
//! is either a root block or a list item's body — indented code needs four
//! columns, and every such opener starts a block, so it can never be a lazy
//! continuation. A column-0 type-7 HTML line after a non-blank line is either
//! a block or paragraph text, because type 7 alone never interrupts a
//! paragraph; that one *can* be a container's lazy continuation.
//!
//! Neither is tracked. Instead the planner locates the end the "root region"
//! reading would give, with the search truncated at the next column-0 content
//! line, and then skips the whole region — keeping its candidates, because the
//! two readings may disagree about where it ends. That is sound exactly when
//! nothing inside the region can begin a root block under either reading, and
//! the planner accepts two proofs of that:
//!
//! * The region covers only the opener's own line, so it has no interior.
//! * The region closes before the next column-0 content line, every non-blank
//!   interior line is indented at least as far as the opener, and the opener
//!   interrupts paragraphs. Then the root reading makes the interior opaque,
//!   while the container reading keeps the enclosing list item open through
//!   all of it — an item's continuation indent is at most the opener's, since
//!   the opener is part of its content.
//!
//! Anything else falls back to the full pass. Skipping rather than walking the
//! interior is also what keeps the planner linear: every closer search either
//! ends the walk or covers a range the walk then jumps over.

use smallvec::SmallVec;

use crate::ParserOptions;

use super::super::Parser;
use super::super::fenced_code::fenced_close_bounds;
use super::super::html::{HtmlBlockStart, html_block_bounds, html_block_end};
use super::super::line_scan::{line_end, line_terminator_end, next_line_start};
use super::super::reference::fence_open;

/// Byte offsets of the `[` openers the shape filter judged possible, ascending.
pub(in crate::parser) type CandidateOpeners = SmallVec<[u32; 16]>;

/// `(start, end)` byte ranges of the body, in document order, merged so that
/// no two of them overlap or touch.
pub(in crate::parser) type Segments = SmallVec<[(usize, usize); 4]>;

/// What the structural definition pass should parse.
pub(in crate::parser) enum DefinitionPlan {
    /// The whole body, exactly as the unsegmented pass did.
    Fallback,
    /// Only these ranges. An empty list means no structural pass at all.
    Segments(Segments),
}

/// A UTF-8 byte-order mark.
///
/// Parser construction strips one from the start of its source, so a segment
/// may never begin on one: the sub-parse would drop three bytes that the full
/// parse keeps as ordinary content.
const BOM: &[u8] = b"\xEF\xBB\xBF";

/// Receives the candidate openers that survive the opaque-region filter.
///
/// Production planning does not need them; the differential tests assert that
/// every survivor really is covered by a segment.
trait CandidateSink {
    fn keep(&mut self, offset: u32);
}

struct IgnoreCandidates;

impl CandidateSink for IgnoreCandidates {
    #[inline]
    fn keep(&mut self, _offset: u32) {}
}

#[cfg(test)]
impl CandidateSink for CandidateOpeners {
    fn keep(&mut self, offset: u32) {
        self.push(offset);
    }
}

/// Plans the structural pass for `source` (a parser's normalized body).
pub(in crate::parser) fn plan_definition_pass(
    source: &str,
    options: &ParserOptions,
    openers: &[u32],
) -> DefinitionPlan {
    plan_with_sink(source, options, openers, &mut IgnoreCandidates)
}

/// What an opener would make opaque, before its end has been located.
#[derive(Clone, Copy)]
enum RegionKind {
    Fence(u8, usize),
    Html(HtmlBlockStart),
}

/// Verdict for a line that might open an opaque region.
enum RegionVerdict {
    /// The line opens nothing the planner has to track.
    None,
    /// The line opens a tracked region ending at this offset.
    Region(usize),
    /// The line may open this region, and may be ordinary container or
    /// paragraph content. Safe to ignore while no column-0 content line lies
    /// inside the range the region would cover. The flag marks an opener that
    /// keeps an enclosing list item open rather than lazily continuing it.
    Ambiguous(RegionKind, bool),
    /// The line's meaning cannot be bounded at all.
    Unknown,
}

/// The first column-0 content line at or after a queried offset.
///
/// The planner walks forwards, so one remembered answer serves every query
/// inside the range it already covered and the scans partition the document.
struct RootLines {
    scanned_from: usize,
    found: usize,
}

impl RootLines {
    fn next(&mut self, bytes: &[u8], from: usize) -> usize {
        if self.scanned_from <= from && from <= self.found {
            return self.found;
        }
        let mut at = from;
        while at < bytes.len() {
            // A line at column 0 that is neither blank nor indented.
            if !matches!(bytes[at], b' ' | b'\t' | b'\n' | b'\r') {
                break;
            }
            at = next_line_start(bytes, at);
        }
        self.scanned_from = from;
        self.found = at;
        at
    }
}

fn plan_with_sink<S: CandidateSink>(
    source: &str,
    options: &ParserOptions,
    openers: &[u32],
    kept: &mut S,
) -> DefinitionPlan {
    // MDX gives `<`, `{` and identifier lines block meanings the rules below
    // do not model, and line comments can hide any line from the grammar.
    if options.mdx || options.line_comments {
        return DefinitionPlan::Fallback;
    }
    let bytes = source.as_bytes();
    if bytes.starts_with(BOM) {
        return DefinitionPlan::Fallback;
    }

    let mut segments = Segments::new();
    // Start of a segment whose candidate has been seen but whose end is not
    // known yet.
    let mut open_segment: Option<usize> = None;
    // The greatest boundary at or before the current line. Offset zero is
    // always one: it is where the real parser begins.
    let mut last_boundary = 0usize;
    let mut candidate = 0usize;
    let mut line_start = 0usize;
    // Whether the real parser starts a fresh root block on this line. True on
    // the first line, after a blank line, and after a tracked opaque region.
    let mut at_root = true;
    let mut root_lines = RootLines {
        scanned_from: 0,
        found: 0,
    };

    while line_start < bytes.len() {
        let end = line_end(bytes, line_start);
        let next = line_terminator_end(bytes, end);

        let mut content = line_start;
        while content < end && matches!(bytes[content], b' ' | b'\t') {
            content += 1;
        }
        if content == end {
            // A blank line closes every leaf block and every block quote, and
            // it cannot hold a candidate.
            at_root = true;
            line_start = next;
            continue;
        }

        let indent = content - line_start;
        let trimmed = &source[content..end];

        if indent == 0 {
            // A region's own opening line is still a root block start, so the
            // boundary is recorded before the region opens.
            if line_start > 0 && at_root && is_boundary(bytes, line_start, trimmed, options) {
                if let Some(start) = open_segment.take() {
                    push_segment(&mut segments, start, line_start);
                }
                last_boundary = line_start;
            }

            match root_region(bytes, line_start, next, trimmed, options, at_root) {
                RegionVerdict::Unknown => return DefinitionPlan::Fallback,
                RegionVerdict::Region(region_end) => {
                    // Candidates inside code or raw HTML cannot be definitions.
                    while candidate < openers.len() && (openers[candidate] as usize) < region_end {
                        candidate += 1;
                    }
                    // A region's last line is a closing fence or a terminator
                    // line, so the line at `region_end` starts a fresh root
                    // block; a type-6 region instead ends on the blank line the
                    // loop reads next, which sets the same flag.
                    at_root = true;
                    line_start = region_end;
                    continue;
                }
                RegionVerdict::Ambiguous(kind, _) => {
                    let limit = root_lines.next(bytes, next);
                    let Some(region_end) = bounded_region_end(bytes, limit, line_start, next, kind)
                    else {
                        return DefinitionPlan::Fallback;
                    };
                    if region_end > next {
                        // The block reading would swallow the lines that
                        // follow, the paragraph reading would not.
                        return DefinitionPlan::Fallback;
                    }
                    // The region is the opener's own line, so the two readings
                    // differ only in what that one line is called. Its own
                    // candidates, if any, stay live below.
                }
                RegionVerdict::None => {}
            }
        } else if matches!(trimmed.as_bytes()[0], b'`' | b'~' | b'<' | b'$')
            // Four columns or more is indented code at the root, and a tab
            // always reaches column four.
            && indent <= 3
            && !bytes[line_start..content].contains(&b'\t')
        {
            // A line indented one to three columns may be a root block or the
            // body of a list item that ends at the next column-0 line. Both
            // readings agree about every boundary as long as the walk can skip
            // whatever the block reading would make opaque.
            match shallow_region(trimmed, options) {
                RegionVerdict::Unknown => return DefinitionPlan::Fallback,
                RegionVerdict::Ambiguous(kind, keeps_container_open) => {
                    let limit = root_lines.next(bytes, next);
                    let Some(region_end) = bounded_region_end(bytes, limit, line_start, next, kind)
                    else {
                        return DefinitionPlan::Fallback;
                    };
                    if keeps_container_open && interior_is_indented(bytes, next, region_end, indent)
                    {
                        // Opaque under one reading, container content under the
                        // other: nothing inside can start a root block. Its
                        // candidates still have to be collected, because the
                        // two readings may disagree about where it ends.
                        if candidate < openers.len() && (openers[candidate] as usize) < region_end {
                            open_segment.get_or_insert(last_boundary);
                            while candidate < openers.len()
                                && (openers[candidate] as usize) < region_end
                            {
                                kept.keep(openers[candidate]);
                                candidate += 1;
                            }
                        }
                        // The container reading leaves a block open here.
                        at_root = false;
                        line_start = region_end;
                        continue;
                    }
                    if region_end > next {
                        // Walking the interior instead would have to judge its
                        // lines without knowing which reading holds, and the
                        // scan that found this end would then be repeated for
                        // each of them. Hand the document to the full pass.
                        return DefinitionPlan::Fallback;
                    }
                    // The region is the opener's own line; nothing follows it
                    // that the two readings could disagree about.
                }
                RegionVerdict::Region(_) | RegionVerdict::None => {}
            }
        }

        if candidate < openers.len() && (openers[candidate] as usize) < next {
            open_segment.get_or_insert(last_boundary);
            while candidate < openers.len() && (openers[candidate] as usize) < next {
                kept.keep(openers[candidate]);
                candidate += 1;
            }
        }

        at_root = false;
        line_start = next;
    }

    if let Some(start) = open_segment {
        push_segment(&mut segments, start, bytes.len());
    }
    DefinitionPlan::Segments(segments)
}

/// End of an ambiguous opener's region, or `None` when ignoring the opener
/// would not be safe.
///
/// `limit` is the first column-0 content line at or after the opener's next
/// line. The closer is searched in the document truncated there, which bounds
/// the search *and* settles the question: a closer found inside the truncated
/// view is the one the untruncated search would find, while a region that
/// runs out of input could swallow the column-0 line and beyond. When `limit`
/// is the end of the document there is no such line left to swallow, so any
/// end is safe.
fn bounded_region_end(
    bytes: &[u8],
    limit: usize,
    line_start: usize,
    next: usize,
    kind: RegionKind,
) -> Option<usize> {
    let view = &bytes[..limit];
    let (region_end, closed) = match kind {
        RegionKind::Fence(fence_byte, fence_len) => {
            let (body_end, after_fence) = fenced_close_bounds(view, fence_byte, fence_len, next);
            // An unclosed fence reports the body running to the end of the
            // view; a real closing fence reports its own line start.
            (after_fence, body_end < limit)
        }
        RegionKind::Html(block_start) => html_block_bounds(view, line_start, block_start),
    };
    (closed || limit == bytes.len()).then_some(region_end)
}

/// Whether a column-0 content line that the real parser reaches at the root
/// may also begin a segment.
fn is_boundary(bytes: &[u8], line_start: usize, trimmed: &str, options: &ParserOptions) -> bool {
    // A `:` body line after a blank line continues the definition list above
    // it, so the parser is inside a `DefinitionList` there, not at the root.
    if options.definition_lists && trimmed.as_bytes()[0] == b':' {
        return false;
    }
    // Parser construction would strip a mark a sub-slice began with.
    !bytes[line_start..].starts_with(BOM)
}

/// Classifies a column-0 content line as an opaque-region opener.
fn root_region(
    bytes: &[u8],
    line_start: usize,
    next: usize,
    trimmed: &str,
    options: &ParserOptions,
    at_root: bool,
) -> RegionVerdict {
    match trimmed.as_bytes()[0] {
        b'`' | b'~' => match fence_open(trimmed) {
            // The info string of the opening line cannot hold a definition, so
            // the region covers the opener too.
            Some((fence_byte, fence_len)) => {
                RegionVerdict::Region(fenced_close_bounds(bytes, fence_byte, fence_len, next).1)
            }
            None => RegionVerdict::None,
        },
        b'<' => {
            if let Some(block_start) = Parser::parse_html_block_start(trimmed) {
                // Types 1-6 interrupt an open paragraph, so they start a block
                // wherever they appear at column 0.
                RegionVerdict::Region(html_block_end(bytes, line_start, block_start))
            } else if !Parser::is_html_block_type7_line(trimmed) {
                RegionVerdict::None
            } else if at_root {
                RegionVerdict::Region(html_block_end(bytes, line_start, HtmlBlockStart::Other))
            } else {
                // A type-7 line never interrupts a paragraph, and after a
                // non-blank line the planner cannot tell whether one is open —
                // the line could even be a container's lazy continuation.
                RegionVerdict::Ambiguous(RegionKind::Html(HtmlBlockStart::Other), false)
            }
        }
        // A display-math block spans blank lines like a fence, and a single
        // `$` line can open one too. Math is off by default, so the whole
        // document falls back rather than growing a second closer rule.
        b'$' if options.math => RegionVerdict::Unknown,
        _ => RegionVerdict::None,
    }
}

/// What an opener indented one to three columns could make opaque.
///
/// Never [`RegionVerdict::Region`]: an indented opener is never tracked, only
/// bounded. The flag says whether the opener would keep an enclosing list item
/// open under the container reading — every kind here interrupts a paragraph
/// and so cannot be a lazy continuation, except a type-7 line, which can.
fn shallow_region(trimmed: &str, options: &ParserOptions) -> RegionVerdict {
    let opener = match trimmed.as_bytes()[0] {
        b'`' | b'~' => fence_open(trimmed).map(|(byte, len)| (RegionKind::Fence(byte, len), true)),
        b'<' => match Parser::parse_html_block_start(trimmed) {
            Some(block_start) => Some((RegionKind::Html(block_start), true)),
            None => Parser::is_html_block_type7_line(trimmed)
                .then_some((RegionKind::Html(HtmlBlockStart::Other), false)),
        },
        // A display-math block has its own closer rule; math is off by default.
        b'$' if options.math => return RegionVerdict::Unknown,
        _ => None,
    };
    opener.map_or(RegionVerdict::None, |(kind, keeps_container_open)| {
        RegionVerdict::Ambiguous(kind, keeps_container_open)
    })
}

/// Whether every non-blank line in `from..to` is indented at least `columns`.
///
/// A tab always reaches column four, and `columns` is at most three, so a line
/// whose indentation holds one satisfies any bound this function is asked for.
fn interior_is_indented(bytes: &[u8], from: usize, to: usize, columns: usize) -> bool {
    debug_assert!(columns <= 3, "only shallow openers ask for this");
    let mut at = from;
    while at < to {
        let end = line_end(bytes, at);
        let mut content = at;
        while content < end && matches!(bytes[content], b' ' | b'\t') {
            content += 1;
        }
        if content < end && content - at < columns && !bytes[at..content].contains(&b'\t') {
            return false;
        }
        at = line_terminator_end(bytes, end);
    }
    true
}

/// Appends `(start, end)`, merging it into the previous range when they touch.
fn push_segment(segments: &mut Segments, start: usize, end: usize) {
    match segments.last_mut() {
        Some(last) if last.1 >= start => last.1 = last.1.max(end),
        _ => segments.push((start, end)),
    }
}

#[cfg(test)]
pub(in crate::parser) fn plan_with_survivors(
    source: &str,
    options: &ParserOptions,
    openers: &[u32],
) -> (DefinitionPlan, CandidateOpeners) {
    let mut kept = CandidateOpeners::new();
    let plan = plan_with_sink(source, options, openers, &mut kept);
    (plan, kept)
}

#[cfg(test)]
mod tests {
    // Owned strings keep the expectations independent of arena storage.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use crate::ParserOptions;

    use super::super::{CandidateOpeners, definition_candidates};
    use super::{DefinitionPlan, plan_definition_pass};

    /// The planned ranges, or `None` when the document falls back.
    fn plan(source: &str, options: &ParserOptions) -> Option<Vec<(usize, usize)>> {
        let mut openers = CandidateOpeners::new();
        definition_candidates(source, options.footnotes, options.mdx, &mut openers);
        match plan_definition_pass(source, options, &openers) {
            DefinitionPlan::Fallback => None,
            DefinitionPlan::Segments(segments) => Some(segments.into_vec()),
        }
    }

    fn gfm() -> ParserOptions {
        ParserOptions::gfm()
    }

    #[test]
    fn a_candidate_inside_a_fence_leaves_nothing_to_parse() {
        // The `comment-incident` shape from the issue: the only `]:` line sits
        // inside a fenced example, so no arena and no parser are built at all.
        let source = "Thanks!\n\nTry this:\n\n```md\n[help]: /getting-started\n```\n\nDone.\n";
        assert_eq!(plan(source, &gfm()), Some(vec![]));
    }

    #[test]
    fn a_root_definition_is_bounded_by_its_own_paragraph() {
        let source = "# Title\n\nIntro text.\n\n[a]: /one\n[b]: /two\n\nMore prose.\n";
        let start = source.find("[a]:").expect("definition line");
        let end = source.find("More prose").expect("following paragraph");
        assert_eq!(plan(source, &gfm()), Some(vec![(start, end)]));
    }

    #[test]
    fn definitions_far_apart_produce_separate_segments() {
        let source = "[a]: /one\n\nfiller\n\nfiller\n\n[b]: /two\n";
        let plan = plan(source, &gfm()).expect("segmented");
        assert_eq!(plan.len(), 2, "{plan:?}");
        assert_eq!(plan[0].0, 0);
        assert_eq!(plan[1].0, source.find("[b]:").expect("second definition"));
    }

    #[test]
    fn adjacent_segments_merge_into_one_range() {
        let source = "[a]: /one\n\n[b]: /two\n";
        assert_eq!(plan(source, &gfm()), Some(vec![(0, source.len())]));
    }

    #[test]
    fn a_container_candidate_starts_its_segment_at_the_container() {
        let source = "prose\n\n- item\n\n  [a]: /one\n\nafter\n";
        let start = source.find("- item").expect("list marker");
        let end = source.find("after").expect("trailing paragraph");
        assert_eq!(plan(source, &gfm()), Some(vec![(start, end)]));
    }

    #[test]
    fn an_indented_fence_inside_a_list_item_does_not_force_a_fallback() {
        let source = "- first\n  ```ts\n  const a = 1\n  ```\n- second\n\n[a]: /one\n";
        let plan = plan(source, &gfm()).expect("segmented");
        assert_eq!(
            plan,
            vec![(source.find("[a]:").expect("definition"), source.len())]
        );
    }

    #[test]
    fn a_column_zero_line_inside_an_indented_fence_falls_back() {
        // The fence at column one could be a root block that swallows the
        // definition, or list content that cannot; the planner cannot tell.
        let source = "- item\n\n ```\n[a]: /one\n ```\n\nafter\n";
        assert_eq!(plan(source, &gfm()), None);
    }

    #[test]
    fn mdx_and_line_comments_fall_back() {
        for options in [
            ParserOptions::mdx(),
            ParserOptions {
                line_comments: true,
                ..ParserOptions::gfm()
            },
        ] {
            assert_eq!(plan("[a]: /one\n", &options), None);
        }
    }

    #[test]
    fn a_math_line_falls_back_only_while_the_option_is_on() {
        let source = "$$\nx\n$$\n\n[a]: /one\n";
        assert_eq!(
            plan(source, &gfm()),
            Some(vec![(source.len() - 10, source.len())])
        );
        assert_eq!(
            plan(
                source,
                &ParserOptions {
                    math: true,
                    ..ParserOptions::gfm()
                }
            ),
            None
        );
    }

    #[test]
    fn a_definition_list_body_line_is_not_a_boundary() {
        let source = "term\n\n: body\n\n  [a]: /one\n";
        let options = ParserOptions {
            definition_lists: true,
            ..ParserOptions::gfm()
        };
        // The `:` line continues the list, so the segment reaches back to the
        // term rather than starting inside the definition list.
        assert_eq!(plan(source, &options), Some(vec![(0, source.len())]));
        // With the option off the `:` line is an ordinary paragraph start.
        let plan = plan(source, &gfm()).expect("segmented");
        assert_eq!(
            plan,
            vec![(source.find(": body").expect("body"), source.len())]
        );
    }

    #[test]
    fn a_leading_byte_order_mark_falls_back() {
        assert_eq!(plan("\u{feff}[a]: /one\n", &gfm()), None);
    }

    #[test]
    fn a_type_seven_line_covering_only_itself_is_not_a_fallback() {
        // `<custom>` after a paragraph line is either a one-line HTML block or
        // paragraph text; with a blank line under it the two readings differ
        // only in what that line is called.
        let source = "prose\n<custom>\n\n[a]: /one\n";
        let start = source.find("[a]:").expect("definition");
        assert_eq!(plan(source, &gfm()), Some(vec![(start, source.len())]));
    }

    #[test]
    fn a_type_seven_line_with_content_under_it_falls_back() {
        // Under the block reading the fence run is HTML content, under the
        // paragraph reading it opens a code block that hides the definition.
        assert_eq!(plan("prose\n<custom>\n```\n\n[a]: /one\n", &gfm()), None);
    }

    #[test]
    fn a_column_zero_type_seven_line_after_a_blank_opens_a_tracked_region() {
        let source = "prose\n\n<custom>\n[a]: /hidden\n\n[a]: /one\n";
        let start = source.find("[a]: /one").expect("root definition");
        assert_eq!(plan(source, &gfm()), Some(vec![(start, source.len())]));
    }
}
