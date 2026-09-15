//! Portable marker scanning and short-vector tails.

use super::INLINE_SPECIAL;

/// How far `next_inline_special` walks byte-at-a-time before switching to the
/// chunked scan. Eight is the only length measured that never regressed:
/// longer prefixes bought a little more on code-dense corpora and lost
/// 12-16% on prose-dense ones.
const SHORT_RUN_PREFIX: usize = 8;

/// Walks at most [`SHORT_RUN_PREFIX`] bytes. Returns the first marker, or
/// `end` when the prefix is clean and the slice ends there, or the first
/// unexamined index when the prefix is clean and more bytes remain.
///
/// Shared by the scalar scan. Putting the short prefix on the vector paths
/// too was measured slower: most of the *bytes* sit in long runs, and those
/// paid eight extra flagged loads before the first vector.
#[inline]
fn scan_short_prefix(bytes: &[u8], from: usize) -> usize {
    let end = bytes.len();
    let quick = from.saturating_add(SHORT_RUN_PREFIX).min(end);
    let mut i = from;
    while i < quick && INLINE_SPECIAL[bytes[i] as usize] == 0 {
        i += 1;
    }
    i
}

#[cfg_attr(target_arch = "aarch64", allow(dead_code))]
#[inline]
pub(super) fn next_inline_special_scalar(bytes: &[u8], from: usize) -> usize {
    let end = bytes.len();

    // Text runs are strongly bimodal on the bundled corpora: 53-64% of them
    // end within eight bytes while holding under 5% of the bytes, and 44-52%
    // of the bytes sit in runs of 64 or more. Walking that short prefix one
    // byte at a time means the common run never pays the chunked scan's eight
    // lookups and seven ORs to find a marker two bytes in, while long runs —
    // where the bytes actually are — still reach the wide scan below.
    let mut i = scan_short_prefix(bytes, from);
    if i >= end || INLINE_SPECIAL[bytes[i] as usize] != 0 {
        return i;
    }

    // Skip eight bytes at a time while the OR of their marker flags is zero.
    // This is not a semantic parser: it only proves that none of those bytes
    // can start inline syntax, so returning the first flagged byte preserves
    // the exact same marker positions as the previous per-byte loop.
    while i + 8 <= end {
        let chunk = &bytes[i..i + 8];
        let mask = INLINE_SPECIAL[chunk[0] as usize]
            | INLINE_SPECIAL[chunk[1] as usize]
            | INLINE_SPECIAL[chunk[2] as usize]
            | INLINE_SPECIAL[chunk[3] as usize]
            | INLINE_SPECIAL[chunk[4] as usize]
            | INLINE_SPECIAL[chunk[5] as usize]
            | INLINE_SPECIAL[chunk[6] as usize]
            | INLINE_SPECIAL[chunk[7] as usize];
        if mask != 0 {
            break;
        }
        i += 8;
    }
    while i < end && INLINE_SPECIAL[bytes[i] as usize] == 0 {
        i += 1;
    }
    i
}

#[allow(dead_code)]
#[inline]
pub(super) fn next_inline_special_options_scalar(bytes: &[u8], from: usize, options: u8) -> usize {
    let mut i = from;
    while i < bytes.len() && !is_inline_marker(bytes[i], options) {
        i += 1;
    }
    i
}

#[inline]
fn is_inline_marker(byte: u8, options: u8) -> bool {
    matches!(
        byte,
        b'*' | b'_' | b'`' | b'[' | b'!' | b'~' | b'\\' | b'<' | b'&' | b'\n' | b'\r'
    ) || (options & 1 != 0 && byte == b'{')
        || (options & 2 != 0 && byte == b'^')
        || (options & 4 != 0 && byte == b'$')
        || (options & 8 != 0 && byte == b'=')
}
