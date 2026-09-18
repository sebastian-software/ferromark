//! Where the next byte that can start an inline construct is.
//!
//! Split out of `inline.rs` so the walk itself stays readable: this file is
//! one cursor and the rule for when it has to be recomputed.

use super::scan::{
    INLINE_MARKER_HIGHLIGHT, INLINE_MARKER_MATH, INLINE_MARKER_MDX, INLINE_MARKER_SUPERSCRIPT,
    next_inline_marker,
};

/// A memo for one forward byte scan over a fixed slice.
///
/// The scan is position-independent: when the next hit at or
/// after `origin` is `hit`, it is still `hit` for every position in
/// `origin..=hit`. Recomputing only when the cursor leaves that window is
/// what keeps the walk linear.
#[derive(Clone, Copy)]
pub(super) struct ForwardScan {
    origin: usize,
    hit: usize,
}

impl ForwardScan {
    pub(super) const fn new() -> Self {
        // `usize::MAX` cannot be a real origin, so the first lookup always
        // computes.
        Self {
            origin: usize::MAX,
            hit: 0,
        }
    }

    pub(super) fn hit(&mut self, from: usize, find: impl FnOnce(usize) -> usize) -> usize {
        if from < self.origin || from > self.hit {
            self.origin = from;
            self.hit = find(from);
        }
        self.hit
    }
}

/// Where the next byte that can start an inline construct is.
///
/// The core classifier and enabled extension markers are selected in one
/// forward lookup. Keeping one memo instead of one memo per optional marker
/// avoids repeated traversals when several extensions are enabled.
pub(in crate::parser) struct InlineMarkerScan {
    optional: u8,
    notes_only: bool,
    scan: ForwardScan,
}

impl InlineMarkerScan {
    pub(in crate::parser) const fn new(options: &crate::ParserOptions) -> Self {
        let mut optional = 0;
        if options.mdx {
            optional |= INLINE_MARKER_MDX;
        }
        if options.superscript || options.inline_footnotes {
            optional |= INLINE_MARKER_SUPERSCRIPT;
        }
        if options.math {
            optional |= INLINE_MARKER_MATH;
        }
        if options.highlight {
            optional |= INLINE_MARKER_HIGHLIGHT;
        }
        Self {
            optional,
            notes_only: options.inline_footnotes && !options.superscript,
            scan: ForwardScan::new(),
        }
    }

    pub(in crate::parser) fn next(&mut self, bytes: &[u8], from: usize) -> usize {
        let optional = self.optional;
        let notes_only = self.notes_only;
        self.scan.hit(from, |mut at| {
            loop {
                at = next_inline_marker(bytes, at, optional);
                if optional == 0 || at == bytes.len() {
                    return at;
                }
                // An isolated equals sign or a caret that cannot start a note
                // stays in the current text run: no extra AST nodes or dispatch.
                if bytes[at] == b'='
                    && bytes.get(at + 1) == Some(&b'=')
                    && (at == 0 || bytes[at - 1].is_ascii_whitespace())
                    && bytes.get(at + 2).is_none_or(u8::is_ascii_whitespace)
                {
                    // A pair surrounded by whitespace can neither open nor
                    // close. Preserve it in the text run without a delimiter record.
                    at += 2;
                    continue;
                }
                if (bytes[at] == b'=' && bytes.get(at + 1) != Some(&b'='))
                    || (notes_only && bytes[at] == b'^' && bytes.get(at + 1) != Some(&b'['))
                {
                    at += 1;
                } else {
                    return at;
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_marker(byte: u8, options: u8) -> bool {
        matches!(
            byte,
            b'*' | b'_' | b'`' | b'[' | b'!' | b'~' | b'\\' | b'<' | b'&' | b'\n' | b'\r'
        ) || (options & INLINE_MARKER_MDX != 0 && byte == b'{')
            || (options & INLINE_MARKER_SUPERSCRIPT != 0 && byte == b'^')
            || (options & INLINE_MARKER_MATH != 0 && byte == b'$')
            || (options & INLINE_MARKER_HIGHLIGHT != 0 && byte == b'=')
    }

    fn oracle(bytes: &[u8], from: usize, options: u8) -> usize {
        let mut at = from;
        while at < bytes.len()
            && (!is_marker(bytes[at], options)
                || (bytes[at] == b'=' && bytes.get(at + 1) != Some(&b'='))
                || (bytes[at] == b'='
                    && bytes.get(at + 1) == Some(&b'=')
                    && (at == 0 || bytes[at - 1].is_ascii_whitespace())
                    && bytes.get(at + 2).is_none_or(u8::is_ascii_whitespace)))
        {
            at += 1;
        }
        at
    }

    #[test]
    fn matches_scalar_oracle_for_all_option_masks_and_offsets() {
        for options in 0..=15 {
            for len in 0..=80 {
                let mut storage = [b'x'; 80];
                for (at, byte) in storage[..len].iter_mut().enumerate() {
                    *byte = match at % 8 {
                        0 => b'{',
                        1 => b'^',
                        2 => b'$',
                        3 => b'*',
                        4 => 0x80,
                        _ => b'x',
                    };
                }
                let bytes = &storage[..len];
                for from in 0..=len {
                    let mut scan = InlineMarkerScan {
                        optional: options,
                        notes_only: false,
                        scan: ForwardScan::new(),
                    };
                    assert_eq!(
                        scan.next(bytes, from),
                        oracle(bytes, from, options),
                        "options {options}, len {len}, from {from}, bytes {bytes:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn matches_every_byte_value_at_every_offset() {
        for options in 0..=15 {
            for value in 0..=u8::MAX {
                let mut bytes = [b'x'; 72];
                for at in 0..bytes.len() {
                    bytes[at] = value;
                    for from in 0..=at {
                        let mut scan = InlineMarkerScan {
                            optional: options,
                            notes_only: false,
                            scan: ForwardScan::new(),
                        };
                        assert_eq!(scan.next(&bytes, from), oracle(&bytes, from, options));
                    }
                }
            }
        }
    }

    #[test]
    fn matches_valid_utf8_and_earliest_marker() {
        let text = "prefixé 中 {math ^ $ * suffix";
        let bytes = text.as_bytes();
        for options in 0..=15 {
            let mut scan = InlineMarkerScan {
                optional: options,
                notes_only: false,
                scan: ForwardScan::new(),
            };
            for from in 0..=bytes.len() {
                assert_eq!(scan.next(bytes, from), oracle(bytes, from, options));
            }
        }
    }
}
