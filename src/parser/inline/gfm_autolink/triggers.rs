//! The block-level autolink pre-flight, answered from the trigger offsets the
//! root scan collected.
//!
//! [`may_contain_autolink`] walks a block's whole content after the inline
//! parse has read it. With GFM autolinks on, the root parse collects the
//! offsets of every byte that can decide that answer while it scans the body
//! for NUL and `]:` anyway (see `root_scan::triggers`). A block whose content
//! is a slice of that body then only visits the offsets inside its range.
//!
//! # Why the answers are equal
//!
//! [`may_contain_autolink`] derives its answer from three facts: whether the
//! content holds an `@`, whether it holds a `://` that does not complete a
//! `](scheme` destination, and whether it holds a `www.`. With an `@` it also
//! looks for `mailto:` and `xmpp:` in front of any colon.
//!
//! - Every `@` of the body is recorded. An `@` inside the content hands the
//!   content to [`may_contain_autolink`] itself, which is the only place the
//!   `mailto:`/`xmpp:` rule matters: without an `@` neither extended scheme
//!   can make the answer differ.
//! - Every `:` of the body that a `/` follows is recorded, so every `://` of
//!   the content is visited at its colon. The same test as the full pass
//!   decides it, on the content's own bytes: `//` has to follow inside the
//!   content, and the destination check looks back no further than the
//!   content's start.
//! - Every `w` of the body that a `.` follows is recorded, so every `www.` of
//!   the content is visited at its last `w`, and counts when the two bytes in
//!   front of it and the `.` behind it are inside the content.
//!
//! Content qualifies by address alone: it has to lie inside the body the
//! offsets were collected from. Both are borrowed from the same immutable
//! source for the whole parse, so equal addresses hold equal bytes. Content a
//! sub-parser copied into the arena (block quotes, most list items, escaped
//! table cells, lines with comments removed) and every document whose body
//! held a NUL take the full pass, as do targets without the collecting scan.

use std::cell::Cell;

use super::AutolinkScan;
use super::scan::{may_contain_autolink, scheme_is_markdown_destination};

/// Offsets of the autolink trigger bytes of one root body.
pub(in crate::parser) struct AutolinkTriggers<'a> {
    /// The body the offsets index.
    body: &'a [u8],
    /// Offset of every trigger byte in `body` (see
    /// `root_scan::is_autolink_trigger`), in ascending order.
    offsets: &'a [u32],
    /// Where the last lookup ended: the index of the first offset at or past
    /// the end of the content it answered. Blocks are asked for in source
    /// order, so the next one usually starts here.
    cursor: Cell<usize>,
}

impl<'a> AutolinkTriggers<'a> {
    /// Offsets collected from `body`, which has to be the exact slice the
    /// root scan read.
    pub(in crate::parser) fn new(body: &'a [u8], offsets: &'a [u32]) -> Self {
        debug_assert!(offsets.windows(2).all(|pair| pair[0] < pair[1]));
        debug_assert!(
            offsets
                .last()
                .is_none_or(|&last| (last as usize) < body.len())
        );
        Self {
            body,
            offsets,
            cursor: Cell::new(0),
        }
    }

    /// Where `content` starts in the indexed body, or `None` when it does not
    /// lie inside it and the full pass has to answer for it.
    #[inline]
    pub(in crate::parser::inline) fn locate(&self, content: &str) -> Option<usize> {
        let start = (content.as_ptr() as usize).wrapping_sub(self.body.as_ptr() as usize);
        (start <= self.body.len() && content.len() <= self.body.len() - start).then_some(start)
    }

    /// The pre-flight's answer for `content`, which starts at `start` in the
    /// indexed body (see [`Self::locate`]).
    #[inline]
    pub(in crate::parser::inline) fn preflight(
        &self,
        content: &str,
        start: usize,
    ) -> Option<AutolinkScan> {
        debug_assert_eq!(self.locate(content), Some(start));
        let end = start + content.len();
        let bytes = content.as_bytes();
        let mut has_www = false;
        let mut has_bare_scheme = false;
        let mut index = self.first_at_or_after(start);
        while let Some(&offset) = self.offsets.get(index) {
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
        self.cursor.set(index);
        // Without an `@` neither extended scheme can apply, and a `www.` is
        // the only reason for the `www.` search.
        (has_www || has_bare_scheme).then_some(AutolinkScan {
            may_have_www: has_www,
            may_have_extended: false,
        })
    }

    /// Index of the first offset at or past `start`.
    #[inline]
    fn first_at_or_after(&self, start: usize) -> usize {
        let offsets = self.offsets;
        let before = |offset: &u32| (*offset as usize) < start;
        let hint = self.cursor.get();
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

/// Whether a root parse on this target collects the trigger offsets.
///
/// Only the NEON root scan collects them (see `root_scan::triggers`); other
/// targets keep the separate pre-flight pass.
#[cfg(not(test))]
#[inline]
pub(in crate::parser) const fn collects_triggers() -> bool {
    cfg!(target_arch = "aarch64")
}

/// Test builds collect them on every target, with the portable byte-wise
/// scan off aarch64, so every block a unit test parses checks the indexed
/// answer against the full pass (see `Parser::autolink_preflight`). A test
/// can switch the index off for a comparison.
#[cfg(test)]
pub(in crate::parser) fn collects_triggers() -> bool {
    COLLECTS_TRIGGERS.with(Cell::get)
}

#[cfg(test)]
thread_local! {
    static COLLECTS_TRIGGERS: Cell<bool> = const { Cell::new(true) };
}

/// Runs `run` with trigger collection switched on or off on this thread.
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

    use super::{AutolinkScan, AutolinkTriggers, may_contain_autolink};
    use crate::parser::root_scan::is_autolink_trigger;

    fn flags(scan: Option<AutolinkScan>) -> Option<(bool, bool)> {
        scan.map(|scan| (scan.may_have_www, scan.may_have_extended))
    }

    fn offsets(body: &[u8]) -> Vec<u32> {
        (0..body.len())
            .filter(|&at| is_autolink_trigger(body[at], body.get(at + 1).copied().unwrap_or(0)))
            .map(|at| at as u32)
            .collect()
    }

    /// Every slice of `body` on character boundaries, asked for in source
    /// order, in reverse, and nested, must get the full pass's answer.
    #[track_caller]
    fn check_every_slice(body: &str) {
        let offsets = offsets(body.as_bytes());
        let triggers = AutolinkTriggers::new(body.as_bytes(), &offsets);
        let bounds: Vec<usize> = (0..=body.len())
            .filter(|&at| body.is_char_boundary(at))
            .collect();
        let ask = |start: usize, end: usize| {
            let content = &body[start..end];
            assert_eq!(triggers.locate(content), Some(start), "{content:?}");
            let indexed = triggers.preflight(content, start);
            assert_eq!(
                flags(indexed),
                flags(may_contain_autolink(content)),
                "{content:?} at {start}..{end} of {body:?}"
            );
        };
        for &start in &bounds {
            for &end in bounds.iter().filter(|&&end| end >= start) {
                ask(start, end);
            }
        }
        for &start in bounds.iter().rev() {
            for &end in bounds.iter().rev().filter(|&&end| end >= start) {
                ask(start, end);
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
        for _ in 0..400 {
            let len = next() % 14;
            let mut body = String::new();
            for _ in 0..len {
                body.push_str(tokens[next() % tokens.len()]);
            }
            check_every_slice(&body);
        }
    }

    #[test]
    fn content_outside_the_body_is_not_answered() {
        let body = "text www.example.com text";
        let offsets = offsets(body.as_bytes());
        let triggers = AutolinkTriggers::new(body.as_bytes(), &offsets);
        let copy = body.to_owned();
        assert_eq!(triggers.locate(&copy), None);
        assert_eq!(triggers.locate(&copy[5..]), None);
        assert_eq!(triggers.locate(&body[5..]), Some(5));
        assert_eq!(triggers.locate(&body[body.len()..]), Some(body.len()));
        // Slices that start inside an indexed prefix but run past its end.
        let longer = format!("{body} and more");
        let prefix = AutolinkTriggers::new(&longer.as_bytes()[..body.len()], &offsets);
        assert_eq!(prefix.locate(&longer[..body.len()]), Some(0));
        assert_eq!(prefix.locate(&longer[1..=body.len()]), None);
        assert_eq!(prefix.locate(&longer[body.len()..]), None);
        assert_eq!(prefix.locate(&longer), None);
    }
}
