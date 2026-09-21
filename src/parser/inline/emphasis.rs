//! Emphasis, strong emphasis, GFM strikethrough, and marked text via the delimiter stack.
//!
//! During inline parsing every enabled `*`/`_`/`~`/`==` run is pushed as a plain text node
//! plus a [`Delimiter`] record carrying its flanking classification. Once
//! the inline sequence is complete, [`Parser::process_emphasis`] pairs
//! closers with openers (nearest matching opener, rule of three), wraps
//! the nodes between into `Emphasis`/`Strong`/`Delete`/`Highlight`, and trims the delimiter
//! text nodes in place. Unpaired runs simply stay literal text.

use crate::allocator::Vec;
use crate::ast::{Node, Span, Text};

use crate::parser::Parser;
use crate::parser::error::{ParseErrorKind, ParseResult};

pub(in crate::parser) struct Delimiter {
    /// Index of the run's text node in the children vec.
    node_index: usize,
    marker: u8,
    /// Original run length (rule-of-three checks use this).
    orig_len: usize,
    /// Unconsumed delimiter characters remaining in the text node.
    remaining: usize,
    can_open: bool,
    can_close: bool,
    /// Depth of the emphasis node this run currently holds as an opener,
    /// counted in nodes below and including it, or zero while it holds
    /// none. Pairing reads it back to measure the tree it is building.
    depth: u32,
    /// The runs still worth visiting, threaded as a doubly linked list
    /// over the vector (`NO_DELIMITER` at either end).
    ///
    /// A closer looks for its opener by walking back through the runs
    /// before it, and a run that pairing has retired — everything strictly
    /// inside a pair, or an unequal strikethrough pair with its contents —
    /// can never take part again. Skipping such runs one by one still
    /// visits them, and openers retired in one place and closed from
    /// another made the walk quadratic: `~a`×n `b_`×m `a~~`×n visited the
    /// same retired entries once per closer. Unlinking a retired run costs
    /// it one visit in total.
    prev: usize,
    next: usize,
}

/// The end of the delimiter list in either direction.
const NO_DELIMITER: usize = usize::MAX;

/// Takes `index` out of the list of runs still worth visiting.
///
/// The depth of the node the run holds is folded into the run before it,
/// which is the opener of the pair that retired it or a run inside that
/// pair, so a pair built later around either still measures the tree it
/// encloses. `remaining` is the caller's to zero: it marks the run spent
/// for the pairing loop, and an unlinked run is always spent.
fn unlink(delimiters: &mut [Delimiter], index: usize) {
    let (prev, next, depth) = {
        let run = &delimiters[index];
        (run.prev, run.next, run.depth)
    };
    if prev != NO_DELIMITER {
        delimiters[prev].next = next;
        delimiters[prev].depth = delimiters[prev].depth.max(depth);
    }
    if next != NO_DELIMITER {
        delimiters[next].prev = prev;
    }
}

impl<'a> Parser<'a> {
    /// Records a `*`/`_`/`~`/`==` run: pushes its text node and the delimiter
    /// entry describing how it may participate in emphasis.
    pub(in crate::parser) fn push_delimiter_run(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        delimiters: &mut Vec<'a, Delimiter>,
        pos: &mut usize,
    ) {
        let bytes = content.as_bytes();
        let marker = bytes[*pos];
        let run_len = Self::marker_run_len(bytes, *pos, marker);

        let (can_open, can_close) =
            self.classify_run(marker, &content[..*pos], &content[*pos + run_len..]);

        Self::push_text(
            children,
            &content[*pos..*pos + run_len],
            offset + *pos,
            offset + *pos + run_len,
        );
        *pos += run_len;
        // A run that can neither open nor close — an intraword `_`, a run
        // with whitespace on both sides — is literal text, and the spec
        // keeps it off the delimiter stack. Recording it anyway only gave
        // every later opener search one more entry to step over.
        if !can_open && !can_close {
            return;
        }
        let index = delimiters.len();
        let prev = match delimiters.last_mut() {
            Some(last) => {
                last.next = index;
                index - 1
            }
            None => NO_DELIMITER,
        };
        delimiters.push(Delimiter {
            node_index: children.len() - 1,
            marker,
            orig_len: run_len,
            remaining: run_len,
            can_open,
            can_close,
            depth: 0,
            prev,
            next: NO_DELIMITER,
        });
    }

    /// Flanking classification for the run between `before` and `after`.
    ///
    /// The neighbouring characters decide it, and in prose both are ASCII,
    /// where the character classes reduce to byte tests: no ASCII character
    /// is East Asian punctuation, so `cjk_emphasis` cannot apply either. A
    /// multibyte neighbour fails the ASCII test on its lead or continuation
    /// byte and takes the character path, which is the only one that has to
    /// decode.
    fn classify_run(&self, marker: u8, mut before: &'a str, mut after: &'a str) -> (bool, bool) {
        if self.options.strikethrough && marker != b'~' {
            // GFM emphasis looks through adjacent extension markers when
            // classifying flanking, as cmark-gfm does for strikethrough.
            // Only an adjacent marker can be trimmed, so test for one first.
            if before.as_bytes().last() == Some(&b'~') {
                before = before.trim_end_matches('~');
            }
            if after.as_bytes().first() == Some(&b'~') {
                after = after.trim_start_matches('~');
            }
        }

        let prev_byte = before.as_bytes().last().copied();
        let next_byte = after.as_bytes().first().copied();
        if prev_byte.is_none_or(|byte| byte.is_ascii())
            && next_byte.is_none_or(|byte| byte.is_ascii())
        {
            return classify_flanking_ascii(marker, prev_byte, next_byte);
        }
        classify_flanking(
            marker,
            before.chars().next_back(),
            after.chars().next(),
            self.options.cjk_emphasis,
        )
    }

    /// Pairs delimiters and restructures `children` into the emphasis
    /// tree, spec algorithm "process emphasis".
    ///
    /// Two things keep it linear on delimiter-dense sequences. Paired
    /// nodes are lifted out of `children` in place, leaving empty text
    /// slots behind, so `node_index` never moves and no pairing has to
    /// walk the vector fixing indices up. And a failed opener search
    /// records how far down it looked (`openers_bottom`), so the next
    /// closer of the same class does not repeat it.
    ///
    /// # Bounding the tree
    ///
    /// Pairing itself is iterative, but the tree it builds is walked
    /// recursively by the HTML renderer and by the public `ast::visit`
    /// walkers, so it is bounded by
    /// [`ParserOptions::max_nesting_depth`](crate::ParserOptions::max_nesting_depth)
    /// like every other nesting: `*`×2n `a` `*`×2n nests n levels deep and
    /// used to overflow the stack and abort the process.
    ///
    /// Depth is the depth of the tree, not the length of a run. Adjacent
    /// pairs (`*a* *b*`) stay one level deep however many there are; only
    /// pairs that enclose one another go deeper, and a closer always pairs
    /// with the nearest opener, so a new node's depth is one more than the
    /// deepest thing it encloses. Each opener remembers the depth of the
    /// node it holds, which the delimiters it encloses (and the opener
    /// itself, when it pairs again with characters left over) already have
    /// to be walked for, so the measurement costs nothing a pairing did not
    /// already pay and a long run stays linear.
    ///
    /// Two counts complete it. `inline_depth` is what encloses this whole
    /// sequence — the link text, image alt or JSX phrasing it sits in — and
    /// `nested_inline_depth` is the deepest subtree finished inside it, so
    /// the bound holds along a path through both kinds of nesting rather
    /// than per sequence, which is what a mixture like `<A>`/`*` nested
    /// 100×100 needs.
    pub(in crate::parser) fn process_emphasis(
        &self,
        children: &mut Vec<'a, Node<'a>>,
        delimiters: &mut Vec<'a, Delimiter>,
    ) -> ParseResult<()> {
        // Nothing else in inline parsing produces an empty text node, so the
        // sweep at the end is only owed the ones pairing leaves behind. The
        // spec suites and snapshot corpora hold this assertion up.
        debug_assert!(
            !children.iter().any(is_empty_text),
            "inline parsing produced an empty text node before pairing"
        );
        // Everything already nested inside this sequence counts against the
        // same budget as the nodes pairing is about to build, and so does
        // everything the sequence itself is nested in. Both are read once:
        // the scan is finished, so neither can change from here on.
        let nested = self.nested_depth_cell();
        let inside = nested.get() as u32;
        let enclosing = self.inline_depth.get().saturating_sub(1);
        let mut produced = inside;

        let mut openers_bottom = OpenersBottom::default();
        let mut emptied = false;
        let mut closer_idx = 0;
        while closer_idx < delimiters.len() {
            if !delimiters[closer_idx].can_close || delimiters[closer_idx].remaining == 0 {
                closer_idx += 1;
                continue;
            }

            let bottom = openers_bottom.get(&delimiters[closer_idx]);
            let Some(opener_idx) = find_opener(delimiters, closer_idx, bottom) else {
                // Nothing below this closer can pair with a later closer of
                // the same class either, so remember where to stop next time.
                openers_bottom.set(&delimiters[closer_idx]);
                if !delimiters[closer_idx].can_open {
                    // It cannot open and has just failed to close: retire it
                    // rather than shifting the rest of the vector down.
                    delimiters[closer_idx].remaining = 0;
                    unlink(delimiters, closer_idx);
                }
                closer_idx += 1;
                continue;
            };

            let strikethrough = delimiters[closer_idx].marker == b'~';
            if strikethrough && delimiters[opener_idx].remaining != delimiters[closer_idx].remaining
            {
                // cmark-gfm consumes the delimiter records for an unequal
                // single/double pair, retaining their literal text. Retire
                // the enclosed records too: they cannot pair across it later.
                let mut index = opener_idx;
                loop {
                    let next = delimiters[index].next;
                    delimiters[index].remaining = 0;
                    unlink(delimiters, index);
                    if index == closer_idx {
                        break;
                    }
                    index = next;
                }
                closer_idx += 1;
                continue;
            }

            let use_delims: u32 =
                if delimiters[opener_idx].remaining >= 2 && delimiters[closer_idx].remaining >= 2 {
                    2
                } else {
                    1
                };
            let opener_node = delimiters[opener_idx].node_index;
            let closer_node = delimiters[closer_idx].node_index;

            // Retire the delimiters strictly inside the pair, which are now
            // unreachable, and take the depth of the nodes they hold on the
            // way: those are exactly the nodes pairing has already built
            // inside the range about to be lifted. The opener counts too —
            // with characters left over it pairs again around the node it
            // already holds, which is how `***a** b*` nests. Retiring an
            // entry means zeroing `remaining` and unlinking it: erasing it
            // from the vector instead would shift every later entry down —
            // which is what made a paragraph full of emphasis quadratic —
            // and leaving it linked would keep every later opener search
            // stepping over it. Only the runs still linked are visited;
            // a run retired earlier already folded its depth into one of
            // them (see `unlink`).
            let mut enclosed = delimiters[opener_idx].depth;
            let mut inner_idx = delimiters[opener_idx].next;
            while inner_idx != closer_idx {
                debug_assert_ne!(
                    inner_idx, NO_DELIMITER,
                    "the closer is linked after its opener"
                );
                let next = delimiters[inner_idx].next;
                enclosed = enclosed.max(delimiters[inner_idx].depth);
                delimiters[inner_idx].remaining = 0;
                unlink(delimiters, inner_idx);
                inner_idx = next;
            }
            let depth = enclosed.max(inside) + 1;
            if self.options.max_nesting_depth > 0
                && enclosing + depth as usize > self.options.max_nesting_depth
            {
                return Err(ParseErrorKind::NestingTooDeep {
                    span: Span::new(
                        children[closer_node].span().start,
                        children[closer_node].span().start,
                    ),
                    max_depth: self.options.max_nesting_depth,
                }
                .into());
            }
            produced = produced.max(depth);

            // Lift the nodes between the delimiters into the new emphasis
            // node and leave empty text behind, so every index stays put.
            // Text nodes emptied by earlier (inner) pairings are dropped on
            // the way — they render as nothing. The range is never empty:
            // two runs of the same marker cannot be adjacent children.
            let mut inner = self.allocator.new_vec();
            for slot in &mut children[opener_node + 1..closer_node] {
                let node = core::mem::replace(slot, empty_text());
                if !is_empty_text(&node) {
                    inner.push(node);
                }
            }
            // The slot right after the opener takes the new node below, so
            // only a wider range leaves a placeholder behind.
            emptied |= closer_node > opener_node + 2;
            let span = inner_span(&inner, use_delims);
            let node = if delimiters[closer_idx].marker == b'=' {
                Node::Highlight(self.allocator.boxed(crate::ast::Highlight {
                    children: inner,
                    span,
                }))
            } else if strikethrough {
                Node::Delete(self.allocator.boxed(crate::ast::Delete {
                    children: inner,
                    span,
                }))
            } else if use_delims == 2 {
                Node::Strong(self.allocator.boxed(crate::ast::Strong {
                    children: inner,
                    span,
                }))
            } else {
                Node::Emphasis(self.allocator.boxed(crate::ast::Emphasis {
                    children: inner,
                    span,
                }))
            };
            children[opener_node + 1] = node;

            // Trim the delimiter text nodes in place (they may end up
            // empty, which renders as nothing).
            emptied |= trim_text_tail(&mut children[opener_node], use_delims);
            emptied |= trim_text_head(&mut children[closer_node], use_delims);

            // The pair has spent `use_delims` characters, and the opener now
            // holds the node just built.
            delimiters[opener_idx].depth = depth;
            delimiters[opener_idx].remaining -= use_delims as usize;
            delimiters[closer_idx].remaining -= use_delims as usize;
            // Stay on the closer: with characters left it retries against an
            // earlier opener, and otherwise the top of the loop steps over it.
        }

        // Emptied delimiter text nodes at this level render as nothing;
        // drop them so consumers see a clean tree. A paragraph whose runs
        // never paired — a stray `*`, an intraword `_`, a lone `~` — has
        // nothing to drop, and it would otherwise pay a full sweep of its
        // children for the delimiters alone.
        if emptied {
            children.retain(|node| !is_empty_text(node));
        }

        // Hand the finished depth to the level above, which wraps this
        // sequence in one more node (see `InlineDepthGuard`).
        nested.set(produced as usize);
        Ok(())
    }
}

/// Whether `node` is the empty text a lifted or trimmed slot leaves behind.
fn is_empty_text(node: &Node<'_>) -> bool {
    matches!(node, Node::Text(text) if text.value.is_empty())
}

/// Placeholder left where a node has been lifted into an emphasis node.
/// Empty text renders as nothing and is dropped once pairing is done.
fn empty_text<'a>() -> Node<'a> {
    Node::Text(Text {
        value: "",
        span: Span::new(0, 0),
    })
}

/// Lowest opener still worth examining, per closer class.
///
/// The spec keys this on the delimiter character, the closer's original
/// run length modulo three, and whether the closer can also open — the
/// three things the rule of three consults. The stored value is a
/// `node_index`, which no longer moves, so it stays a valid bound for the
/// whole sequence.
#[derive(Default)]
struct OpenersBottom {
    star: [[Option<usize>; 2]; 3],
    underscore: [[Option<usize>; 2]; 3],
    tilde: [[Option<usize>; 2]; 3],
    equals: [[Option<usize>; 2]; 3],
}

impl OpenersBottom {
    fn slot(&mut self, closer: &Delimiter) -> &mut Option<usize> {
        let table = match closer.marker {
            b'_' => &mut self.underscore,
            b'~' => &mut self.tilde,
            b'=' => &mut self.equals,
            _ => &mut self.star,
        };
        &mut table[closer.orig_len % 3][usize::from(closer.can_open)]
    }

    fn get(&mut self, closer: &Delimiter) -> Option<usize> {
        *self.slot(closer)
    }

    fn set(&mut self, closer: &Delimiter) {
        *self.slot(closer) = Some(closer.node_index);
    }
}

/// Nearest opener that may pair with `delimiters[closer_idx]`, stopping at
/// `bottom` — a `node_index` a previous failed search for this closer class
/// already proved nothing below could match.
///
/// Walks the linked runs, so retired ones cost nothing; a spent run that
/// still holds a node stays linked (its depth is read by the pair around it)
/// and is skipped by its `remaining` of zero.
fn find_opener(
    delimiters: &[Delimiter],
    closer_idx: usize,
    bottom: Option<usize>,
) -> Option<usize> {
    let closer = &delimiters[closer_idx];
    let mut opener_idx = closer.prev;
    while opener_idx != NO_DELIMITER {
        let opener = &delimiters[opener_idx];
        if bottom.is_some_and(|bottom| opener.node_index < bottom) {
            return None;
        }
        if opener.marker == closer.marker && opener.can_open && opener.remaining != 0 {
            // Rule of three: when one side can both open and close, sums
            // divisible by three only pair if both lengths are.
            let sum_of_three = (opener.can_close || closer.can_open)
                && (opener.orig_len + closer.orig_len).is_multiple_of(3)
                && !(opener.orig_len.is_multiple_of(3) && closer.orig_len.is_multiple_of(3));
            if !sum_of_three {
                return Some(opener_idx);
            }
        }
        opener_idx = opener.prev;
    }
    None
}

fn inner_span(inner: &Vec<'_, Node<'_>>, use_delims: u32) -> Span {
    let start = inner.first().map_or(0, |node| node_span(node).start);
    let end = inner.last().map_or(start, |node| node_span(node).end);
    Span::new(start.saturating_sub(use_delims), end + use_delims)
}

fn node_span(node: &Node<'_>) -> Span {
    node.span()
}

/// Both trims report whether the node is now empty, which is what decides
/// that the sweep at the end of pairing has something to do.
fn trim_text_tail(node: &mut Node<'_>, count: u32) -> bool {
    if let Node::Text(text) = node {
        let new_len = text.value.len().saturating_sub(count as usize);
        text.value = &text.value[..new_len];
        text.span = Span::new(text.span.start, text.span.end - count);
        return text.value.is_empty();
    }
    false
}

fn trim_text_head(node: &mut Node<'_>, count: u32) -> bool {
    if let Node::Text(text) = node {
        text.value = &text.value[(count as usize).min(text.value.len())..];
        text.span = Span::new(text.span.start + count, text.span.end);
        return text.value.is_empty();
    }
    false
}

/// Flanking classification (CommonMark "Emphasis and strong emphasis").
/// Sequence boundaries count as whitespace.
///
/// `cjk_emphasis` reclassifies East Asian punctuation as an ordinary character
/// here and nowhere else; see [`ParserOptions::cjk_emphasis`].
///
/// [`ParserOptions::cjk_emphasis`]: crate::ParserOptions::cjk_emphasis
fn classify_flanking(
    marker: u8,
    prev: Option<char>,
    next: Option<char>,
    cjk_emphasis: bool,
) -> (bool, bool) {
    let is_punct =
        |ch: char| is_punctuation_like(ch) && !(cjk_emphasis && is_east_asian_punctuation(ch));

    Neighbors {
        prev_ws: prev.is_none_or(char::is_whitespace),
        next_ws: next.is_none_or(char::is_whitespace),
        prev_punct: prev.is_some_and(is_punct),
        next_punct: next.is_some_and(is_punct),
    }
    .flanking(marker)
}

/// [`classify_flanking`] for ASCII neighbours, which is what prose has.
///
/// The classes collapse to byte tests there: ASCII punctuation is the whole
/// of [`is_punctuation_like`] below `0x80`, none of it is East Asian, and the
/// whitespace set is the one `char::is_whitespace` accepts below `0x80` —
/// which includes the vertical tab that `u8::is_ascii_whitespace` leaves out.
fn classify_flanking_ascii(marker: u8, prev: Option<u8>, next: Option<u8>) -> (bool, bool) {
    let is_space = |byte: u8| matches!(byte, b'\t' | b'\n' | 0x0B | 0x0C | b'\r' | b' ');

    Neighbors {
        prev_ws: prev.is_none_or(is_space),
        next_ws: next.is_none_or(is_space),
        prev_punct: prev.is_some_and(|byte| byte.is_ascii_punctuation()),
        next_punct: next.is_some_and(|byte| byte.is_ascii_punctuation()),
    }
    .flanking(marker)
}

/// How a run's two neighbouring characters classify. Sequence boundaries
/// count as whitespace, so both sides always have a class.
struct Neighbors {
    prev_ws: bool,
    next_ws: bool,
    prev_punct: bool,
    next_punct: bool,
}

impl Neighbors {
    /// The flanking rules themselves, shared by both classifications so they
    /// cannot drift apart.
    const fn flanking(&self, marker: u8) -> (bool, bool) {
        let left_flanking = !self.next_ws && (!self.next_punct || self.prev_ws || self.prev_punct);
        let right_flanking = !self.prev_ws && (!self.prev_punct || self.next_ws || self.next_punct);

        if matches!(marker, b'*' | b'~' | b'=') {
            (left_flanking, right_flanking)
        } else {
            (
                left_flanking && (!right_flanking || self.prev_punct),
                right_flanking && (!left_flanking || self.next_punct),
            )
        }
    }
}

/// Approximates the spec's Unicode punctuation class (general categories
/// P and S): anything printable that is neither alphanumeric nor
/// whitespace.
fn is_punctuation_like(ch: char) -> bool {
    ch.is_ascii_punctuation()
        || (!ch.is_ascii() && !ch.is_alphanumeric() && !ch.is_whitespace() && !ch.is_control())
}

/// Punctuation that East Asian scripts set directly against the text, with no
/// separating space — the reason CommonMark's flanking rules reject emphasis
/// that Latin text would accept.
///
/// The ranges are the fullwidth and CJK-specific blocks only. Halfwidth ASCII
/// punctuation is deliberately excluded even in CJK text: it is written the
/// same way in every script, so reclassifying it would change how ordinary
/// Latin documents parse.
///
/// U+3000 IDEOGRAPHIC SPACE falls inside the first range but is whitespace, and
/// the flanking rules test whitespace before punctuation, so it is unaffected.
fn is_east_asian_punctuation(ch: char) -> bool {
    matches!(ch,
        // CJK Symbols and Punctuation: 、。〈〉《》「」『』【】〜 and friends.
        '\u{3000}'..='\u{303F}'
        // Vertical forms and CJK Compatibility Forms: vertical/rotated variants.
        | '\u{FE10}'..='\u{FE19}'
        | '\u{FE30}'..='\u{FE4F}'
        // Small Form Variants: small comma, small full stop, small brackets.
        | '\u{FE50}'..='\u{FE6F}'
        // Fullwidth ASCII punctuation, split around the fullwidth digits and
        // letters, which stay alphanumeric.
        | '\u{FF01}'..='\u{FF0F}'
        | '\u{FF1A}'..='\u{FF20}'
        | '\u{FF3B}'..='\u{FF40}'
        | '\u{FF5B}'..='\u{FF65}'
    )
}

#[cfg(test)]
mod tests {
    use super::{classify_flanking, classify_flanking_ascii};

    /// Every marker that can carry a delimiter run, including the extension
    /// ones, plus a byte that is none of them so the `_` branch is reached
    /// the way an unknown marker would reach it.
    const MARKERS: [u8; 5] = [b'*', b'_', b'~', b'=', b'!'];

    #[test]
    fn ascii_flanking_matches_the_character_classification() {
        for marker in MARKERS {
            for prev in std::iter::once(None).chain((0..=127u8).map(Some)) {
                for next in std::iter::once(None).chain((0..=127u8).map(Some)) {
                    // `cjk_emphasis` only reclassifies East Asian
                    // punctuation, which no ASCII byte is, so both settings
                    // have to agree with the byte path.
                    for cjk_emphasis in [false, true] {
                        assert_eq!(
                            classify_flanking_ascii(marker, prev, next),
                            classify_flanking(
                                marker,
                                prev.map(char::from),
                                next.map(char::from),
                                cjk_emphasis,
                            ),
                            "marker {marker:#x}, prev {prev:?}, next {next:?}, cjk {cjk_emphasis}"
                        );
                    }
                }
            }
        }
    }
}
