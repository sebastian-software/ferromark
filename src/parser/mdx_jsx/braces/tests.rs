#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use super::{record_brace_matches, skip_braces};

fn scan(source: &str) -> Option<(&str, usize)> {
    let end = record_brace_matches(source.as_bytes(), 0, &mut |_, _| {}).close?;
    Some((&source[1..end - 1], end))
}

#[test]
fn scan_keeps_inner_source_without_braces() {
    let (value, end) = scan("{items.map}").expect("balanced");
    assert_eq!(value, "items.map");
    assert_eq!(end, 11);
}

#[test]
fn scan_skips_block_comment_with_braces_inside() {
    let (value, _) = scan("{/* hide } */}").expect("comment");
    assert_eq!(value, "/* hide } */");
}

#[test]
fn scan_skips_nested_braces_and_strings() {
    let source = "{...{html:\"<script>\"}}";
    let (value, end) = scan(source).expect("nested");
    assert_eq!(value, "...{html:\"<script>\"}");
    assert_eq!(end, source.len());
}

#[test]
fn unclosed_brace_or_comment_is_none() {
    assert!(skip_braces(b"{items.map", 0).is_none());
    assert!(skip_braces(b"{/* hide", 0).is_none());
    assert!(skip_braces(b"{foo /* bar", 0).is_none());
}

/// The record answers for every brace one walk passes, so each answer has
/// to be the one a plain scan from that brace reaches — the braces it
/// reports closed, and the ones its range leaves unreported.
#[test]
fn recorded_matches_agree_with_a_plain_skip() {
    for source in [
        "{{{{",
        "{{{{}",
        "{a{b{c}}}",
        "{a}{b}{c}",
        "{{{ }}}",
        "{\"s{\"}{'t}'}",
        "{`x{`}",
        "{/* } */{}}",
        "{// }\n{}}",
        "{/* unterminated {",
        "{\"unterminated {",
        "{\"a{b}c\"",
        "{\"a{b}c\"{{",
        "{// c {\n{{",
        "{}",
        "not a brace",
    ] {
        let bytes = source.as_bytes();
        for start in 0..bytes.len() {
            if bytes[start] != b'{' {
                continue;
            }
            let mut recorded = std::collections::HashMap::new();
            let walk = record_brace_matches(bytes, start, &mut |brace, close| {
                recorded.insert(brace, close);
            });
            assert_eq!(
                walk.close,
                skip_braces(bytes, start),
                "outer answer for {source:?} at {start}"
            );
            for (brace, close) in &recorded {
                assert_eq!(
                    *close,
                    skip_braces(bytes, *brace),
                    "recorded answer for {source:?} at {brace}"
                );
            }
            if walk.close.is_some() {
                continue;
            }
            // The range the walk read is the claim the memo makes about
            // every brace it did not report: nothing closes it.
            let (from, until) = walk.read;
            for brace in from..until {
                if bytes[brace] == b'{' && !recorded.contains_key(&brace) {
                    assert_eq!(
                        skip_braces(bytes, brace),
                        None,
                        "unreported brace in {source:?} at {brace} does close"
                    );
                }
            }
        }
    }
}
