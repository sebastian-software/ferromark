// Owned strings keep the generated inputs independent of production arena
// storage.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use super::super::gfm_autolink::{AutolinkScan, may_contain_autolink};
use super::scalar::{
    next_inline_special_scalar, next_marker_tracking_scalar, visit_autolink_triggers_scalar,
};
use super::*;

/// The facts the scalar rule derives from every position before `upto`:
/// the definition the vector visits have to agree with.
fn facts_before(bytes: &[u8], upto: usize) -> [bool; 4] {
    let mut facts = AutolinkFacts::new();
    for at in 0..upto {
        facts.visit(bytes, at);
    }
    facts.facts()
}

/// The marker set of `next_inline_marker`, spelled out.
fn is_marker(byte: u8, options: u8) -> bool {
    matches!(
        byte,
        b'*' | b'_' | b'`' | b'[' | b'!' | b'~' | b'\\' | b'<' | b'&' | b'\n' | b'\r'
    ) || (options & INLINE_MARKER_MDX != 0 && byte == b'{')
        || (options & INLINE_MARKER_SUPERSCRIPT != 0 && byte == b'^')
        || (options & INLINE_MARKER_MATH != 0 && byte == b'$')
        || (options & INLINE_MARKER_HIGHLIGHT != 0 && byte == b'=')
}

/// One tracking scan from a fresh watermark: it must stop where the plain
/// scan stops, leave the watermark behind the stop, and have visited
/// exactly the triggers in front of the watermark.
#[track_caller]
fn check_tracking(bytes: &[u8], from: usize, options: u8) {
    let mut facts = AutolinkFacts::new();
    let hit = next_inline_marker_tracking(bytes, from, options, &mut facts);
    assert_eq!(
        hit,
        next_inline_marker(bytes, from, options),
        "stop: options {options}, from {from}, bytes {bytes:?}"
    );
    let covered = if hit < bytes.len() {
        hit + 1
    } else {
        bytes.len()
    };
    assert!(
        facts.seen() >= covered.max(from.min(bytes.len())),
        "watermark {} behind {covered}: options {options}, from {from}, bytes {bytes:?}",
        facts.seen()
    );
    assert_eq!(
        facts.facts(),
        facts_before(bytes, facts.seen()),
        "facts: options {options}, from {from}, bytes {bytes:?}"
    );

    // The portable scan, which vector targets only reach for short content,
    // on the same input and from the same watermark.
    let mut portable = AutolinkFacts::new();
    portable.fill_to(bytes, from.min(bytes.len()));
    if from < bytes.len() {
        let stop = next_marker_tracking_scalar(bytes, from, options, &mut portable);
        assert_eq!(stop, hit, "portable stop: options {options}, from {from}");
        assert!(portable.seen() >= covered);
        assert_eq!(
            portable.facts(),
            facts_before(bytes, portable.seen()),
            "portable facts: options {options}, from {from}, bytes {bytes:?}"
        );
    }
}

#[test]
fn tracked_classes_are_the_markers_and_the_triggers() {
    for options in 0..=15u8 {
        for byte in 0..=255u8 {
            let class = TRACKED_CLASS[byte as usize];
            assert_eq!(
                class & tracked_marker_bits(options) != 0,
                is_marker(byte, options),
                "marker class of {byte:#04x}, options {options}"
            );
            assert_eq!(
                class & AUTOLINK_TRIGGER != 0,
                matches!(byte, b'@' | b':' | b'.'),
                "trigger class of {byte:#04x}"
            );
            assert_eq!(is_autolink_trigger(byte), class & AUTOLINK_TRIGGER != 0);
            // The watermark moves past a marker without visiting it.
            assert!(!(is_marker(byte, options) && is_autolink_trigger(byte)));
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[test]
fn tracking_tables_admit_exactly_the_markers_and_two_triggers() {
    for options in 0..=15u8 {
        let (low, high) = selected_tracking_tables(options);
        for byte in 0..=255u8 {
            let stops = low[(byte & 0x0F) as usize] & high[(byte >> 4) as usize] != 0;
            assert_eq!(
                stops,
                is_marker(byte, options) || byte == b'@' || byte == b':',
                "byte {byte:#04x}, options {options}"
            );
        }
    }
}

#[test]
fn tracking_scan_agrees_for_every_byte_value_at_every_offset() {
    // A trigger-free background, then one with a trigger of every kind, so
    // the byte under test is sometimes the only fact and sometimes one of
    // many. Neither background holds a marker.
    let backgrounds: [&[u8]; 2] = [
        &[b'x'; 72],
        b"mailto:a@b.cwww.x://y.zw.xmpp:q//w.wwww.:@xhttps://www.x.y@z.w.ww.x",
    ];
    for background in backgrounds {
        for len in [12usize, 40, 72] {
            let len = len.min(background.len());
            for options in [0u8, 6, 15] {
                for value in 0..=255u8 {
                    for at in 0..len {
                        let mut buffer = [0u8; 72];
                        buffer[..len].copy_from_slice(&background[..len]);
                        buffer[at] = value;
                        let bytes = &buffer[..len];
                        for from in [
                            0,
                            1,
                            15,
                            16,
                            17,
                            31,
                            32,
                            33,
                            at.saturating_sub(1),
                            at,
                            at + 1,
                        ] {
                            if from <= len {
                                check_tracking(bytes, from, options);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn tracking_scan_sees_every_needle_across_vector_edges() {
    // Each needle at every offset of an otherwise plain line, with a marker
    // before it, after it, or nowhere, scanned from every offset — so the
    // `w.` pair straddles vector and tail edges and the watermark start.
    for needle in [
        &b"www."[..],
        b"http://",
        b"](http://",
        b"](HTTPS://",
        b"mailto:",
        b"xmpp:",
        b"@",
        b"w.",
        b"ww.",
    ] {
        for len in [9usize, 15, 16, 17, 31, 32, 33, 48, 70] {
            for at in 0..=len.saturating_sub(needle.len()) {
                for marker in [None, Some(0), Some(at.saturating_sub(1)), Some(len - 1)] {
                    let mut buffer = [b'x'; 72];
                    buffer[at..at + needle.len()].copy_from_slice(needle);
                    if let Some(marker) = marker
                        && !(at..at + needle.len()).contains(&marker)
                    {
                        buffer[marker] = b'*';
                    }
                    let bytes = &buffer[..len];
                    for from in 0..=len {
                        for options in [0u8, 15] {
                            check_tracking(bytes, from, options);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn tracking_walks_answer_the_preflight() {
    // A walk the way the inline parser drives the scan: forward past a
    // one-byte marker, over a construct the scan never classified, and
    // back to re-scan text it already passed. Whatever the path, finishing
    // must give the separate pre-flight's answer.
    let tokens = [
        "www.",
        "ww",
        "w.",
        ".",
        "w",
        "://",
        ":",
        "](http://",
        "](ftp://",
        "http",
        "mailto:",
        "xmpp:",
        "@",
        "a@b.c",
        "*",
        "`",
        "[",
        "]",
        "(",
        ")",
        "\\",
        "&",
        "<",
        "\n",
        "$",
        "^",
        "=",
        "{",
        " ",
        "x",
        "abcdefghijklmnop",
        "é",
        "中",
    ];
    let mut state = 0x853c_49e6_748f_ea9bu64;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as usize
    };
    for _ in 0..3_000 {
        let mut source = std::string::String::new();
        for _ in 0..next() % 40 {
            source.push_str(tokens[next() % tokens.len()]);
        }
        let bytes = source.as_bytes();
        let options = (next() % 16) as u8;
        let mut facts = AutolinkFacts::new();
        let mut pos = 0;
        let mut rewinds = 0;
        loop {
            let hit = next_inline_marker_tracking(bytes, pos, options, &mut facts);
            assert_eq!(hit, next_inline_marker(bytes, pos, options), "{source:?}");
            assert_eq!(
                facts.facts(),
                facts_before(bytes, facts.seen()),
                "{source:?} at {pos}"
            );
            if hit >= bytes.len() {
                break;
            }
            pos = match next() % 8 {
                0 if rewinds < 3 => {
                    rewinds += 1;
                    next() % (hit + 1)
                }
                1 | 2 => (hit + 1 + next() % 24).min(bytes.len()),
                _ => hit + 1,
            };
        }
        let answer = |scan: Option<AutolinkScan>| {
            scan.map(|scan| (scan.may_have_www, scan.may_have_extended))
        };
        assert_eq!(
            answer(facts.finish(bytes)),
            answer(may_contain_autolink(&source)),
            "{source:?}"
        );
    }
}

#[test]
fn bulk_trigger_visits_match_the_scalar_rule() {
    let tokens = [
        "www.",
        "w.",
        ".",
        "w",
        "://",
        ":",
        "](http://",
        "mailto:",
        "xmpp:",
        "@",
        "x",
        " ",
        "abcdefghijklmnop",
        "é",
    ];
    let mut state = 0xda3e_39cb_94b9_5bdbu64;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as usize
    };
    for _ in 0..150 {
        let mut source = std::string::String::new();
        for _ in 0..next() % 16 {
            source.push_str(tokens[next() % tokens.len()]);
        }
        let bytes = source.as_bytes();
        for from in 0..=bytes.len() {
            for to in from..=bytes.len() {
                let mut bulk = AutolinkFacts::new();
                visit_autolink_triggers(bytes, from, to, &mut bulk);
                let mut scalar = AutolinkFacts::new();
                visit_autolink_triggers_scalar(bytes, from, to, &mut scalar);
                let mut each = AutolinkFacts::new();
                for at in from..to {
                    each.visit(bytes, at);
                }
                assert_eq!(bulk.facts(), each.facts(), "{source:?} {from}..{to}");
                assert_eq!(scalar.facts(), each.facts(), "{source:?} {from}..{to}");
            }
        }
    }
}

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
