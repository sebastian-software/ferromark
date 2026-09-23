//! The block-level autolink pre-flight, answered from recorded trigger
//! offsets.
//!
//! [`may_contain_autolink`] walks a block's whole content after the inline
//! parse has read it, and a block costs that walk's setup however short it
//! is. With GFM autolinks on, the root parser instead records where the bytes
//! that can decide the answer are (see [`collect`]), as blocks ask for them,
//! and a block whose content is a slice of its source only visits the offsets
//! inside its range.
//!
//! # Which bytes are recorded
//!
//! Only bytes blocks ask for, each at most once. Blocks are asked for in
//! source order, so the record is one run that grows forward:
//!
//! - A block that starts inside the run, or at most [`GAP`] bytes past it
//!   (the blank line, list marker or pipe between two blocks), extends the
//!   run to its end and on to the next multiple of [`CHUNK`], so the blocks
//!   that follow it are usually covered already and only look up offsets.
//! - A block that starts further on, past a code block or raw HTML no block
//!   asked for, starts a new run with exactly its own bytes, and the old run
//!   is dropped. An isolated block therefore records what the full pass would
//!   have read, and a body that is mostly code or HTML pays for its few
//!   inline blocks, not for every byte.
//! - Content that starts before the run (a block asked for out of order)
//!   takes the full pass.
//!
//! So the bytes recorded never exceed the body, and exceed the bytes the full
//! pass reads by at most the gaps between blocks and one chunk of read-ahead
//! per stretch of consecutive blocks.
//!
//! The record also stops for good once it holds more offsets than a density
//! bound allows for the bytes recorded so far ([`OFFSET_ALLOWANCE`] plus one
//! per [`BYTES_PER_OFFSET`]): a body dense in `@`, `://` or `www.` would pay
//! for recording and visiting them on top of the full pass that every block
//! with an `@` takes anyway. Every later block then takes the full pass.
//!
//! # Why the answers are equal
//!
//! [`may_contain_autolink`] derives its answer from three facts: whether the
//! content holds an `@`, whether it holds a `://` that does not complete a
//! `](scheme` destination, and whether it holds a `www.`. With an `@` it also
//! looks for `mailto:` and `xmpp:` in front of any colon. A recorded run holds
//! every trigger offset in its range, so for content inside it:
//!
//! - Every `@` is recorded. An `@` inside the content hands the content to
//!   [`may_contain_autolink`] itself, which is the only place the
//!   `mailto:`/`xmpp:` rule matters: without an `@` neither extended scheme
//!   can make the answer differ.
//! - Every `:` that a `/` follows is recorded, so every `://` of the content
//!   is visited at its colon. The same test as the full pass decides it, on
//!   the content's own bytes: `//` has to follow inside the content, and the
//!   destination check looks back no further than the content's start.
//! - Every `w` that a `.` follows is recorded, so every `www.` of the content
//!   is visited at its last `w`, and counts when the two bytes in front of it
//!   and the `.` behind it are inside the content.
//!
//! Content qualifies by address alone: it has to lie inside the root source.
//! Both are borrowed from the same immutable source for the whole parse, so
//! equal addresses hold equal bytes. Content a sub-parser copied into the
//! arena (block quotes, most list items, escaped table cells, lines with
//! comments removed) takes the full pass, as does every block on targets
//! without the vector recorder.

use std::cell::RefCell;

use crate::allocator::{Allocator, Vec};

use super::AutolinkScan;
use super::scan::{may_contain_autolink, scheme_is_markdown_destination};

mod collect;

/// A block that extends the record reads ahead to the next multiple of this,
/// so the blocks after it in the same stretch are covered already.
pub(super) const CHUNK: usize = 512;

/// How far past the recorded run a block may start and still extend it
/// rather than start a new run.
pub(super) const GAP: usize = 64;

/// Offsets any record may hold, however few bytes it has read.
///
/// The broad corpus averages about one trigger per kilobyte (400 in the 40
/// `gfm` documents' 394 KB); its densest document holds 7.4 per kilobyte, and
/// its densest real one 4.1.
pub(super) const OFFSET_ALLOWANCE: usize = 32;

/// Bytes read per offset a record may hold beyond [`OFFSET_ALLOWANCE`]: 16
/// per kilobyte, twice the densest corpus document.
pub(super) const BYTES_PER_OFFSET: usize = 64;

/// Trigger offsets of the root source, recorded as blocks ask for them.
pub(in crate::parser) struct AutolinkTriggers<'a> {
    /// The root parser's source, which the offsets index.
    body: &'a [u8],
    record: RefCell<Record<'a>>,
}

/// What has been recorded so far.
struct Record<'a> {
    /// Offset of every trigger byte in `run_start..covered_end`, ascending.
    offsets: Vec<'a, u32>,
    /// Start of the recorded run.
    run_start: usize,
    /// End of the recorded run.
    covered_end: usize,
    /// Where the last lookup ended: the index of the first offset at or past
    /// the end of the content it answered. Blocks are asked for in source
    /// order, so the next one usually starts here.
    cursor: usize,
    /// Bytes recorded over the whole parse, dropped runs included.
    scanned: usize,
    /// Offsets recorded over the whole parse, dropped runs included.
    recorded: usize,
    /// The density bound was passed: nothing is recorded any more.
    exhausted: bool,
}

impl<'a> AutolinkTriggers<'a> {
    /// An empty record over `body`, the root parser's source.
    pub(in crate::parser) fn new(allocator: &'a Allocator, body: &'a [u8]) -> Self {
        Self {
            body,
            record: RefCell::new(Record {
                offsets: allocator.new_vec(),
                run_start: 0,
                covered_end: 0,
                cursor: 0,
                scanned: 0,
                recorded: 0,
                // Offsets are stored as `u32`.
                exhausted: u32::try_from(body.len()).is_err(),
            }),
        }
    }

    /// Where `content` starts in the body, once the record covers it; `None`
    /// when the full pass has to answer: `content` lies outside the body,
    /// starts before the recorded run, or the record is exhausted.
    #[inline]
    pub(in crate::parser::inline) fn cover(&self, content: &str) -> Option<usize> {
        let len = self.body.len();
        let start = (content.as_ptr() as usize).wrapping_sub(self.body.as_ptr() as usize);
        if start > len || content.len() > len - start {
            return None;
        }
        let end = start + content.len();
        let mut record = self.record.borrow_mut();
        if record.exhausted {
            return None;
        }
        if start == end {
            // Nothing to visit.
            return Some(start);
        }
        if start < record.run_start {
            return None;
        }
        if end <= record.covered_end {
            return Some(start);
        }
        let (scan_from, to) = if start <= record.covered_end.saturating_add(GAP) {
            // The next block of a stretch: extend the run and read ahead.
            let ahead = end.div_ceil(CHUNK).saturating_mul(CHUNK).min(len);
            (record.covered_end, ahead)
        } else {
            // Past bytes no block asked for: a new run of exactly this block.
            record.offsets.clear();
            record.cursor = 0;
            record.run_start = start;
            (start, end)
        };
        let before = record.offsets.len();
        collect::record(self.body, scan_from, to, &mut record.offsets);
        record.covered_end = to;
        record.scanned += to - scan_from;
        record.recorded += record.offsets.len() - before;
        if record.recorded > OFFSET_ALLOWANCE + record.scanned / BYTES_PER_OFFSET {
            record.exhausted = true;
            record.offsets.clear();
            return None;
        }
        Some(start)
    }

    /// The pre-flight's answer for `content`, which starts at `start` in the
    /// body and which [`Self::cover`] has just covered.
    #[inline]
    pub(in crate::parser::inline) fn preflight(
        &self,
        content: &str,
        start: usize,
    ) -> Option<AutolinkScan> {
        let mut record = self.record.borrow_mut();
        let end = start + content.len();
        debug_assert!(
            start == end
                || (!record.exhausted && start >= record.run_start && end <= record.covered_end)
        );
        let bytes = content.as_bytes();
        let mut has_www = false;
        let mut has_bare_scheme = false;
        let mut index = record.first_at_or_after(start);
        while let Some(&offset) = record.offsets.get(index) {
            let offset = offset as usize;
            if offset >= end {
                break;
            }
            index += 1;
            let at = offset - start;
            match bytes[at] {
                // Rare in prose. The full pass also settles `mailto:` and
                // `xmpp:`, whose colons are not recorded.
                b'@' => return may_contain_autolink(content),
                b':' => {
                    if bytes[at + 1..].starts_with(b"//") {
                        has_bare_scheme |= !scheme_is_markdown_destination(bytes, at);
                    }
                }
                // A `w` that a `.` or `/` follows in the body.
                byte => {
                    debug_assert_eq!(byte, b'w');
                    has_www |=
                        at >= 2 && bytes.get(at + 1) == Some(&b'.') && bytes[at - 2..at] == *b"ww";
                }
            }
        }
        record.cursor = index;
        // Without an `@` neither extended scheme can apply, and a `www.` is
        // the only reason for the `www.` search.
        (has_www || has_bare_scheme).then_some(AutolinkScan {
            may_have_www: has_www,
            may_have_extended: false,
        })
    }

    /// Whether the density bound stopped the record.
    #[cfg(test)]
    fn is_exhausted(&self) -> bool {
        self.record.borrow().exhausted
    }

    /// Bytes recorded so far.
    #[cfg(test)]
    fn scanned(&self) -> usize {
        self.record.borrow().scanned
    }
}

impl Record<'_> {
    /// Index of the first offset at or past `start`.
    #[inline]
    fn first_at_or_after(&self, start: usize) -> usize {
        let offsets = self.offsets.as_slice();
        let before = |offset: &u32| (*offset as usize) < start;
        let hint = self.cursor;
        if hint > 0 && !before(&offsets[hint - 1]) {
            // Content before the last lookup's: a sub-source, or a block
            // parsed out of order.
            offsets[..hint].partition_point(before)
        } else if offsets.get(hint).is_some_and(before) {
            // Offsets between the two, in code blocks or other non-inline
            // content.
            hint + offsets[hint..].partition_point(before)
        } else {
            hint
        }
    }
}

/// Whether a root parse on this target records trigger offsets.
///
/// Only aarch64 has the vector recorder (see [`collect`]); other targets keep
/// the separate pre-flight pass.
#[cfg(not(test))]
#[inline]
pub(in crate::parser) const fn collects_triggers() -> bool {
    cfg!(target_arch = "aarch64")
}

/// Test builds record them on every target, with the portable byte-wise
/// recorder off aarch64, so every block a unit test parses checks the indexed
/// answer against the full pass (see `Parser::autolink_preflight`). A test
/// can switch the record off for a comparison.
#[cfg(test)]
pub(in crate::parser) fn collects_triggers() -> bool {
    COLLECTS_TRIGGERS.with(std::cell::Cell::get)
}

#[cfg(test)]
thread_local! {
    static COLLECTS_TRIGGERS: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Runs `run` with trigger recording switched on or off on this thread.
#[cfg(test)]
pub(super) fn with_triggers<R>(collect: bool, run: impl FnOnce() -> R) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            COLLECTS_TRIGGERS.with(|cell| cell.set(self.0));
        }
    }
    let _restore = Restore(COLLECTS_TRIGGERS.with(|cell| cell.replace(collect)));
    run()
}

#[cfg(test)]
mod tests {
    // Owned buffers keep the test oracle independent of production storage.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use crate::allocator::Allocator;

    use super::{
        AutolinkScan, AutolinkTriggers, BYTES_PER_OFFSET, CHUNK, GAP, OFFSET_ALLOWANCE,
        may_contain_autolink,
    };

    #[test]
    fn isolated_blocks_record_only_their_own_bytes() {
        // Short blocks between long stretches no block asks for, like
        // headings between raw HTML tables: each starts a new run of exactly
        // its bytes, however long the body.
        let html = "<tr><td><code>--flag</code></td><td>text</td></tr>\n".repeat(40);
        let mut body = String::new();
        let mut blocks = Vec::new();
        for heading in ["Options", "www.example.com", "More http://x.y"] {
            body.push_str(&html);
            let start = body.len();
            body.push_str(heading);
            blocks.push((start, body.len()));
            body.push('\n');
        }
        body.push_str(&html);
        let allocator = Allocator::new();
        let triggers = AutolinkTriggers::new(&allocator, body.as_bytes());
        let mut expected = 0;
        for &(start, end) in &blocks {
            assert!(ask(&triggers, &body, start, end));
            expected += end - start;
            assert_eq!(triggers.scanned(), expected, "{start}..{end}");
        }
        // A block right after one of them extends the run over the gap and
        // reads ahead to the next chunk boundary.
        let (_, last_end) = blocks[blocks.len() - 1];
        let next = last_end + GAP;
        assert!(ask(&triggers, &body, next, next + 10));
        let ahead = (next + 10).div_ceil(CHUNK) * CHUNK;
        assert_eq!(triggers.scanned(), expected + ahead - last_end);
    }

    fn flags(scan: Option<AutolinkScan>) -> Option<(bool, bool)> {
        scan.map(|scan| (scan.may_have_www, scan.may_have_extended))
    }

    /// Asks the record for `body[start..end]`, and checks any answer it gives
    /// against the full pass. Reports whether it answered.
    #[track_caller]
    fn ask(triggers: &AutolinkTriggers<'_>, body: &str, start: usize, end: usize) -> bool {
        let content = &body[start..end];
        let Some(located) = triggers.cover(content) else {
            return false;
        };
        assert_eq!(located, start, "{content:?}");
        assert_eq!(
            flags(triggers.preflight(content, start)),
            flags(may_contain_autolink(content)),
            "{content:?} at {start}..{end} of {body:?}"
        );
        true
    }

    /// Every slice of `body` on character boundaries, asked for in source
    /// order, then in reverse, then nested, on fresh records. Forward order
    /// must be answered throughout while the record stays under its bound.
    #[track_caller]
    fn check_every_slice(body: &str) {
        let allocator = Allocator::new();
        let bounds: Vec<usize> = (0..=body.len())
            .filter(|&at| body.is_char_boundary(at))
            .collect();
        let forward = AutolinkTriggers::new(&allocator, body.as_bytes());
        for &start in &bounds {
            for &end in bounds.iter().filter(|&&end| end >= start) {
                let answered = ask(&forward, body, start, end);
                assert!(
                    answered || forward.is_exhausted(),
                    "{start}..{end} of {body:?}"
                );
            }
        }
        let reverse = AutolinkTriggers::new(&allocator, body.as_bytes());
        for &start in bounds.iter().rev() {
            for &end in bounds.iter().rev().filter(|&&end| end >= start) {
                ask(&reverse, body, start, end);
            }
        }
        let nested = AutolinkTriggers::new(&allocator, body.as_bytes());
        for (index, &start) in bounds.iter().enumerate() {
            let end = bounds[bounds.len() - 1 - index];
            if end >= start {
                ask(&nested, body, start, end);
            }
        }
    }

    #[test]
    fn slices_of_shaped_bodies() {
        for body in [
            "",
            "@",
            "://",
            "www.",
            "wwww.x",
            "see www.example.com today",
            "[a](https://x.y) and https://bare.example",
            "[a](HTTPS://x.y)",
            "a](http://b",
            "](ftp://x",
            "mailto:a@b.c",
            "xmpp:a@b.c/r",
            "mailto: prose a@b.example",
            "a@b w/o www/ :. ://",
            "http://a www. @ mailto:x@y.z",
            "é@🙂www.中://",
            "Note: a::b key: value, now.",
        ] {
            check_every_slice(body);
        }
    }

    #[test]
    fn slices_of_generated_bodies() {
        let tokens = [
            "www.", "ww", "w", ".", "/", "//", ":", "://", "@", "](", "http", "https", "ftp",
            "HTTP", "mailto", "xmpp", "mailto:", "a", " ", "é", "\n", "[", "]", "(",
        ];
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 33) as usize
        };
        for _ in 0..300 {
            let len = next() % 14;
            let mut body = String::new();
            for _ in 0..len {
                body.push_str(tokens[next() % tokens.len()]);
            }
            check_every_slice(&body);
        }
    }

    #[test]
    fn blocks_across_chunks_and_gaps() {
        // Blocks in source order over a body several chunks long, with the
        // needles on and across chunk boundaries, and gaps of whole chunks
        // no block asks for, which start a new run.
        let mut next = {
            let mut state = 0x2545_f491_4f6c_dd1du64;
            move || {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                (state >> 33) as usize
            }
        };
        let needles = [
            "www.a.b ",
            "http://c.d ",
            "[x](https://e.f) ",
            "g@h.i ",
            "w",
            ".",
            ":",
            "//",
            "mailto:",
            "ww",
        ];
        for round in 0..40 {
            let mut body = String::new();
            let mut blocks = Vec::new();
            while body.len() < 12 * CHUNK {
                if next() % 5 == 0 {
                    // A stretch no block asks for, like a fenced code block,
                    // with a needle in it now and then.
                    for _ in 0..next() % 80 {
                        body.push_str(if next() % 8 == 0 {
                            "see http://z.w\n"
                        } else {
                            "let value = compute(input);\n"
                        });
                    }
                }
                let start = body.len();
                for _ in 0..next() % 40 {
                    body.push_str(if next() % 12 == 0 {
                        needles[next() % needles.len()]
                    } else {
                        "ordinary prose "
                    });
                }
                blocks.push((start, body.len()));
                body.push('\n');
            }
            let allocator = Allocator::new();
            let triggers = AutolinkTriggers::new(&allocator, body.as_bytes());
            for &(start, end) in &blocks {
                assert!(
                    ask(&triggers, &body, start, end),
                    "round {round}: {start}..{end}"
                );
            }
            assert!(!triggers.is_exhausted(), "round {round}");
            // Asked again out of order: answered from the current run or by
            // the full pass, and never wrongly.
            for &(start, end) in blocks.iter().rev() {
                ask(&triggers, &body, start, end);
            }
        }
    }

    /// Bytes of a one-chunk body holding exactly `count` triggers, all `w.`
    /// pairs so none of them sends a block to the full pass.
    fn body_with_triggers(len: usize, count: usize) -> String {
        let mut body = "x".repeat(len);
        for index in 0..count {
            let at = index * 3;
            body.replace_range(at..at + 2, "w.");
        }
        body
    }

    #[test]
    fn the_record_stops_just_past_its_bound() {
        // One block covering one whole chunk: its bound is the allowance plus
        // one offset per `BYTES_PER_OFFSET` bytes read.
        let bound = OFFSET_ALLOWANCE + CHUNK / BYTES_PER_OFFSET;
        let allocator = Allocator::new();
        for (count, exhausted) in [(bound - 1, false), (bound, false), (bound + 1, true)] {
            let body = body_with_triggers(CHUNK, count);
            let triggers = AutolinkTriggers::new(&allocator, body.as_bytes());
            let answered = ask(&triggers, &body, 0, body.len());
            assert_eq!(answered, !exhausted, "{count} triggers");
            assert_eq!(triggers.is_exhausted(), exhausted, "{count} triggers");
            // Once stopped, every later block takes the full pass.
            assert_eq!(ask(&triggers, &body, 1, 10), !exhausted, "{count} triggers");
        }
    }

    #[test]
    fn a_dense_prefix_stops_the_record_for_the_sparse_rest() {
        // The first block's chunk holds more triggers than its own bound
        // allows, although the body as a whole would not: the bound is
        // checked on what is read, so the record stops at the first block.
        let bound = OFFSET_ALLOWANCE + CHUNK / BYTES_PER_OFFSET;
        let mut body = body_with_triggers(CHUNK, bound + 1);
        for line in 0..60 {
            body.push_str(if line % 10 == 0 {
                "see www.example.com today.\n"
            } else {
                "sparse prose without needles.\n"
            });
        }
        assert!(bound + 1 + 6 <= OFFSET_ALLOWANCE + body.len() / BYTES_PER_OFFSET);
        let allocator = Allocator::new();
        let triggers = AutolinkTriggers::new(&allocator, body.as_bytes());
        assert!(!ask(&triggers, &body, 0, 100));
        assert!(triggers.is_exhausted());
        for start in (CHUNK..body.len()).step_by(37) {
            assert!(!ask(&triggers, &body, start, body.len().min(start + 36)));
        }
        // Read as one block, the same body stays under its bound.
        let whole = AutolinkTriggers::new(&allocator, body.as_bytes());
        assert!(ask(&whole, &body, 0, body.len()));
        assert!(!whole.is_exhausted());
    }

    #[test]
    fn a_sparse_prefix_leaves_room_for_a_dense_block() {
        // The bound counts every byte read so far: a dense block after a long
        // sparse stretch is still answered while the total stays under it.
        let sparse = "sparse prose without needles.\n".repeat(40);
        let bound = OFFSET_ALLOWANCE + (sparse.len() + CHUNK) / BYTES_PER_OFFSET;
        let dense_count = OFFSET_ALLOWANCE + CHUNK / BYTES_PER_OFFSET + 4;
        assert!(dense_count <= bound);
        let body = format!("{sparse}{}", body_with_triggers(CHUNK, dense_count));
        let allocator = Allocator::new();
        let triggers = AutolinkTriggers::new(&allocator, body.as_bytes());
        for start in (0..sparse.len()).step_by(30) {
            assert!(ask(&triggers, &body, start, start + 29));
        }
        assert!(ask(&triggers, &body, sparse.len(), body.len()));
        assert!(!triggers.is_exhausted());
    }

    #[test]
    fn content_outside_the_body_is_not_answered() {
        let body = "text www.example.com text";
        let allocator = Allocator::new();
        let triggers = AutolinkTriggers::new(&allocator, body.as_bytes());
        let copy = body.to_owned();
        assert_eq!(triggers.cover(&copy), None);
        assert_eq!(triggers.cover(&copy[5..]), None);
        assert_eq!(triggers.cover(&body[5..]), Some(5));
        assert_eq!(triggers.cover(&body[body.len()..]), Some(body.len()));
        // Slices that start inside an indexed prefix but run past its end.
        let longer = format!("{body} and more");
        let prefix = AutolinkTriggers::new(&allocator, &longer.as_bytes()[..body.len()]);
        assert_eq!(prefix.cover(&longer[..body.len()]), Some(0));
        assert_eq!(prefix.cover(&longer[1..=body.len()]), None);
        assert_eq!(prefix.cover(&longer[body.len()..]), None);
        assert_eq!(prefix.cover(&longer), None);
    }
}
