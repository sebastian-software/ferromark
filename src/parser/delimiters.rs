//! Shared delimiter scans for links, images, code spans, math and MDX.
//!
//! Keep marker runs, closed-code skipping and cached closer discovery alongside
//! balanced bracket scanning. Syntax-specific parsing stays with its construct.

use memchr::memchr;
use smallvec::SmallVec;

use super::Parser;
use super::byte_class::ByteClass;

impl<'a> Parser<'a> {
    pub(super) fn marker_run_len(bytes: &[u8], start: usize, marker: u8) -> usize {
        let mut count = 1;
        while start + count < bytes.len() && bytes[start + count] == marker {
            count += 1;
        }
        count
    }

    /// Returns the byte after a closed code span that opens at `start`.
    ///
    /// Inline constructs outside code spans use this to skip over backtick
    /// regions while scanning for their own closing delimiter. An unmatched
    /// opener stays literal and therefore is not skipped.
    pub(super) fn closed_code_span_end(bytes: &[u8], start: usize) -> Option<usize> {
        let open_len = Self::marker_run_len(bytes, start, b'`');
        let mut cursor = start + open_len;
        while cursor < bytes.len() {
            let relative = memchr::memchr(b'`', &bytes[cursor..])?;
            cursor += relative;
            let close_len = Self::marker_run_len(bytes, cursor, b'`');
            if close_len == open_len {
                return Some(cursor + close_len);
            }
            cursor += close_len;
        }
        None
    }

    /// Reports whether `closer` occurs at or after `from` in `content`.
    ///
    /// The balanced scans (`scan_balanced` for `]`, `skip_braces` for `}`)
    /// only report that nothing closed after walking to the end of the
    /// content, so a run of unclosed openers pays one full walk each and
    /// costs O(n²). A forward window settles the run instead: the answer is
    /// monotone in `from` — a closer at or after `from` is one at or after
    /// every earlier position too — so the bytes one opener reads answer for
    /// every opener behind it, and the run costs one scan in total.
    ///
    /// See [`CloserWindow`] for the invariant and for why the window is kept
    /// in addresses. Keeping it that way is what lets the same window answer
    /// for a slice and for the sub-slices of it that nested content is
    /// parsed from, instead of being rebuilt for each.
    pub(super) fn has_closer_from(&self, content: &'a str, from: usize, closer: u8) -> bool {
        let bytes = content.as_bytes();
        if from >= bytes.len() {
            return false;
        }
        let base = bytes.as_ptr() as usize;
        // The question and the end of the slice, as addresses.
        let at = base + from;
        let end = base + bytes.len();

        let cell = &self.closer_windows[closer_slot(closer)];
        let cached = cell.get();
        let known = cached.closer == closer;

        // The bytes this question still has to read, and the window that
        // decides it when they hold no closer.
        let (start, stop, beyond) = if known && at >= cached.lo && at <= cached.hi {
            if cached.hi >= end {
                // Clean from here past the end of this slice.
                return false;
            }
            if cached.found {
                // The closer at `hi` is at or after `at` and inside the slice.
                return true;
            }
            // Clean up to `hi`, unread from there on.
            (cached.hi - base, bytes.len(), None)
        } else if known && at < cached.lo && cached.lo <= end && (cached.found || cached.hi >= end)
        {
            // Behind the window: only `at..lo` is unread, and what the window
            // knows from `lo` on already settles the rest of the slice.
            (from, cached.lo - base, Some(cached))
        } else {
            (from, bytes.len(), None)
        };

        let next = memchr(closer, &bytes[start..stop]).map(|offset| base + start + offset);
        let window = match (next, beyond) {
            // A closer inside the bytes just read, so inside the slice.
            (Some(hit), _) => CloserWindow {
                closer,
                lo: at,
                hi: hit,
                found: true,
            },
            // Nothing up to `lo`, so the window's own answer stands, now
            // reaching back to `at`.
            (None, Some(settled)) => CloserWindow { lo: at, ..settled },
            // Nothing in `at..stop`, and `stop` is the end of the slice.
            (None, None) => CloserWindow {
                closer,
                lo: at,
                hi: base + stop,
                found: false,
            },
        };
        cell.set(window);
        window.found && window.hi < end
    }

    /// Reports whether `]]` occurs at or after `from` in `content`.
    ///
    /// The wiki-link scan walks to the end of the content to find that
    /// nothing closes a `[[`, so a run of unclosed openers paid one walk each
    /// — `[`×n `a]` cost 8 s at 128 KB with wiki links on. The position of
    /// the last closer settles it for every opener in the slice at once,
    /// which is the bound [`Self::has_closer_from`] carries for the plain
    /// closers.
    pub(super) fn has_wiki_closer_from(&self, content: &'a str, from: usize) -> bool {
        let key = (content.as_ptr() as usize, content.len());

        let cached = self
            .extension_memos()
            .wiki_closer
            .borrow()
            .get(&key)
            .copied();
        let last = if let Some(last) = cached {
            last
        } else {
            let last = memchr::memmem::rfind(content.as_bytes(), b"]]");
            self.extension_memos()
                .wiki_closer
                .borrow_mut()
                .insert(key, last);
            last
        };

        last.is_some_and(|last| last >= from)
    }

    /// Scans a bracketed region and returns the index of the `]` that closes
    /// it, or `content.len()` when the brackets never balance, together with
    /// whether an unescaped `[` occurred inside the region. Callers use that
    /// flag in place of a second search for nested brackets.
    ///
    /// Constructs that bind tighter than brackets are skipped whole:
    /// backslash escapes, code spans (an unmatched opener stays literal),
    /// autolinks, and inline raw HTML. This is what makes
    /// `[not a `link](/foo`)` a code span instead of a link.
    ///
    /// Only the five bytes that can change the verdict are inspected; the
    /// ordinary text between them is skipped with [`BRACKET_STOP`]. Link
    /// text is the second-largest scalar walk after destinations on
    /// link-dense documents, so this matters for every `[`.
    pub(super) fn scan_balanced(content: &str, cursor: usize) -> (usize, bool) {
        Self::walk_balanced::<false>(content, cursor, &mut |_, _, _| {})
    }

    /// [`Self::scan_balanced`], answered from the openers an earlier walk
    /// already decided, so a run of nested brackets costs one walk instead
    /// of one per level.
    ///
    /// Every decision the walk makes — escape, code span, autolink, raw HTML
    /// — depends on the position alone and never on where the walk started,
    /// so two walks that both reach a position normally agree from there on.
    /// The `]` that returns a walk to an opener's own depth is therefore the
    /// `]` a walk starting just after that opener stops at, and a suffix one
    /// walk ends unbalanced is one such a walk ends unbalanced too. An opener
    /// inside a region a walk skipped whole is never recorded, so a scan that
    /// starts inside a code span still walks for itself.
    pub(super) fn scan_balanced_matched(&self, content: &'a str, cursor: usize) -> (usize, bool) {
        if let Some(matched) = self.bracket_match(content, cursor) {
            return matched;
        }
        let (close, nested) = Self::scan_balanced(content, cursor);
        if nested && close >= content.len() {
            // Nothing closes this bracket, and the same is true for every
            // opener behind it in the run — the shape that walked to the end
            // of the content once per opener. One more walk decides them all.
            return self.record_bracket_matches(content, cursor);
        }
        (close, nested)
    }

    /// Walks from `cursor` keeping the match of every opener it passes, so
    /// the brackets nested below it are answered from the map.
    ///
    /// Only called where the walk pays for itself: a region about to be
    /// parsed in place, or a run of openers with nothing to close them.
    /// Bracket text that is never re-walked must not pay for the map.
    pub(super) fn record_bracket_matches(&self, content: &'a str, cursor: usize) -> (usize, bool) {
        if let Some(matched) = self.bracket_match(content, cursor) {
            return matched;
        }
        let base = content.as_ptr() as usize;
        let end = base + content.len();
        Self::walk_balanced::<true>(content, cursor, &mut |start, close, nested| {
            self.bracket_matches()
                .borrow_mut()
                .insert((base + start, end), (close - start, nested));
        })
    }

    /// The first byte at or after `from` that could start an inline
    /// construct other than a bracket, or `content.len()` when there is
    /// none.
    ///
    /// Answers whether a bracket's text can be parsed where it stands
    /// before any of it is parsed: a region that holds one of these bytes
    /// is handed to the probe untouched, so the two paths never both run
    /// over the same text. One memo covers a whole nested run, which is
    /// what keeps the question O(1) per level.
    pub(super) fn next_bracket_text_stop(&self, content: &'a str, from: usize) -> usize {
        let memo = self.bracket_text_stop.get();
        let base = content.as_ptr() as usize;
        if memo.content == base
            && memo.len == content.len()
            && from >= memo.origin
            && from <= memo.hit
        {
            return memo.hit;
        }
        let hit = BRACKET_TEXT_STOP.first_in(content.as_bytes(), from);
        self.bracket_text_stop.set(ForwardMemo {
            content: base,
            len: content.len(),
            origin: from,
            hit,
        });
        hit
    }

    /// The recorded match for a scan of `content` starting at `cursor`.
    ///
    /// A walk that recorded an opener recorded every opener inside it too,
    /// so one hit here means the whole region below is answered.
    fn bracket_match(&self, content: &str, cursor: usize) -> Option<(usize, bool)> {
        // Only a recording walk allocates the table, so a parse that never
        // met a nested run answers every `[` here without a lookup.
        let matched = self.bracket_matches.get()?.borrow();
        if matched.is_empty() {
            return None;
        }
        let base = content.as_ptr() as usize;
        matched
            .get(&(base + cursor, base + content.len()))
            .map(|&(span, nested)| (cursor + span, nested))
    }

    /// The bracket walk both scans above are.
    ///
    /// With `RECORD` the walk keeps the opener stack it otherwise only
    /// counts, and reports the scan start, the closing bracket and the
    /// nested flag for the scan itself and for every opener it passes — at
    /// that opener's own closing bracket, or at the end of the content for
    /// the ones that never close.
    fn walk_balanced<const RECORD: bool>(
        content: &str,
        cursor: usize,
        matched: &mut impl FnMut(usize, usize, bool),
    ) -> (usize, bool) {
        let bytes = content.as_bytes();
        // Deep enough for any nesting a document holds by hand; the shapes
        // that go past it are the ones this record exists for, and they
        // spill once. Never touched without `RECORD`.
        let mut open: SmallVec<[(usize, u32); 16]> = SmallVec::new();
        let mut depth = 1;
        let mut opens = 0u32;
        let mut at = cursor;
        loop {
            at = BRACKET_STOP.first_in(bytes, at);
            let Some(&byte) = bytes.get(at) else {
                break;
            };
            match byte {
                b'\\' => {
                    // An escaped ASCII punctuation byte (which covers both
                    // delimiters) is inert for bracket matching.
                    let escapes_next = at + 1 < bytes.len() && bytes[at + 1].is_ascii_punctuation();
                    at += if escapes_next { 2 } else { 1 };
                }
                b'`' => {
                    let run = Self::marker_run_len(bytes, at, b'`');
                    at += run;
                    let mut scan = at;
                    while scan < bytes.len() {
                        let Some(off) = memchr(b'`', &bytes[scan..]) else {
                            break;
                        };
                        scan += off;
                        let closer = Self::marker_run_len(bytes, scan, b'`');
                        if closer == run {
                            at = scan + closer;
                            break;
                        }
                        scan += closer;
                    }
                }
                b'<' => {
                    if let Some(end) = super::inline::autolink_end(content, at) {
                        at = end;
                    } else if let Some((_, end)) = Parser::parse_inline_html(content, at, 0) {
                        at = end;
                    } else {
                        at += 1;
                    }
                }
                b'[' => {
                    depth += 1;
                    opens += 1;
                    if RECORD {
                        open.push((at + 1, opens));
                    }
                    at += 1;
                }
                b']' => {
                    depth -= 1;
                    // Stop AT the closing delimiter.
                    if depth == 0 {
                        if RECORD && opens > 0 {
                            matched(cursor, at, true);
                        }
                        return (at, opens > 0);
                    }
                    if RECORD && let Some((start, seen)) = open.pop() {
                        matched(start, at, opens > seen);
                    }
                    at += 1;
                }
                _ => at += 1,
            }
        }
        if RECORD && opens > 0 {
            // Nothing after `at` can close these, so every opener still open
            // ends the content unbalanced, exactly as its own scan would.
            matched(cursor, at, true);
            for (start, seen) in open {
                matched(start, at, opens > seen);
            }
        }
        (at, opens > 0)
    }
}

/// One memoized forward byte search over one slice.
///
/// Same shape as the inline marker scan's memo: when the first hit at or
/// after `origin` is `hit`, it is still `hit` for every position in
/// `origin..=hit`, so a walk that moves forward recomputes only when it
/// leaves that window. `content` and `len` name the slice the window
/// belongs to, since nested content is parsed as slices of one buffer.
#[derive(Clone, Copy, Default)]
pub(super) struct ForwardMemo {
    content: usize,
    len: usize,
    origin: usize,
    hit: usize,
}

/// One forward window over the bytes a parser reads, for
/// [`Parser::has_closer_from`]: no `closer` occurs at any address in
/// `lo..hi`, and when `found`, the byte at `hi` is one.
///
/// Same shape as [`ForwardMemo`] and the same reason: the answer is monotone
/// in the position asked about, so the closer found once ahead of `lo`
/// answers for every position up to it, and only a question that leaves the
/// window reads anything.
///
/// It is kept in addresses rather than in offsets of one named slice because
/// nested content is parsed as slices of one buffer, and a statement about a
/// range of that buffer holds for every slice that covers the range. A window
/// built for a paragraph therefore still answers inside a bracket text cut
/// out of it, and one built for the whole source still answers inside the
/// paragraphs of it — which a window naming one slice would have to rebuild,
/// re-reading the same bytes once per alternation. Live slices never overlap
/// across buffers, so a window can only be read where it was written.
///
/// A default window names the empty range at address zero with `closer` 0,
/// which no question can match: every closer asked about is an ASCII
/// punctuation byte.
#[derive(Clone, Copy, Default)]
pub(super) struct CloserWindow {
    /// The closing byte this window answers for.
    closer: u8,
    /// Whether a `closer` sits at `hi`. When it does not, `hi` is only as
    /// far as the bytes behind the window have been read.
    found: bool,
    /// First address the window covers.
    lo: usize,
    /// One past the last address known to hold no `closer`.
    hi: usize,
}

/// Number of windows [`Parser::has_closer_from`] keeps, one per closer.
pub(super) const CLOSER_SLOTS: usize = 3;

/// The window slot a closer's answers live in.
///
/// The `[`, `<` and `{` of one paragraph ask about the same slice in turn, so
/// a single window between them would be rebuilt by every other question, and
/// a run of openers that nothing closes would read the content once each —
/// the quadratic shape the window exists to bound. A slot per closer keeps
/// the three questions on their own forward walk. Any other byte shares the
/// last slot, and since the slot names the closer it answers for, sharing
/// costs a scan and never a wrong answer.
const fn closer_slot(closer: u8) -> usize {
    match closer {
        b']' => 0,
        b'>' => 1,
        _ => 2,
    }
}

/// Bytes that can change the outcome of [`Parser::scan_balanced`]: the
/// escape and code-span markers, the start of an autolink or raw HTML tag,
/// and the brackets themselves.
static BRACKET_STOP: ByteClass = ByteClass::from_flags({
    let mut t = [0u8; 256];
    t[b'\\' as usize] = 1;
    t[b'`' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'[' as usize] = 1;
    t[b']' as usize] = 1;
    t
});

/// Every byte that can start an inline construct, except `[`.
///
/// This is the marker set of `InlineMarkerScan` with every optional marker
/// switched on and the bracket taken out, so a region without one of these
/// holds nothing but text and brackets whatever the options say. Being
/// wider than the enabled marker set only refuses more regions, never
/// fewer.
static BRACKET_TEXT_STOP: ByteClass = ByteClass::from_flags({
    let mut t = [0u8; 256];
    t[b'*' as usize] = 1;
    t[b'_' as usize] = 1;
    t[b'`' as usize] = 1;
    t[b'!' as usize] = 1;
    t[b'~' as usize] = 1;
    t[b'\\' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'&' as usize] = 1;
    t[b'\n' as usize] = 1;
    t[b'\r' as usize] = 1;
    t[b'{' as usize] = 1;
    t[b'^' as usize] = 1;
    t[b'$' as usize] = 1;
    t[b'=' as usize] = 1;
    t
});

#[cfg(test)]
mod tests;
