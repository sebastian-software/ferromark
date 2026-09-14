//! Inline link destination and title parsing (CommonMark "Links").
//!
//! Handles the `(...)` part of `[text](dest "title")`: pointy-bracket and
//! bare destinations, the three title quoting styles, backslash escapes,
//! and the surrounding whitespace rules. Escaped destinations/titles are
//! unescaped into the arena so the AST keeps borrowing from parser memory.

use crate::parser::Parser;

pub(in crate::parser) struct LinkTarget<'a> {
    pub url: &'a str,
    pub title: Option<&'a str>,
    /// Byte index in the inline content just past the closing `)`.
    pub end: usize,
}

impl<'a> Parser<'a> {
    /// Parses a link target starting at the `(` at `open`. Returns `None`
    /// when the parenthesized run is not a valid destination/title pair,
    /// in which case the bracket text falls back to literal parsing.
    pub(in crate::parser) fn parse_link_target(
        &self,
        content: &'a str,
        open: usize,
    ) -> Option<LinkTarget<'a>> {
        let bytes = content.as_bytes();
        let mut i = skip_ws(bytes, open + 1);

        let (raw_url, after_dest) = parse_destination(content, i)?;
        i = skip_ws(bytes, after_dest);

        let mut title = None;
        // A title needs whitespace between it and the destination.
        if i > after_dest
            && let Some((raw_title, after_title)) = parse_title(content, i)
        {
            title = Some(self.unescape_link_component(raw_title));
            i = skip_ws(bytes, after_title);
        }

        if bytes.get(i) != Some(&b')') {
            return None;
        }
        Some(LinkTarget { url: self.unescape_link_component(raw_url), title, end: i + 1 })
    }

    /// Removes backslashes that escape ASCII punctuation and decodes
    /// entity/numeric character references (both apply inside link
    /// destinations and titles). Returns the input slice untouched when
    /// nothing decodes; otherwise the copy is allocated in the arena.
    pub(in crate::parser) fn unescape_link_component(&self, raw: &'a str) -> &'a str {
        let bytes = raw.as_bytes();
        // Probe once so unchanged components can stay borrowed without a byte
        // walk. Keep the scalar tail: dense escapes must not pay for repeated
        // vector searches. memchr provides portable and short/tail fallbacks.
        let Some(mut i) = memchr::memchr2(b'\\', b'&', bytes) else {
            return raw;
        };
        let mut start = 0;
        let mut out: Option<ferromark_allocator::String<'a>> = None;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' if i + 1 < bytes.len() && bytes[i + 1].is_ascii_punctuation() => {
                    let out = out.get_or_insert_with(|| self.allocator.new_string());
                    out.push_str(&raw[start..i]);
                    start = i + 1;
                    i += 2;
                }
                b'&' => {
                    if let Some((value, len)) = super::entity::scan_entity(&raw[i..]) {
                        let out = out.get_or_insert_with(|| self.allocator.new_string());
                        out.push_str(&raw[start..i]);
                        match value {
                            super::entity::EntityValue::Named(expansion) => {
                                out.push_str(expansion);
                            }
                            super::entity::EntityValue::Char(ch) => out.push(ch),
                        }
                        i += len;
                        start = i;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }
        match out {
            Some(mut out) => {
                out.push_str(&raw[start..]);
                out.into_bump_str()
            }
            None => raw,
        }
    }
}

/// Parses a destination at `i`: either `<...>` (may contain spaces, no
/// newlines or unescaped angle brackets) or a bare run without whitespace
/// or control characters and with balanced unescaped parentheses.
pub(in crate::parser) fn parse_destination(content: &str, i: usize) -> Option<(&str, usize)> {
    let bytes = content.as_bytes();
    if bytes.get(i) == Some(&b'<') {
        let mut j = i + 1;
        loop {
            match bytes.get(j)? {
                b'\\' if is_escape(bytes, j) => j += 2,
                b'>' => return Some((&content[i + 1..j], j + 1)),
                b'<' | b'\n' | b'\r' => return None,
                _ => j += 1,
            }
        }
    }

    // Bare destinations are mostly plain URL bytes; only whitespace,
    // control bytes, backslashes, and parentheses need a decision. Jump
    // between those with the classifier below instead of matching every
    // byte: link-dense documents (encyclopedia articles) spend a fifth of
    // their parse time in this loop otherwise.
    let mut depth = 0usize;
    let mut j = i;
    loop {
        j = next_destination_stop(bytes, j);
        let Some(&byte) = bytes.get(j) else {
            break;
        };
        match byte {
            b'\\' if is_escape(bytes, j) => j += 2,
            b'\\' => j += 1,
            b'(' => {
                depth += 1;
                j += 1;
            }
            b')' if depth == 0 => break,
            b')' => {
                depth -= 1;
                j += 1;
            }
            // Whitespace or a control byte (including DEL) ends the run.
            _ => break,
        }
    }
    if depth > 0 {
        return None;
    }
    Some((&content[i..j], j))
}

/// Flag table for the bare-destination scan: `DESTINATION_STOP[b] != 0` iff
/// `b` is ASCII whitespace or control (every byte below `0x21`, plus DEL),
/// a backslash, or a parenthesis. Bytes at or above `0x80` never stop the
/// scan. This is the definition the vector path must agree with.
static DESTINATION_STOP: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut b = 0;
    while b < 0x21 {
        t[b] = 1;
        b += 1;
    }
    t[0x7F] = 1;
    t[b'\\' as usize] = 1;
    t[b'(' as usize] = 1;
    t[b')' as usize] = 1;
    t
};

/// Nibble-pair tables for the vector path. A byte stops iff
/// `LOW[b & 0x0F] & HIGH[b >> 4]` is nonzero: rows `0x0_`/`0x1_` accept every
/// low nibble (bit 0), row `0x2_` accepts space and both parentheses
/// (bit 1), row `0x5_` the backslash (bit 2), and row `0x7_` DEL (bit 3).
/// Every other high nibble, including all bytes >= 0x80, maps to zero.
#[cfg(target_arch = "aarch64")]
const DESTINATION_LOW_NIBBLE: [u8; 16] = [
    0x03, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x03, 0x03, 0x01, 0x01, 0x05, 0x01, 0x01, 0x09,
];
#[cfg(target_arch = "aarch64")]
const DESTINATION_HIGH_NIBBLE: [u8; 16] =
    [0x01, 0x01, 0x02, 0, 0, 0x04, 0, 0x08, 0, 0, 0, 0, 0, 0, 0, 0];

/// Offset of the first stop byte at or after `from`, or `bytes.len()`.
#[inline]
fn next_destination_stop(bytes: &[u8], from: usize) -> usize {
    #[cfg(target_arch = "aarch64")]
    {
        // One vector is the breakeven; shorter remainders (most short
        // relative links) stay on the table walk.
        if bytes.len() - from >= 16 {
            return next_destination_stop_neon(bytes, from);
        }
    }
    next_destination_stop_scalar(bytes, from)
}

#[inline]
fn next_destination_stop_scalar(bytes: &[u8], from: usize) -> usize {
    let mut j = from;
    while j < bytes.len() && DESTINATION_STOP[bytes[j] as usize] == 0 {
        j += 1;
    }
    j
}

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn next_destination_stop_neon(bytes: &[u8], from: usize) -> usize {
    use std::arch::aarch64::*;
    let end = bytes.len();
    let mut i = from;
    // SAFETY: every 16-byte load below is bounded by an explicit
    // `i + 16 <= end` check or reads the final full vector at `end - 16`.
    unsafe {
        let low = vld1q_u8(DESTINATION_LOW_NIBBLE.as_ptr());
        let high = vld1q_u8(DESTINATION_HIGH_NIBBLE.as_ptr());
        let nibble = vdupq_n_u8(0x0F);
        let classify = |v: uint8x16_t| {
            let lo = vqtbl1q_u8(low, vandq_u8(v, nibble));
            let hi = vqtbl1q_u8(high, vshrq_n_u8(v, 4));
            let m = vtstq_u8(lo, hi);
            vget_lane_u64(vreinterpret_u64_u8(vshrn_n_u16(vreinterpretq_u16_u8(m), 4)), 0)
        };
        while i + 16 <= end {
            let mask = classify(vld1q_u8(bytes.as_ptr().add(i)));
            if mask != 0 {
                return i + (mask.trailing_zeros() / 4) as usize;
            }
            i += 16;
        }
        if i < end {
            // Overlapping tail: re-read the last vector and drop the lanes
            // the loop already cleared. `vtstq_u8` is exact per lane.
            let base = end - 16;
            let mask =
                classify(vld1q_u8(bytes.as_ptr().add(base))) & (u64::MAX << ((i - base) * 4));
            if mask != 0 {
                return base + (mask.trailing_zeros() / 4) as usize;
            }
        }
    }
    end
}

/// Parses a `"..."`, `'...'`, or `(...)` title starting at `i`.
pub(in crate::parser) fn parse_title(content: &str, i: usize) -> Option<(&str, usize)> {
    let bytes = content.as_bytes();
    let (closer, nested_open) = match bytes.get(i)? {
        b'"' => (b'"', None),
        b'\'' => (b'\'', None),
        b'(' => (b')', Some(b'(')),
        _ => return None,
    };

    let mut j = i + 1;
    loop {
        match bytes.get(j)? {
            b'\\' if is_escape(bytes, j) => j += 2,
            byte if *byte == closer => return Some((&content[i + 1..j], j + 1)),
            byte if nested_open == Some(*byte) => return None,
            _ => j += 1,
        }
    }
}

fn is_escape(bytes: &[u8], i: usize) -> bool {
    i + 1 < bytes.len() && bytes[i + 1].is_ascii_punctuation()
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while matches!(bytes.get(i), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    // Owned strings keep the test oracle independent of production arena storage.
    #![allow(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

    use std::borrow::Cow;

    use ferromark_allocator::Allocator;

    use super::Parser;
    use crate::parser::inline::entity::{EntityValue, scan_entity};

    // Keep the original byte-at-a-time walk as an independent scan oracle.
    // Entity decoding itself is unchanged by the scanner optimization.
    fn scalar_unescape(raw: &str) -> Cow<'_, str> {
        let bytes = raw.as_bytes();
        let (mut i, mut start) = (0, 0);
        let mut out: Option<String> = None;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' if i + 1 < bytes.len() && bytes[i + 1].is_ascii_punctuation() => {
                    out.get_or_insert_with(String::new).push_str(&raw[start..i]);
                    start = i + 1;
                    i += 2;
                }
                b'&' => {
                    if let Some((value, len)) = scan_entity(&raw[i..]) {
                        let out = out.get_or_insert_with(String::new);
                        out.push_str(&raw[start..i]);
                        match value {
                            EntityValue::Named(text) => out.push_str(text),
                            EntityValue::Char(ch) => out.push(ch),
                        }
                        i += len;
                        start = i;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }
        out.map_or(Cow::Borrowed(raw), |mut out| {
            out.push_str(&raw[start..]);
            Cow::Owned(out)
        })
    }

    // The original byte-at-a-time destination walk, kept as the oracle for
    // the stop-byte classifier.
    fn scalar_destination(content: &str, i: usize) -> Option<(&str, usize)> {
        let bytes = content.as_bytes();
        if bytes.get(i) == Some(&b'<') {
            return super::parse_destination(content, i);
        }
        let mut depth = 0usize;
        let mut j = i;
        while j < bytes.len() {
            match bytes[j] {
                b'\\' if super::is_escape(bytes, j) => j += 2,
                b'(' => {
                    depth += 1;
                    j += 1;
                }
                b')' if depth == 0 => break,
                b')' => {
                    depth -= 1;
                    j += 1;
                }
                byte if byte.is_ascii_whitespace() || byte.is_ascii_control() => break,
                _ => j += 1,
            }
        }
        if depth > 0 {
            return None;
        }
        Some((&content[i..j], j))
    }

    fn check_destination(content: &str, i: usize) {
        assert_eq!(
            super::parse_destination(content, i),
            scalar_destination(content, i),
            "input: {content:?} at {i}"
        );
    }

    #[test]
    fn destination_stop_table_matches_definition() {
        for byte in 0..=255u8 {
            let expected = byte.is_ascii_whitespace()
                || byte.is_ascii_control()
                || matches!(byte, b'\\' | b'(' | b')');
            assert_eq!(super::DESTINATION_STOP[byte as usize] != 0, expected, "byte {byte:#x}");
        }
    }

    #[test]
    fn destination_scan_matches_scalar_at_byte_boundaries_and_tails() {
        let mut needles: Vec<String> = (0..=0x7Fu8).map(|b| char::from(b).to_string()).collect();
        needles.extend(
            [
                "\\(", "\\)", "\\a", "\\", "(a)", "((", "))", "(", ")", "é", "中", "🙂", "\u{7f}",
                "\u{80}",
            ]
            .map(str::to_owned),
        );
        for offset in 0..=40 {
            for tail in [0, 1, 7, 8, 15, 16, 17, 31, 32, 33, 64] {
                for needle in &needles {
                    let input = format!("{}{}{}", "u".repeat(offset), needle, "v".repeat(tail));
                    check_destination(&input, 0);
                    if offset > 0 {
                        check_destination(&input, 1);
                    }
                }
            }
        }
    }

    #[test]
    fn destination_scan_matches_scalar_on_mixed_inputs() {
        let tokens = [
            "a",
            "/",
            "é",
            "中",
            "🙂",
            "\\",
            "\\)",
            "\\(",
            "(",
            ")",
            " ",
            "\t",
            "\n",
            "\u{1}",
            "\u{7f}",
            "<",
            ">",
            "&amp;",
            "http://x.y/",
        ];
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        for _ in 0..3000 {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let len = (state >> 33) as usize % 96;
            let mut input = String::new();
            for _ in 0..len {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                input.push_str(tokens[(state >> 33) as usize % tokens.len()]);
            }
            check_destination(&input, 0);
            let mid = input.len() / 2;
            if input.is_char_boundary(mid) {
                check_destination(&input, mid);
            }
        }
        check_destination(&"(x)".repeat(64), 0);
        check_destination(&format!("{}({}", "a".repeat(40), "b".repeat(40)), 0);
        check_destination(&format!("{})", "a".repeat(100)), 0);
    }

    fn check(raw: &str) {
        let allocator = Allocator::new();
        let parser = Parser::new(&allocator, "");
        let expected = scalar_unescape(raw);
        let actual = parser.unescape_link_component(raw);
        assert_eq!(actual, expected.as_ref(), "input: {raw:?}");
        if matches!(expected, Cow::Borrowed(_)) {
            assert_eq!(actual.as_ptr(), raw.as_ptr(), "unchanged input must stay borrowed");
        }
    }

    #[test]
    fn component_scan_matches_scalar_at_byte_boundaries_and_tails() {
        let needles = [
            "",
            "\\",
            "&",
            "\\!",
            "\\a",
            "\\é",
            "\\\\&amp;",
            "\\&amp;",
            "&amp;",
            "&NotEqualTilde;",
            "&#0;",
            "&#x1F642;",
            "&#xD800;",
            "&#1114112;",
            "&unknown;",
            "&#99999999;",
            "&amp",
            "é中🙂",
            "&&amp;\\)",
        ];
        for offset in 0..=65 {
            for tail in [0, 1, 7, 8, 15, 16, 31, 32, 63, 64, 255] {
                for needle in needles {
                    let input = format!("{}{}{}", "a".repeat(offset), needle, "z".repeat(tail));
                    check(&input);
                }
            }
        }
        for value in 0..=127u8 {
            check(&format!("prefix\\{}&amp;suffix", char::from(value)));
        }
    }

    #[test]
    fn component_scan_matches_scalar_on_mixed_unicode_and_dense_candidates() {
        let tokens =
            ["a", "é", "中", "🙂", "\\", "&", "&amp;", "&#x41;", "\\!", "\\a", "&bad;", "()"];
        let mut state = 0x46e2_94c3_198a_7bf5u64;
        for _ in 0..2000 {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let len = (state >> 32) as usize % 192;
            let mut input = String::new();
            for _ in 0..len {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                input.push_str(tokens[(state >> 32) as usize % tokens.len()]);
            }
            check(&input);
        }
        check(&"\\&amp;&&#x41;\\aé\\!".repeat(256));
        check(&format!("{}&amp;{}", "ü".repeat(4096), "中".repeat(4096)));
    }
}
