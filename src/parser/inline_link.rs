//! Inline link parsing.
//!
//! Split out of `inline_helpers` because CommonMark's "a link may not
//! contain a link" rule makes this the one inline construct that has to
//! parse its own text before it can decide what it is.
//!
//! Bracket text that holds another bracket is parsed once, where it stands
//! ([`Parser::parse_bracket_text`]), instead of being probed on its own and
//! then parsed again by the literal-bracket fallback. See
//! `docs/decisions/2026-09-17-linear-link-probe.md`.

use crate::allocator::Vec;
use crate::ast::{Link, Node, Span};

use super::Parser;
use crate::parser::error::{ParseErrorKind, ParseResult};
use crate::parser::inline::InlineMarkerScan;
use crate::parser::short_scan;

impl<'a> Parser<'a> {
    /// Parses the bracket at `pos` and reports whether it appended a link
    /// node to `children`.
    ///
    /// The answer is what a bracket around this one needs: a link may not
    /// contain a link, so an enclosing bracket that sees one here stays
    /// literal. `parse_inline_special` ignores it.
    pub(super) fn parse_link(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        markers: &mut InlineMarkerScan,
        pos: &mut usize,
    ) -> ParseResult<bool> {
        let bytes = content.as_bytes();
        let link_start = *pos;

        // `[^label]` is a footnote reference when the extension is on and
        // a definition exists; otherwise it falls through to normal link
        // handling and may still be a link label.
        if self.options.footnotes
            && bytes.get(*pos + 1) == Some(&b'^')
            && self.try_parse_footnote_reference(content, offset, children, pos)
        {
            return Ok(false);
        }

        // Every accepting branch below needs a `]`, and `scan_balanced`
        // only reports that there is none after walking to the end of the
        // content — once per bracket, which is quadratic over a run of
        // them. Settle it for the whole slice first.
        if !self.has_closer_from(content, *pos + 1, b']') {
            Self::push_text(children, "[", offset + link_start, offset + link_start + 1);
            *pos = link_start + 1;
            return Ok(false);
        }

        if self.options.wiki_links
            && bytes.get(link_start + 1) == Some(&b'[')
            && let Some((link, end)) = self.try_parse_wiki_link(content, offset, link_start)?
        {
            children.push(link);
            *pos = end;
            return Ok(true);
        }

        *pos += 1;
        let text_start = *pos;
        let (close, nested) = self.scan_balanced_matched(content, *pos);
        *pos = close;

        if *pos < content.len() && bytes[*pos] == b']' {
            let close = *pos;
            let link_text = &content[text_start..close];

            // Bracket text with a bracket inside is the shape that used to
            // be parsed once per level: probed here, then parsed again by
            // the fallback below, at every enclosing bracket. Parse it in
            // place instead — the fallback's own result — and decide
            // afterwards. Text that holds anything but brackets is left to
            // the probe *before* any of it is parsed, so the two paths never
            // both run over the same bytes. `None` means the walk found a
            // construct that ends past the closing bracket after all, which
            // only its own parse can settle.
            if nested
                && self.cached_probe_verdict(link_text).is_none()
                && self.next_bracket_text_stop(content, text_start) >= close
                && let Some(made_link) = self.parse_nested_bracket(
                    content,
                    offset,
                    children,
                    markers,
                    (link_start, close),
                    pos,
                )?
            {
                return Ok(made_link);
            }

            // The probe needs the parsed children, and so does every
            // accepting branch below, so parse once and hand the nodes on.
            // The verdict is memoized because the literal-bracket fallback
            // makes the caller re-scan these same bytes; re-probing there
            // is what turns nested brackets into exponential work. The
            // balanced scan already reported whether an unescaped `[` sits
            // inside the text; without one the probe cannot find a link.
            let mut inner_nodes = None;
            let inner_has_link =
                nested && self.probe_link_text(link_text, offset + text_start, &mut inner_nodes)?;

            // Links may not contain other links; when the bracket text
            // parses to one, the outer bracket stays literal and the inner
            // (re-parsed after the fallback) wins.
            if !inner_has_link && let Some(resolved) = self.resolve_link(content, close, link_text)
            {
                let children_nodes = match inner_nodes.take() {
                    Some(nodes) => nodes,
                    None => self.parse_inline(link_text, offset + text_start)?,
                };
                children.push(Node::Link(self.allocator.boxed(Link {
                    url: resolved.url,
                    title: resolved.title,
                    children: children_nodes,
                    span: Span::new((offset + link_start) as u32, (offset + resolved.end) as u32),
                })));
                *pos = resolved.end;
                return Ok(true);
            }
        }

        // No valid inline link here: the bracket is literal text and the
        // rest of the bracketed run is re-parsed for other inline markup.
        Self::push_text(children, "[", offset + link_start, offset + link_start + 1);
        *pos = link_start + 1;
        Ok(false)
    }

    /// Where a closed bracket's destination comes from: the inline
    /// `(dest "title")` form, the full `[text][label]` or collapsed
    /// `[text][]` reference form, or the shortcut `[label]` form.
    ///
    /// Only reached for bracket text that holds no link, since a link may
    /// not contain a link.
    fn resolve_link(
        &self,
        content: &'a str,
        close: usize,
        link_text: &'a str,
    ) -> Option<ResolvedLink<'a>> {
        let bytes = content.as_bytes();

        // Inline form: [text](dest "title")
        if bytes.get(close + 1) == Some(&b'(')
            && let Some(target) = self.parse_link_target(content, close + 1)
        {
            return Some(ResolvedLink {
                url: target.url,
                title: target.title,
                end: target.end,
            });
        }

        // Full [text][label] and collapsed [text][] reference forms.
        let mut well_formed_reference = false;
        if self.options.allow_link_refs
            && bytes.get(close + 1) == Some(&b'[')
            && self.has_closer_from(content, close + 2, b']')
        {
            let label_start = close + 2;
            let (label_end, _) = self.scan_balanced_matched(content, label_start);
            if label_end < content.len() && bytes[label_end] == b']' {
                well_formed_reference = true;
                let raw_label = &content[label_start..label_end];
                let key = if raw_label.trim().is_empty() {
                    link_text
                } else {
                    raw_label
                };
                if let Some(reference) = self.lookup_reference(key) {
                    return Some(ResolvedLink {
                        url: reference.url,
                        title: reference.title,
                        end: label_end + 1,
                    });
                }
            }
        }

        // Shortcut form: [label]. Suppressed when an explicit (but
        // unknown) [label] followed, which must stay literal.
        if !well_formed_reference && let Some(reference) = self.lookup_reference(link_text) {
            return Some(ResolvedLink {
                url: reference.url,
                title: reference.title,
                end: close + 1,
            });
        }

        None
    }

    /// Parses bracket text that holds another bracket where it stands, and
    /// reports whether the nodes it appended hold a link — or `None` when it
    /// could not be parsed in place and the caller has to probe instead.
    ///
    /// The literal bracket is pushed first, speculatively: when the text
    /// does hold a link, the nodes are already exactly what the
    /// literal-bracket fallback would have produced and nothing is redone.
    /// When it does not and the bracket resolves to a link after all, the
    /// same nodes become that link's children.
    fn parse_nested_bracket(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        markers: &mut InlineMarkerScan,
        bracket: (usize, usize),
        pos: &mut usize,
    ) -> ParseResult<Option<bool>> {
        let (link_start, close) = bracket;
        let text_start = link_start + 1;
        let link_text = &content[text_start..close];
        let node_start = children.len();
        let nested_depth = self.nested_inline_depth();
        Self::push_text(children, "[", offset + link_start, offset + link_start + 1);

        let Some((resume, made_link)) =
            self.parse_bracket_text(content, offset, children, markers, (text_start, close))?
        else {
            children.truncate(node_start);
            self.restore_nested_inline_depth(nested_depth);
            return Ok(None);
        };
        // This walk answered what the probe would have been asked, so the
        // probe never has to ask it again — of this text, or of the same
        // bytes reached as a wiki-link label.
        self.remember_probe_verdict(link_text, made_link);

        if !made_link && let Some(resolved) = self.resolve_link(content, close, link_text) {
            // The trailing text run is held back for the caller to merge
            // with what follows the bracket; as link children it ends here.
            if resume < close {
                Self::push_text(
                    children,
                    &content[resume..close],
                    offset + resume,
                    offset + close,
                );
            }
            let mut inner_nodes = self
                .allocator
                .new_vec_with_capacity(children.len() - node_start - 1);
            inner_nodes.extend(children.drain(node_start + 1..));
            // Drops the speculative literal bracket.
            children.truncate(node_start);
            children.push(Node::Link(self.allocator.boxed(Link {
                url: resolved.url,
                title: resolved.title,
                children: inner_nodes,
                span: Span::new((offset + link_start) as u32, (offset + resolved.end) as u32),
            })));
            *pos = resolved.end;
            return Ok(Some(true));
        }

        *pos = resume;
        Ok(Some(made_link))
    }

    /// Parses `content[text_start..close]` in place: the same walk
    /// `parse_inline` runs, bounded to the region and restricted to the one
    /// marker that keeps it there.
    ///
    /// Returns the position where the region's trailing text run starts
    /// (deliberately not pushed, so the caller's own run scan merges it with
    /// what follows the bracket) and whether a link node was appended.
    ///
    /// The caller has already refused every region holding a marker other
    /// than `[` (`Parser::next_bracket_text_stop`): parsing a region in
    /// place is only the same as parsing it on its own while nothing inside
    /// it can reach past the closing bracket or pair with a delimiter
    /// outside it, and every other marker can — emphasis and the other
    /// delimiter runs pair across the bracket, a code span, an autolink,
    /// raw HTML, an image, an inline note or an MDX expression can all close
    /// after it, and a line ending folds into the run that follows.
    ///
    /// `None` is the one thing left that only the walk can find: a
    /// construct that ends past the closing bracket. The brackets inside
    /// the region are balanced, so a `[` in it closes in it, but its
    /// destination or its label can still read past the `]` — `[[a](u]x)]`
    /// is one.
    fn parse_bracket_text(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        markers: &mut InlineMarkerScan,
        region: (usize, usize),
    ) -> ParseResult<Option<(usize, bool)>> {
        let (text_start, close) = region;
        // One inline context per bracket level, counted where the probe used
        // to count it, so the nesting cap refuses exactly what it refused
        // before (`docs/decisions/2026-09-17-inline-nesting-cap.md`).
        let _depth = self.enter_inline(offset + text_start)?;
        let bytes = content.as_bytes();
        let mut made_link = false;
        let mut recorded = false;
        let mut pos = text_start;
        loop {
            let start = pos;
            let marker = markers.next(bytes, pos);
            if marker >= close {
                return Ok(Some((start, made_link)));
            }
            if bytes[marker] != b'[' {
                // `next_bracket_text_stop` refused every other marker before
                // this walk started; keep the walk honest if the two byte
                // sets ever drift apart.
                return Ok(None);
            }
            if !recorded {
                // A bracket inside this one: without this, the brackets
                // below would each walk their own text to find their `]`,
                // once per level. One walk from here decides all of them.
                // Not before the first one is found — a region that bails
                // out above must not pay for the map.
                recorded = true;
                self.record_bracket_matches(content, marker + 1);
            }
            if marker > start {
                Self::push_text(
                    children,
                    &content[start..marker],
                    offset + start,
                    offset + marker,
                );
            }
            pos = marker;
            made_link |= self.parse_link(content, offset, children, markers, &mut pos)?;
            if pos > close {
                return Ok(None);
            }
            if pos == close {
                return Ok(Some((pos, made_link)));
            }
        }
    }

    fn try_parse_wiki_link(
        &self,
        content: &'a str,
        offset: usize,
        link_start: usize,
    ) -> ParseResult<Option<(Node<'a>, usize)>> {
        if !self.has_wiki_closer_from(content, link_start + 2) {
            return Ok(None);
        }
        let Some(close) = Self::scan_wiki_link_close(content.as_bytes(), link_start + 2) else {
            return Ok(None);
        };
        let (inner, inner_offset) =
            trim_with_offset(&content[link_start + 2..close], link_start + 2);
        if inner.is_empty() {
            return Ok(None);
        }

        let (target_part, label_part, label_part_offset) =
            split_wiki_link_inner(inner, inner_offset);
        let (target, target_offset) = trim_with_offset(target_part, inner_offset);
        if target.is_empty() {
            return Ok(None);
        }

        let (label, label_offset) = if let Some(label_part) = label_part {
            let (label, label_offset) = trim_with_offset(label_part, label_part_offset);
            if label.is_empty() {
                (target, target_offset)
            } else {
                (label, label_offset)
            }
        } else {
            (target, target_offset)
        };

        let mut label_nodes = None;
        // A link label is short enough that the probe stays off the vector
        // path: nested-bracket candidates are rare, but every link pays it.
        if short_scan::find(b'[', label.as_bytes()).is_some()
            && self.probe_link_text(label, offset + label_offset, &mut label_nodes)?
        {
            return Ok(None);
        }

        let children = match label_nodes.take() {
            Some(nodes) => nodes,
            None => self.parse_inline(label, offset + label_offset)?,
        };
        Ok(Some((
            Node::Link(self.allocator.boxed(Link {
                url: target,
                title: None,
                children,
                span: Span::new((offset + link_start) as u32, (offset + close + 2) as u32),
            })),
            close + 2,
        )))
    }

    fn scan_wiki_link_close(bytes: &[u8], mut cursor: usize) -> Option<usize> {
        while cursor + 1 < bytes.len() {
            match bytes[cursor] {
                b'\\' => {
                    cursor += 2;
                }
                b'`' => {
                    cursor = Self::closed_code_span_end(bytes, cursor)
                        .unwrap_or_else(|| cursor.saturating_add(1));
                }
                b']' if bytes[cursor + 1] == b']' => return Some(cursor),
                _ => cursor += 1,
            }
        }
        None
    }

    /// The verdict a probe has already reached for these exact bytes, if
    /// any. Bracket text is parsed in place only when there is none, so a
    /// text the probe has judged keeps taking the path it took before.
    fn cached_probe_verdict(&self, link_text: &'a str) -> Option<bool> {
        let cache = self.link_probe_cache().borrow();
        if cache.is_empty() {
            return None;
        }
        cache
            .get(&(link_text.as_ptr() as usize, link_text.len()))
            .copied()
    }

    fn remember_probe_verdict(&self, link_text: &'a str, verdict: bool) {
        self.link_probe_cache()
            .borrow_mut()
            .insert((link_text.as_ptr() as usize, link_text.len()), verdict);
    }

    /// Reports whether `link_text` already parses to something containing a
    /// link, so the surrounding bracket cannot become one.
    ///
    /// On a cache miss the parsed nodes are handed back through `nodes`,
    /// because an accepting branch in `parse_link` needs exactly those
    /// children and would otherwise parse the same bytes a second time.
    fn probe_link_text(
        &self,
        link_text: &'a str,
        offset: usize,
        nodes: &mut Option<Vec<'a, Node<'a>>>,
    ) -> ParseResult<bool> {
        // Look the verdict up without holding the borrow: the parse below
        // re-enters this method.
        if let Some(verdict) = self.cached_probe_verdict(link_text) {
            return Ok(verdict);
        }
        let parsed = match self.parse_inline(link_text, offset) {
            Ok(parsed) => parsed,
            // A depth failure is a verdict on the document, not on this
            // bracket text: the literal-bracket fallback below would go on
            // to probe the next opener at the same depth, so the run has to
            // end here. Any other failing sub-parse cannot yield a link, and
            // the caller's own parse of the same text surfaces the error.
            Err(error) if matches!(error.kind(), ParseErrorKind::NestingTooDeep { .. }) => {
                return Err(error);
            }
            Err(_) => return Ok(false),
        };
        let verdict = contains_link(&parsed);
        self.remember_probe_verdict(link_text, verdict);
        *nodes = Some(parsed);
        Ok(verdict)
    }
}

/// Where a closed bracket resolves to, whichever of the three link forms
/// answered (see [`Parser::resolve_link`]).
struct ResolvedLink<'a> {
    url: &'a str,
    title: Option<&'a str>,
    /// Byte index in the inline content just past the link's last byte.
    end: usize,
}

fn split_wiki_link_inner(inner: &str, offset: usize) -> (&str, Option<&str>, usize) {
    if let Some((target, label)) = inner.split_once('|') {
        (target, Some(label), offset + target.len() + 1)
    } else {
        (inner, None, offset)
    }
}

fn trim_with_offset(value: &str, offset: usize) -> (&str, usize) {
    let trimmed_start = value.trim_start();
    let leading = value.len() - trimmed_start.len();
    (trimmed_start.trim_end(), offset + leading)
}

/// Does any node in the tree contain a link?
fn contains_link(nodes: &[Node<'_>]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Link(_) => true,
        Node::FootnoteDefinition(n) if n.label.is_none() => true,
        Node::Emphasis(n) => contains_link(&n.children),
        Node::Strong(n) => contains_link(&n.children),
        Node::Highlight(n) => contains_link(&n.children),
        Node::Delete(n) => contains_link(&n.children),
        Node::Superscript(n) => contains_link(&n.children),
        Node::Subscript(n) => contains_link(&n.children),
        _ => false,
    })
}
