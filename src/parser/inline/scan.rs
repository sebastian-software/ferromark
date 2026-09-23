//! Scanning for the next byte that can begin an inline construct.
//!
//! This is the parser's hottest loop: it classifies every byte of every
//! block's inline content. Ten markers is too many for the `memchr` family
//! and too scattered for a SWAR fold, so the portable path is a 256-entry
//! flag table read eight bytes at a time — one load per byte.
//!
//! Both SIMD paths replace that with a nibble-pair classifier that needs no
//! per-byte load at all, and they run the *same* classifier: aarch64 uses
//! `vqtbl1q_u8` and x86-64 `pshufb`, which are the same 16-entry table
//! lookup. AVX2 broadcasts that table into both 128-bit lanes so one shuffle
//! pair covers 32 bytes. Every path is checked against the flag table by the
//! differential tests in `tests.rs`, for all 256 byte values at every offset.

#[cfg(target_arch = "aarch64")]
mod neon;
mod scalar;
#[cfg(target_arch = "x86_64")]
mod x86;

#[cfg(target_arch = "aarch64")]
use neon::{
    next_marker_tracking_neon, next_special_neon, next_special_neon_options,
    visit_autolink_triggers_neon,
};
#[cfg(not(target_arch = "aarch64"))]
use scalar::{
    next_inline_special_options_scalar, next_inline_special_scalar, next_marker_tracking_scalar,
    visit_autolink_triggers_scalar,
};

use super::gfm_autolink::AutolinkFacts;
#[cfg(target_arch = "x86_64")]
use x86::{
    next_special_avx2, next_special_avx2_options, next_special_ssse3, next_special_ssse3_options,
};

/// Lookup table: `INLINE_SPECIAL[b] == 1` iff the byte can begin an inline
/// construct handled by `parse_inline_special`.
///
/// This is the definition the vectorized paths must agree with, and the
/// scan every target without one falls back to. The table deliberately
/// stores flags instead of enum variants: the scalar scan ORs eight entries
/// at a time, which gives LLVM a branch-free loop for long runs of normal
/// text. The actual parser decision still happens only after a candidate
/// byte is found.
static INLINE_SPECIAL: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'*' as usize] = 1;
    t[b'_' as usize] = 1;
    t[b'`' as usize] = 1;
    t[b'[' as usize] = 1;
    t[b'!' as usize] = 1;
    t[b'~' as usize] = 1;
    t[b'\\' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'\n' as usize] = 1;
    t[b'\r' as usize] = 1;
    t[b'&' as usize] = 1;
    t
};

/// Nibble-pair classifier tables for the vectorized paths.
///
/// A byte is special iff `LOW[b & 0x0F] & HIGH[b >> 4]` is nonzero. The ten
/// markers fall into six (high nibble, low-nibble set) groups — line endings;
/// `!`/`&`/`*`; `<`; `[`/`\`/`_`; `` ` ``; `~` — and each group owns one
/// bit, so the AND is exact: it admits no byte outside the set, and every
/// byte >= 0x80 maps to a zero high-nibble entry.
///
/// A 16-entry table is what a vector shuffle can hold, which is the whole
/// point: `vqtbl1q_u8` / `pshufb` look up all sixteen lanes in one
/// instruction, where the flag table needs sixteen loads.
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
const LOW_NIBBLE: [u8; 16] = [
    0x10, 0x02, 0, 0, 0, 0, 0x02, 0, 0, 0, 0x03, 0x08, 0x0C, 0x01, 0x20, 0x08,
];
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
const HIGH_NIBBLE: [u8; 16] = [
    0x01, 0, 0x02, 0x04, 0, 0x08, 0x10, 0x20, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// The optional bytes use otherwise-unused intersections: `$` gets bit 0x40
/// at (high 2, low 4), `^` gets bit 0x80 at (high 5, low E), and `{` reuses
/// the existing `~` bit at (high 7, low B). These choices avoid admitting any
/// cross-product byte. `=` shares the `<` bit at (high 3, low D).
/// Sixteen precomputed pairs let each scan use only the
/// enabled extension markers; disabled bytes never need retry filtering.
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
const fn marker_tables(options: u8) -> ([u8; 16], [u8; 16]) {
    let mut low = LOW_NIBBLE;
    let mut high = HIGH_NIBBLE;
    if options & 1 != 0 {
        low[11] |= 0x20;
    }
    if options & 2 != 0 {
        low[14] |= 0x80;
        high[5] |= 0x80;
    }
    if options & 4 != 0 {
        low[4] |= 0x40;
        high[2] |= 0x40;
    }
    if options & 8 != 0 {
        low[13] |= 0x04; // `=` shares the high nibble of `<`.
    }
    (low, high)
}

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
const OPTION_TABLES: [([u8; 16], [u8; 16]); 16] = [
    marker_tables(0),
    marker_tables(1),
    marker_tables(2),
    marker_tables(3),
    marker_tables(4),
    marker_tables(5),
    marker_tables(6),
    marker_tables(7),
    marker_tables(8),
    marker_tables(9),
    marker_tables(10),
    marker_tables(11),
    marker_tables(12),
    marker_tables(13),
    marker_tables(14),
    marker_tables(15),
];

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[inline]
fn selected_marker_tables(options: u8) -> (&'static [u8; 16], &'static [u8; 16]) {
    let tables = &OPTION_TABLES[(options & 15) as usize];
    (&tables.0, &tables.1)
}

/// The marker tables with the two single-byte GFM autolink triggers, `@`
/// and `:`, admitted as stops too, for the scan that also answers the
/// autolink pre-flight ([`next_inline_marker_tracking`]).
///
/// Neither needs a bit of its own, which is what keeps the classifier at
/// one shuffle pair: `@` (high 4, low 0) joins the backtick bit 0x10, whose
/// only low-nibble entry is 0, and `:` (high 3, low A) joins the `<` bit
/// 0x04, whose only high-nibble entry is 3. No option set adds either bit
/// anywhere else, so each addition admits exactly its own byte. The stop
/// handler tells the triggers from markers by the byte itself.
#[cfg(target_arch = "aarch64")]
const fn tracking_marker_tables(options: u8) -> ([u8; 16], [u8; 16]) {
    let (mut low, mut high) = marker_tables(options);
    high[4] |= 0x10;
    low[10] |= 0x04;
    (low, high)
}

#[cfg(target_arch = "aarch64")]
const TRACKING_TABLES: [([u8; 16], [u8; 16]); 16] = [
    tracking_marker_tables(0),
    tracking_marker_tables(1),
    tracking_marker_tables(2),
    tracking_marker_tables(3),
    tracking_marker_tables(4),
    tracking_marker_tables(5),
    tracking_marker_tables(6),
    tracking_marker_tables(7),
    tracking_marker_tables(8),
    tracking_marker_tables(9),
    tracking_marker_tables(10),
    tracking_marker_tables(11),
    tracking_marker_tables(12),
    tracking_marker_tables(13),
    tracking_marker_tables(14),
    tracking_marker_tables(15),
];

#[cfg(target_arch = "aarch64")]
#[inline]
fn selected_tracking_tables(options: u8) -> (&'static [u8; 16], &'static [u8; 16]) {
    let tables = &TRACKING_TABLES[(options & 15) as usize];
    (&tables.0, &tables.1)
}

/// Byte classes for the scalar tracking scans: bit 0 marks the core markers
/// of [`INLINE_SPECIAL`], bits 1–4 the optional markers in `INLINE_MARKER_*`
/// order (so `options << 1` selects them), and [`AUTOLINK_TRIGGER`] the bytes
/// the autolink pre-flight derives its facts from.
///
/// No marker is a trigger, which is what lets a scan move the pre-flight
/// watermark past the marker it stops at without visiting it.
static TRACKED_CLASS: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut byte = 0;
    while byte < 256 {
        t[byte] = INLINE_SPECIAL[byte];
        byte += 1;
    }
    t[b'{' as usize] |= INLINE_MARKER_MDX << 1;
    t[b'^' as usize] |= INLINE_MARKER_SUPERSCRIPT << 1;
    t[b'$' as usize] |= INLINE_MARKER_MATH << 1;
    t[b'=' as usize] |= INLINE_MARKER_HIGHLIGHT << 1;
    t[b'@' as usize] |= AUTOLINK_TRIGGER;
    t[b':' as usize] |= AUTOLINK_TRIGGER;
    t[b'.' as usize] |= AUTOLINK_TRIGGER;
    t
};

/// [`TRACKED_CLASS`] bit of the bytes `AutolinkFacts::visit` acts on.
const AUTOLINK_TRIGGER: u8 = 0x80;

/// The [`TRACKED_CLASS`] bits that stop a scan with these optional markers.
#[inline]
const fn tracked_marker_bits(options: u8) -> u8 {
    1 | ((options & 15) << 1)
}

/// True for the bytes `AutolinkFacts::visit` acts on.
#[inline]
fn is_autolink_trigger(byte: u8) -> bool {
    TRACKED_CLASS[byte as usize] & AUTOLINK_TRIGGER != 0
}

/// [`next_inline_marker`] for a scan that also answers the block's GFM
/// autolink pre-flight: returns the same position, and on the way visits
/// every autolink trigger before it that `facts` has not seen yet.
///
/// Bytes between the watermark and `from` were consumed by a construct the
/// marker scan never classified, so they are visited first. Afterwards the
/// watermark stands behind the returned marker: every trigger before it has
/// been visited, and the marker byte itself is never one.
#[inline]
pub(super) fn next_inline_marker_tracking(
    bytes: &[u8],
    from: usize,
    options: u8,
    facts: &mut AutolinkFacts,
) -> usize {
    facts.fill_to(bytes, from.min(bytes.len()));
    if from >= bytes.len() {
        return from;
    }
    #[cfg(target_arch = "aarch64")]
    {
        next_marker_tracking_neon(bytes, from, options, facts)
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        next_marker_tracking_scalar(bytes, from, options, facts)
    }
}

/// Visits every autolink trigger in `from..to`; the bytes around the range
/// still count as context. The caller owns the watermark.
#[inline]
pub(super) fn visit_autolink_triggers(
    bytes: &[u8],
    from: usize,
    to: usize,
    facts: &mut AutolinkFacts,
) {
    #[cfg(target_arch = "aarch64")]
    visit_autolink_triggers_neon(bytes, from, to, facts);
    #[cfg(not(target_arch = "aarch64"))]
    visit_autolink_triggers_scalar(bytes, from, to, facts);
}

#[cfg(target_arch = "aarch64")]
#[inline]
pub(super) fn next_inline_special(bytes: &[u8], from: usize) -> usize {
    next_special_neon(bytes, from)
}

#[cfg(target_arch = "x86_64")]
#[inline]
pub(super) fn next_inline_special(bytes: &[u8], from: usize) -> usize {
    // AVX2 / SSSE3 are not in the x86-64 baseline, so they are detected
    // rather than assumed. `is_x86_feature_detected!` caches its answer
    // in an atomic, and a machine without either keeps the scalar scan.
    if std::arch::is_x86_feature_detected!("avx2") {
        // SAFETY: guarded by the detection above.
        #[allow(unsafe_code)]
        unsafe {
            next_special_avx2(bytes, from)
        }
    } else if std::arch::is_x86_feature_detected!("ssse3") {
        // SAFETY: guarded by the detection above.
        #[allow(unsafe_code)]
        unsafe {
            next_special_ssse3(bytes, from)
        }
    } else {
        next_inline_special_scalar(bytes, from)
    }
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
#[inline]
pub(super) fn next_inline_special(bytes: &[u8], from: usize) -> usize {
    next_inline_special_scalar(bytes, from)
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn next_inline_special_options(bytes: &[u8], from: usize, options: u8) -> usize {
    next_special_neon_options(bytes, from, options)
}

#[cfg(target_arch = "x86_64")]
#[inline]
fn next_inline_special_options(bytes: &[u8], from: usize, options: u8) -> usize {
    if std::arch::is_x86_feature_detected!("avx2") {
        #[allow(unsafe_code)]
        unsafe {
            next_special_avx2_options(bytes, from, options)
        }
    } else if std::arch::is_x86_feature_detected!("ssse3") {
        #[allow(unsafe_code)]
        unsafe {
            next_special_ssse3_options(bytes, from, options)
        }
    } else {
        next_inline_special_options_scalar(bytes, from, options)
    }
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
#[inline]
fn next_inline_special_options(bytes: &[u8], from: usize, options: u8) -> usize {
    next_inline_special_options_scalar(bytes, from, options)
}

/// Marker bits used by [`next_inline_marker`].
pub(super) const INLINE_MARKER_MDX: u8 = 1 << 0;
pub(super) const INLINE_MARKER_SUPERSCRIPT: u8 = 1 << 1;
pub(super) const INLINE_MARKER_HIGHLIGHT: u8 = 1 << 3;
pub(super) const INLINE_MARKER_MATH: u8 = 1 << 2;

/// Find the next core marker or enabled extension marker.
///
/// The core marker scan stays on its architecture-specific classifier when no
/// extensions are enabled. With extensions, one option-specific nibble table
/// is selected and the shared SIMD path emits only enabled markers.
#[inline]
pub(super) fn next_inline_marker(bytes: &[u8], from: usize, options: u8) -> usize {
    if options == 0 {
        return next_inline_special(bytes, from);
    }

    next_inline_special_options(bytes, from, options)
}

#[cfg(test)]
mod tests;
