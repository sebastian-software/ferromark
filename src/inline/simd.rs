//! Markdown byte-set definitions. Native search lives in `byte_search`.
use crate::byte_search::ByteSet;

const BASE: ByteSet<10> = ByteSet::new(b"*_`[]<\\\n~$");
const HIGHLIGHT: ByteSet<11> = ByteSet::new(b"*_`[]<\\\n~$=");
const SUPERSCRIPT: ByteSet<11> = ByteSet::new(b"*_`[]<\\\n~$^");
const BOTH: ByteSet<12> = ByteSet::new(b"*_`[]<\\\n~$=^");

#[inline]
pub(super) fn has_inline_specials<const H: bool, const S: bool>(input: &[u8]) -> bool {
    if H && S {
        BOTH.contains_any(input)
    } else if H {
        HIGHLIGHT.contains_any(input)
    } else if S {
        SUPERSCRIPT.contains_any(input)
    } else {
        BASE.contains_any(input)
    }
}

#[inline]
pub(super) fn next_mark_special<const H: bool, const S: bool>(input: &[u8]) -> Option<usize> {
    if H && S {
        BOTH.find(input)
    } else if H {
        HIGHLIGHT.find(input)
    } else if S {
        SUPERSCRIPT.find(input)
    } else {
        BASE.find(input)
    }
}
