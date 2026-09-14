use ferromark_allocator::Vec;
use ferromark_ast::{Image, Node, Span, Text};
use memchr::{memchr, memchr3};

use super::Parser;
use super::byte_class::ByteClass;
use crate::error::ParseResult;

impl<'a> Parser<'a> {
    pub(super) fn parse_image(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let bytes = content.as_bytes();
        if *pos + 1 >= content.len() || bytes[*pos + 1] != b'[' {
            Self::push_text(children, "!", offset + *pos, offset + *pos + 1);
            *pos += 1;
            return Ok(());
        }

        let image_start = *pos;

        // Nothing can close this bracket, so skip the balanced scan that
        // would walk to the end of the content to reach the same verdict.
        // Same fallback as below, reached without the walk.
        if !self.has_closer_from(content, *pos + 2, b']') {
            Self::push_text(children, "![", offset + image_start, offset + image_start + 2);
            *pos = image_start + 2;
            return Ok(());
        }

        *pos += 2;
        let alt_start = *pos;
        *pos = Self::scan_balanced(content, *pos).0;

        if *pos < content.len() && bytes[*pos] == b']' {
            let close = *pos;
            let raw_alt = &content[alt_start..close];
            let alt = self.flatten_image_alt(raw_alt, offset + alt_start)?;

            if bytes.get(close + 1) == Some(&b'(')
                && let Some(target) = self.parse_link_target(content, close + 1)
            {
                children.push(Node::Image(self.allocator.boxed(Image {
                    url: target.url,
                    alt,
                    title: target.title,
                    span: Span::new((offset + image_start) as u32, (offset + target.end) as u32),
                })));
                *pos = target.end;
                return Ok(());
            }

            let mut well_formed_reference = false;
            if bytes.get(close + 1) == Some(&b'[') && self.has_closer_from(content, close + 2, b']')
            {
                let label_start = close + 2;
                let (label_end, _) = Self::scan_balanced(content, label_start);
                if label_end < content.len() && bytes[label_end] == b']' {
                    well_formed_reference = true;
                    let raw_label = &content[label_start..label_end];
                    let key = if raw_label.trim().is_empty() { raw_alt } else { raw_label };
                    if let Some(reference) = self.lookup_reference(key) {
                        children.push(Node::Image(self.allocator.boxed(Image {
                            url: reference.url,
                            alt,
                            title: reference.title,
                            span: Span::new(
                                (offset + image_start) as u32,
                                (offset + label_end + 1) as u32,
                            ),
                        })));
                        *pos = label_end + 1;
                        return Ok(());
                    }
                }
            }

            if !well_formed_reference && let Some(reference) = self.lookup_reference(raw_alt) {
                children.push(Node::Image(self.allocator.boxed(Image {
                    url: reference.url,
                    alt,
                    title: reference.title,
                    span: Span::new((offset + image_start) as u32, (offset + close + 1) as u32),
                })));
                *pos = close + 1;
                return Ok(());
            }
        }

        // No valid inline image here: `![` is literal text and the rest of
        // the bracketed run is re-parsed for other inline markup.
        Self::push_text(children, "![", offset + image_start, offset + image_start + 2);
        *pos = image_start + 2;
        Ok(())
    }

    /// Builds an image's `alt` attribute: the bracket text parsed as
    /// inlines and flattened to plain text (links contribute their text,
    /// code its literal content). Plain text stays zero-copy.
    fn flatten_image_alt(&self, raw: &'a str, offset: usize) -> ParseResult<&'a str> {
        if memchr3(b'[', b'*', b'_', raw.as_bytes()).is_none()
            && memchr3(b'`', b'\\', b'&', raw.as_bytes()).is_none()
            && memchr(b'<', raw.as_bytes()).is_none()
        {
            return Ok(raw);
        }
        let nodes = self.parse_inline(raw, offset)?;
        let mut out = self.allocator.new_string();
        flatten_inline_text(&nodes, &mut out);
        Ok(out.into_bump_str())
    }

    /// Child slots to reserve for `content_len` bytes of inline content.
    ///
    /// A bump-allocated `Vec` cannot extend the block it owns — bumpalo
    /// hands back a fresh region and memcpies — so growing copies every
    /// node so far at each doubling step and abandons the old block in the
    /// arena. Measured over the bundled corpora the node count tracks the
    /// content length closely (p90 ≈ one node per 20 bytes in every length
    /// bucket), so reserving that covers most blocks in one allocation and
    /// still uses ~3% *less* arena than growing did. The floor keeps short
    /// spans at bumpalo's own minimum; the ceiling stops a long paragraph
    /// from reserving a kilobyte it will not fill.
    pub(super) fn inline_children_capacity(content_len: usize) -> usize {
        const BYTES_PER_NODE: usize = 20;
        (content_len / BYTES_PER_NODE).clamp(4, 12)
    }

    pub(super) fn push_text(
        children: &mut Vec<'a, Node<'a>>,
        value: &'a str,
        start: usize,
        end: usize,
    ) {
        children.push(Node::Text(Text { value, span: Span::new(start as u32, end as u32) }));
    }

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
    /// costs O(n²). The position of the last closer settles it for every
    /// opener in the slice at once, so the run costs one scan in total.
    pub(super) fn has_closer_from(&self, content: &'a str, from: usize, closer: u8) -> bool {
        let key = (content.as_ptr() as usize, content.len(), closer);

        let cached = self.last_closer.borrow().get(&key).copied();
        let last = if let Some(last) = cached {
            last
        } else {
            let last = memchr::memrchr(closer, content.as_bytes());
            self.last_closer.borrow_mut().insert(key, last);
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
    pub(super) fn scan_balanced(content: &str, mut cursor: usize) -> (usize, bool) {
        let bytes = content.as_bytes();
        let mut depth = 1;
        let mut nested = false;
        loop {
            cursor = BRACKET_STOP.first_in(bytes, cursor);
            let Some(&byte) = bytes.get(cursor) else {
                break;
            };
            match byte {
                b'\\' => {
                    // An escaped ASCII punctuation byte (which covers both
                    // delimiters) is inert for bracket matching.
                    let escapes_next =
                        cursor + 1 < bytes.len() && bytes[cursor + 1].is_ascii_punctuation();
                    cursor += if escapes_next { 2 } else { 1 };
                }
                b'`' => {
                    let run = Self::marker_run_len(bytes, cursor, b'`');
                    cursor += run;
                    let mut scan = cursor;
                    while scan < bytes.len() {
                        let Some(off) = memchr(b'`', &bytes[scan..]) else {
                            break;
                        };
                        scan += off;
                        let closer = Self::marker_run_len(bytes, scan, b'`');
                        if closer == run {
                            cursor = scan + closer;
                            break;
                        }
                        scan += closer;
                    }
                }
                b'<' => {
                    if let Some(end) = super::inline::autolink_end(content, cursor) {
                        cursor = end;
                    } else if let Some((_, end)) = Parser::parse_inline_html(content, cursor, 0) {
                        cursor = end;
                    } else {
                        cursor += 1;
                    }
                }
                b'[' => {
                    depth += 1;
                    nested = true;
                    cursor += 1;
                }
                b']' => {
                    depth -= 1;
                    // Stop AT the closing delimiter.
                    if depth == 0 {
                        return (cursor, nested);
                    }
                    cursor += 1;
                }
                _ => cursor += 1,
            }
        }
        (cursor, nested)
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

/// Flattens inline nodes to their plain-text content (image `alt` rules).
fn flatten_inline_text(nodes: &[Node<'_>], out: &mut ferromark_allocator::String<'_>) {
    for node in nodes {
        match node {
            Node::Text(n) => out.push_str(n.value),
            Node::InlineCode(n) => out.push_str(n.value),
            Node::Emphasis(n) => flatten_inline_text(&n.children, out),
            Node::Strong(n) => flatten_inline_text(&n.children, out),
            Node::Delete(n) => flatten_inline_text(&n.children, out),
            Node::Superscript(n) => flatten_inline_text(&n.children, out),
            Node::Subscript(n) => flatten_inline_text(&n.children, out),
            Node::Link(n) => flatten_inline_text(&n.children, out),
            Node::Image(n) => out.push_str(n.alt),
            Node::Break(_) => out.push('\n'),
            _ => {}
        }
    }
}

#[cfg(test)]
mod scan_balanced_tests {
    // Owned strings keep the test oracle independent of production arena storage.
    #![allow(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

    use super::Parser;

    /// The original byte-at-a-time walk, kept as the oracle. The nested flag
    /// is derived from the same walk: it is set when an unescaped `[` is seen.
    fn scalar_scan_balanced(content: &str, mut cursor: usize) -> (usize, bool) {
        let bytes = content.as_bytes();
        let mut depth = 1;
        let mut nested = false;
        while cursor < bytes.len() {
            match bytes[cursor] {
                b'\\' => {
                    let escapes_next =
                        cursor + 1 < bytes.len() && bytes[cursor + 1].is_ascii_punctuation();
                    cursor += if escapes_next { 2 } else { 1 };
                }
                b'`' => {
                    let run = Parser::marker_run_len(bytes, cursor, b'`');
                    cursor += run;
                    let mut scan = cursor;
                    while scan < bytes.len() {
                        let Some(off) = memchr::memchr(b'`', &bytes[scan..]) else {
                            break;
                        };
                        scan += off;
                        let closer = Parser::marker_run_len(bytes, scan, b'`');
                        if closer == run {
                            cursor = scan + closer;
                            break;
                        }
                        scan += closer;
                    }
                }
                b'<' => {
                    if let Some(end) = super::super::inline::autolink_end(content, cursor) {
                        cursor = end;
                    } else if let Some((_, end)) = Parser::parse_inline_html(content, cursor, 0) {
                        cursor = end;
                    } else {
                        cursor += 1;
                    }
                }
                b'[' => {
                    depth += 1;
                    nested = true;
                    cursor += 1;
                }
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        return (cursor, nested);
                    }
                    cursor += 1;
                }
                _ => cursor += 1,
            }
        }
        (cursor, nested)
    }

    fn check(content: &str, from: usize) {
        assert_eq!(
            Parser::scan_balanced(content, from),
            scalar_scan_balanced(content, from),
            "input: {content:?} from {from}"
        );
    }

    #[test]
    fn bracket_stop_matches_definition() {
        for byte in 0..=255u8 {
            let expected = matches!(byte, b'\\' | b'`' | b'<' | b'[' | b']');
            assert_eq!(super::BRACKET_STOP.contains(byte), expected, "byte {byte:#x}");
        }
    }

    #[test]
    fn scan_matches_scalar_at_byte_boundaries_and_tails() {
        let mut needles: Vec<String> = (0..=0x7Fu8).map(|b| char::from(b).to_string()).collect();
        needles.extend(
            [
                "\\]",
                "\\[",
                "\\a",
                "``",
                "`x`",
                "`",
                "<a>",
                "<http://x.y>",
                "<",
                "[[",
                "]]",
                "[]",
                "é",
                "中",
            ]
            .map(str::to_owned),
        );
        for offset in 0..=36 {
            for tail in [0, 1, 7, 8, 15, 16, 17, 31, 32, 33] {
                for needle in &needles {
                    let input = format!("{}{}{}]", "t".repeat(offset), needle, "u".repeat(tail));
                    check(&input, 0);
                    let unclosed = format!("{}{}{}", "t".repeat(offset), needle, "u".repeat(tail));
                    check(&unclosed, 0);
                }
            }
        }
    }

    #[test]
    fn scan_matches_scalar_on_mixed_inputs() {
        let tokens = [
            "a",
            " ",
            "é",
            "中",
            "🙂",
            "\\",
            "\\]",
            "\\[",
            "[",
            "]",
            "`",
            "``",
            "`code`",
            "<",
            ">",
            "<b>",
            "<https://e.x/>",
            "\n",
            "(",
            ")",
        ];
        let mut state = 0x2545_f491_4f6c_dd1du64;
        for _ in 0..3000 {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let len = (state >> 33) as usize % 80;
            let mut input = String::new();
            for _ in 0..len {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                input.push_str(tokens[(state >> 33) as usize % tokens.len()]);
            }
            check(&input, 0);
            let mid = input.len() / 2;
            if input.is_char_boundary(mid) {
                check(&input, mid);
            }
        }
        check(&"[x]".repeat(64), 0);
        check(&format!("{}]", "a".repeat(100)), 0);
        check(&format!("{}`{}`{}]", "a".repeat(20), "]".repeat(20), "b".repeat(20)), 0);
    }
}
