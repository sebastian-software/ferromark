// Owned strings keep the test oracle independent of production arena storage.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use super::Parser;
use crate::allocator::Allocator;
use crate::parser::byte_class::tests::assert_backends_match_flags;

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
        assert_eq!(
            super::BRACKET_STOP.contains(byte),
            expected,
            "byte {byte:#x}"
        );
    }
}

#[test]
fn bracket_stop_vector_scans_match_flags() {
    assert_backends_match_flags(&super::BRACKET_STOP);
}

#[test]
fn bracket_text_stop_vector_scans_match_flags() {
    assert_backends_match_flags(&super::BRACKET_TEXT_STOP);
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
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let len = (state >> 33) as usize % 80;
        let mut input = String::new();
        for _ in 0..len {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
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
    check(
        &format!("{}`{}`{}]", "a".repeat(20), "]".repeat(20), "b".repeat(20)),
        0,
    );
}

/// The recorded walk has to answer exactly what the plain walk answers, both
/// on the walk that records and on every later scan the recording covers.
///
/// The record is keyed by address, so the parser has to be built on the same
/// string the scans run over.
fn check_matched(content: &str) {
    let allocator = Allocator::new();
    let parser = Parser::new(&allocator, content);
    // Cold: every start walks for itself, some of them recording.
    for from in 0..=content.len() {
        if !content.is_char_boundary(from) {
            continue;
        }
        assert_eq!(
            parser.scan_balanced_matched(content, from),
            Parser::scan_balanced(content, from),
            "cold scan of {content:?} from {from}"
        );
    }
    // Warm: after one recording walk from every start, every answer comes
    // from the map instead.
    let recorded = Parser::new(&allocator, content);
    for from in 0..=content.len() {
        if content.is_char_boundary(from) {
            recorded.record_bracket_matches(content, from);
        }
    }
    for from in 0..=content.len() {
        if !content.is_char_boundary(from) {
            continue;
        }
        assert_eq!(
            recorded.scan_balanced_matched(content, from),
            Parser::scan_balanced(content, from),
            "recorded scan of {content:?} from {from}"
        );
    }
}

#[test]
fn recorded_matches_answer_what_the_walk_answers() {
    let tokens = [
        "a",
        "[",
        "]",
        "[a]",
        "](u)",
        "\\",
        "\\]",
        "`",
        "`c`",
        "<",
        "<b>",
        "<https://e.x/>",
        " ",
        "\n",
        "(",
        ")",
        "[[",
        "]]",
    ];
    let mut state = 0x853c_49e6_748f_ea9bu64;
    for _ in 0..2000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let len = (state >> 33) as usize % 12;
        let mut input = String::new();
        for _ in 0..len {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            input.push_str(tokens[(state >> 33) as usize % tokens.len()]);
        }
        check_matched(&input);
    }
}

#[test]
fn recorded_matches_cover_nested_and_unbalanced_runs() {
    check_matched(&("[".repeat(24) + "a" + &"](u)".repeat(24)));
    check_matched(&("[".repeat(24) + "a](u)"));
    check_matched(&("[".repeat(24) + "a" + &"]".repeat(23)));
    check_matched(&"[x]".repeat(24));
    // An opener inside a skipped region is never recorded, and a scan that
    // starts inside one still has to walk for itself.
    check_matched("[a `[b] c` [d]](u)");
    check_matched("[a <b [c]> [d]](u)");
    check_matched("[a \\[b [c]](u)");
}
