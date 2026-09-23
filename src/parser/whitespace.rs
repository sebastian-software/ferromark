//! ASCII whitespace trimming for block structure.
//!
//! CommonMark forms a paragraph's or heading's raw content by "removing
//! initial and final spaces or tabs", calls a line holding only spaces or
//! tabs blank, and GFM trims the spaces between a table cell's pipes and its
//! content. None of these rules treat non-ASCII whitespace as structure: a
//! no-break space (`U+00A0`) at the start of a paragraph is content, and cmark
//! keeps it. The `str` methods `trim`, `trim_start` and `trim_end` strip every
//! Unicode `White_Space` character instead, and decode UTF-8 to find them.
//!
//! These helpers work on bytes and strip the ASCII members of `White_Space`:
//! space, tab, line feed, vertical tab, form feed and carriage return. Line
//! feed and carriage return are line endings a slice may still carry.
//! Vertical tab and form feed are wider than the specification's "spaces or
//! tabs", but the `str` methods stripped them too, so keeping them limits the
//! difference to non-ASCII whitespace. Every stripped byte is ASCII, so each
//! cut lands on a character boundary.
//!
//! Syntax that the specification defines through Unicode whitespace, such as
//! emphasis flanking, does not use these helpers. `clippy.toml` disallows the
//! `str` methods inside the parser so a new call site has to choose.

/// Whether `byte` is one of the whitespace bytes these helpers strip.
#[inline]
pub(in crate::parser) const fn is_trimmed_byte(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
}

/// `value` without its leading ASCII whitespace.
#[inline]
pub(in crate::parser) fn trim_start(value: &str) -> &str {
    let bytes = value.as_bytes();
    let mut start = 0;
    while start < bytes.len() && is_trimmed_byte(bytes[start]) {
        start += 1;
    }
    &value[start..]
}

/// `value` without its trailing ASCII whitespace.
#[inline]
pub(in crate::parser) fn trim_end(value: &str) -> &str {
    let bytes = value.as_bytes();
    let mut end = bytes.len();
    while end > 0 && is_trimmed_byte(bytes[end - 1]) {
        end -= 1;
    }
    &value[..end]
}

/// `value` without its leading and trailing ASCII whitespace.
#[inline]
pub(in crate::parser) fn trim(value: &str) -> &str {
    trim_end(trim_start(value))
}

/// [`trim`] together with the byte length of the leading whitespace it
/// removed, so a caller can keep the source offset of the trimmed content.
#[inline]
pub(in crate::parser) fn trim_with_leading(value: &str) -> (&str, usize) {
    let start = trim_start(value);
    (trim_end(start), value.len() - start.len())
}

/// Whether `value` is empty or holds only ASCII whitespace.
#[inline]
pub(in crate::parser) fn is_blank(value: &str) -> bool {
    value.bytes().all(is_trimmed_byte)
}

#[cfg(test)]
mod tests {
    // The `str` methods are the oracle for the ASCII subset, and owned
    // strings keep the test inputs independent of the parser arena.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use super::{is_blank, is_trimmed_byte, trim, trim_end, trim_start, trim_with_leading};

    #[test]
    fn strips_exactly_the_ascii_members_of_white_space() {
        for byte in 0..=0x7F_u8 {
            assert_eq!(
                is_trimmed_byte(byte),
                char::from(byte).is_whitespace(),
                "byte {byte:#04x}"
            );
        }
        for byte in 0x80..=0xFF_u8 {
            assert!(!is_trimmed_byte(byte), "byte {byte:#04x}");
        }
    }

    #[test]
    fn keeps_non_ascii_whitespace_as_content() {
        let non_ascii = (0x80..=0x3000_u32)
            .filter_map(char::from_u32)
            .filter(|ch| ch.is_whitespace());
        for ch in non_ascii {
            let value = format!(" \t{ch}a{ch}\r\n");
            let inner = format!("{ch}a{ch}");
            assert_eq!(trim(&value), inner, "U+{:04X}", u32::from(ch));
            assert_eq!(trim_with_leading(&value), (inner.as_str(), 2));
            assert!(!is_blank(&ch.to_string()), "U+{:04X}", u32::from(ch));
        }
    }

    #[test]
    fn matches_the_str_methods_on_ascii_input() {
        let alphabet = [" ", "\t", "\n", "\u{b}", "\u{c}", "\r", "a", "|"];
        let mut value = String::new();
        for first in alphabet {
            for second in alphabet {
                for third in alphabet {
                    value.clear();
                    value.push_str(first);
                    value.push_str(second);
                    value.push_str(third);
                    assert_eq!(trim(&value), value.trim());
                    assert_eq!(trim_start(&value), value.trim_start());
                    assert_eq!(trim_end(&value), value.trim_end());
                    assert_eq!(is_blank(&value), value.trim().is_empty());
                }
            }
        }
        assert_eq!(trim(""), "");
        assert!(is_blank(""));
    }
}
