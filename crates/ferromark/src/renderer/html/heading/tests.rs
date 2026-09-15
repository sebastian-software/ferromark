//! Guarantees about the generated slug alphabet.
//!
//! The renderer writes generated heading ids straight into the output instead
//! of running them through attribute escaping. That is only sound while a slug
//! can never contain a byte the escaper would replace, so these tests compare
//! the slug against the real escaper rather than against a hand-written list.

use std::fmt::Write as _;

use super::slugify_heading;
use crate::renderer::html::escape::write_attribute_escaped_into;

/// Checks a candidate id the way `write_prepared_heading_id` assumes it may be
/// checked: escaping it has to be the identity, and every character has to come
/// from the documented alphabet.
///
/// The second assertion is not redundant. It restates the alphabet as a
/// positive claim so a future slugifier change that happens to dodge the
/// escaper's bytes still has to be a deliberate one.
fn assert_emittable_verbatim(input: &str, id: &str, scratch: &mut String) {
    assert!(!id.is_empty(), "empty slug for input {input:?}");
    for ch in id.chars() {
        assert!(
            ch == '-' || ch.is_alphanumeric(),
            "slug for input {input:?} contains {ch:?}"
        );
    }
    scratch.clear();
    write_attribute_escaped_into(scratch, id);
    assert_eq!(
        scratch.as_str(),
        id,
        "attribute escaping rewrote the id for input {input:?}"
    );
}

/// Slugifies `input` and checks the result, reusing `scratch` so the sweeps
/// below do not allocate an escape buffer per iteration.
fn check_slug(input: &str, scratch: &mut String) {
    let slug = slugify_heading(input);
    assert_emittable_verbatim(input, &slug, scratch);
}

/// As `check_slug`, plus the `-N` forms that duplicate headings produce.
fn check_slug_and_suffixes(input: &str, scratch: &mut String) {
    let slug = slugify_heading(input);
    assert_emittable_verbatim(input, &slug, scratch);
    let mut unique = String::new();
    for suffix in [1usize, 9, 10, 1234] {
        unique.clear();
        unique.push_str(&slug);
        let _ = write!(unique, "-{suffix}");
        assert_emittable_verbatim(input, &unique, scratch);
    }
}

#[test]
fn generated_slugs_never_need_attribute_escaping_for_ascii() {
    // Every ASCII byte alone and in every ordered pair: this covers the five
    // escaped bytes, CR/LF, the controls, and their interaction with the
    // separator-collapsing and trailing-trim rules.
    let mut scratch = String::new();
    let mut input = String::new();
    for first in 0u8..0x80 {
        input.clear();
        input.push(first as char);
        check_slug_and_suffixes(&input, &mut scratch);
        for second in 0u8..0x80 {
            input.clear();
            input.push(first as char);
            input.push(second as char);
            check_slug(&input, &mut scratch);
        }
    }
}

#[test]
fn generated_slugs_never_need_attribute_escaping_for_unicode() {
    // Sampled sweep over the whole scalar range. The step is coprime with the
    // common block sizes so it lands inside scripts, punctuation blocks, and
    // the astral planes rather than repeatedly on block boundaries. Each scalar
    // is paired with an escapable ASCII byte so the mixed ASCII/non-ASCII run
    // transitions in the slugifier are exercised too.
    let mut scratch = String::new();
    let mut input = String::new();
    for code in (0u32..=0x0010_FFFF).step_by(37) {
        let Some(ch) = char::from_u32(code) else {
            continue;
        };
        input.clear();
        input.push(ch);
        input.push('&');
        input.push(ch);
        check_slug(&input, &mut scratch);
    }
}

#[test]
fn generated_slugs_never_need_attribute_escaping_for_dense_lower_planes() {
    // Latin, Greek, Cyrillic, Hebrew, Arabic and the general punctuation block
    // are where case folding and `is_alphanumeric` disagree most often, so this
    // range is swept exhaustively instead of sampled.
    let mut scratch = String::new();
    let mut input = String::new();
    for code in 0u32..0x0800 {
        let Some(ch) = char::from_u32(code) else {
            continue;
        };
        input.clear();
        input.push(ch);
        input.push('"');
        input.push(ch);
        check_slug(&input, &mut scratch);
    }
}

#[test]
fn generated_slugs_never_need_attribute_escaping_for_case_folding_edge_cases() {
    // Scalars whose lowercase mapping is longer than one char, or that map a
    // non-ASCII scalar onto an ASCII one. A multi-byte UTF-8 encoding holds no
    // ASCII byte, so case folding is the only way a non-ASCII input can
    // contribute an ASCII byte to a slug at all.
    let mut scratch = String::new();
    for input in [
        "\u{0130}", // LATIN CAPITAL LETTER I WITH DOT ABOVE -> "i\u{307}"
        "\u{212A}", // KELVIN SIGN -> "k"
        "\u{212B}", // ANGSTROM SIGN -> "\u{E5}"
        "\u{1E9E}", // LATIN CAPITAL LETTER SHARP S -> "\u{DF}"
        "\u{FB00}", // LATIN SMALL LIGATURE FF
        "\u{FF21}", // FULLWIDTH LATIN CAPITAL LETTER A
        "\u{0345}", // COMBINING GREEK YPOGEGRAMMENI -> "\u{3B9}"
        "\u{1F88}", // GREEK CAPITAL LETTER ALPHA WITH PSILI AND PROSGEGRAMMENI
        "\u{0130}\u{212A}\"&<>'",
        "はじめに",
        "Über — Größe & Maß",
        "\r\n\r\n",
        "",
        "   ",
        "!!!???",
        "---",
        "a&b<c>d\"e'f",
        "Options & Defaults\r\nPart 2",
    ] {
        check_slug_and_suffixes(input, &mut scratch);
    }
}

#[test]
fn empty_and_punctuation_only_headings_fall_back_to_section() {
    for input in ["", "   ", "!!!", "---", "\r\n", "&<>\"'"] {
        assert_eq!(slugify_heading(input), "section", "{input:?}");
    }
}

#[test]
fn a_very_long_heading_keeps_the_slug_alphabet() {
    let mut scratch = String::new();
    let long_ascii = "Configuring the Renderer, Part 3: Options & Defaults! ".repeat(200);
    check_slug_and_suffixes(&long_ascii, &mut scratch);
    let long_mixed = "設定 Options & 既定値 — part 3 ".repeat(200);
    check_slug_and_suffixes(&long_mixed, &mut scratch);
}
