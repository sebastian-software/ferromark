use super::scalar::next_inline_special_scalar;
use super::*;

/// The definition the chunked scan has to agree with.
fn reference(bytes: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < bytes.len() && INLINE_SPECIAL[bytes[i] as usize] == 0 {
        i += 1;
    }
    i
}

#[test]
fn matches_reference_around_the_prefix_boundary() {
    // A marker at every offset, in inputs long and short enough to exercise
    // the short-run prefix, the prefix/chunk handoff, and the scalar tail.
    for len in 0..72usize {
        for marker_at in 0..len {
            let mut buffer = [b'x'; 72];
            buffer[marker_at] = b'*';
            let bytes = &buffer[..len];
            for from in 0..=len {
                assert_eq!(
                    next_inline_special(bytes, from),
                    reference(bytes, from),
                    "len {len}, marker at {marker_at}, from {from}"
                );
            }
        }
    }
}

#[test]
fn matches_reference_with_no_marker() {
    let buffer = [b'x'; 72];
    for len in 0..=buffer.len() {
        let bytes = &buffer[..len];
        for from in 0..=len {
            assert_eq!(next_inline_special(bytes, from), reference(bytes, from));
        }
    }
}

#[test]
fn matches_reference_for_every_byte_value_at_every_offset() {
    // The vectorized classifiers decide from nibble pairs rather than a
    // 256-entry table, so a wrong entry would admit or drop a byte the
    // flag table disagrees with. Check all 256 values, at every offset
    // of a buffer long enough to cross the 32-byte AVX2 / dual-NEON
    // step, the 16-byte overlapping tail, and the scalar remainder.
    for value in 0..=255u8 {
        for at in 0..72usize {
            let mut buffer = [b'x'; 72];
            buffer[at] = value;
            let bytes = &buffer[..];
            assert_eq!(
                next_inline_special(bytes, 0),
                reference(bytes, 0),
                "byte {value:#04x} at {at}"
            );
            assert_eq!(
                next_inline_special_scalar(bytes, 0),
                reference(bytes, 0),
                "scalar: byte {value:#04x} at {at}"
            );
        }
    }
}

#[test]
fn matches_reference_on_every_marker_byte() {
    for marker in *b"*_`[!~\\<\n&" {
        for lead in 0..20usize {
            let mut buffer = [b'x'; 40];
            buffer[lead] = marker;
            let bytes = &buffer[..lead + 8];
            assert_eq!(
                next_inline_special(bytes, 0),
                reference(bytes, 0),
                "marker {marker:?} at {lead}"
            );
        }
    }
}

#[test]
fn matches_reference_on_overlapping_vector_tails() {
    // Markers in the last 1..=31 bytes, scanned from every offset, so
    // the 16-byte and 32-byte overlapping re-reads have to agree with
    // the flag table after masking off already-cleared lanes.
    for len in 16..72usize {
        for marker_at in (len.saturating_sub(31))..len {
            let mut buffer = [b'x'; 72];
            buffer[marker_at] = b'*';
            let bytes = &buffer[..len];
            for from in 0..=len {
                assert_eq!(
                    next_inline_special(bytes, from),
                    reference(bytes, from),
                    "len {len}, marker at {marker_at}, from {from}"
                );
            }
        }
    }
}
