//! Guarantees about the generated slug alphabet.
//!
//! The renderer writes generated heading ids straight into the output instead
//! of running them through attribute escaping. That is only sound while a slug
//! can never contain a byte the escaper would replace, so these tests compare
//! the slug against the real escaper rather than against a hand-written list.

use std::fmt::Write as _;

use super::{
    HeadingIdPlanner, heading_has_permalink_marker, slugify_heading, slugify_heading_into,
};
use crate::allocator::Allocator;
use crate::ast::Node;
use crate::parser::Parser;
use crate::renderer::html::escape::write_attribute_escaped_into;

/// The `String::push`-per-character slugifier that `slugify_heading_into`
/// replaced, kept verbatim as a differential oracle.
///
/// The byte-cursor rewrite is only worth keeping while it is indistinguishable
/// from this, including the "append, do not clear" contract and the
/// `start_len`-relative trailing trim.
fn slugify_heading_into_oracle(text: &str, out: &mut String) {
    let bytes = text.as_bytes();
    out.reserve(text.len());
    let start_len = out.len();
    let mut last_was_separator = true;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b < 0x80 {
            if b.is_ascii_alphanumeric() {
                let lower = if b.is_ascii_uppercase() { b + 32 } else { b };
                out.push(lower as char);
                last_was_separator = false;
            } else if !last_was_separator {
                out.push('-');
                last_was_separator = true;
            }
            i += 1;
        } else {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] >= 0x80 {
                j += 1;
            }
            for ch in text[i..j].chars() {
                for lower in ch.to_lowercase() {
                    if lower.is_alphanumeric() {
                        out.push(lower);
                        last_was_separator = false;
                    } else if !last_was_separator {
                        out.push('-');
                        last_was_separator = true;
                    }
                }
            }
            i = j;
        }
    }

    while out.len() > start_len && out.ends_with('-') {
        out.pop();
    }

    if out.len() == start_len {
        out.push_str("section");
    }
}

/// Asserts that the rewrite matches the oracle, both into an empty buffer and
/// appended after each prefix, so the `start_len` bookkeeping is covered too.
fn assert_matches_oracle(input: &str) {
    let mut actual = String::new();
    let mut expected = String::new();
    slugify_heading_into(input, &mut actual);
    slugify_heading_into_oracle(input, &mut expected);
    assert_eq!(actual, expected, "input {input:?}");

    // `slugify_heading_into` appends. A prefix ending in `-` is the case the
    // trailing trim must not reach into, and a non-ASCII prefix checks that the
    // ASCII cursor starts from the real end of the buffer.
    for prefix in ["", "existing", "existing-", "-", "既存-"] {
        actual.clear();
        actual.push_str(prefix);
        expected.clear();
        expected.push_str(prefix);
        slugify_heading_into(input, &mut actual);
        slugify_heading_into_oracle(input, &mut expected);
        assert_eq!(actual, expected, "input {input:?} after prefix {prefix:?}");
    }
}

/// Deterministic xorshift, so a failure is reproducible from the test alone.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

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

#[test]
fn byte_cursor_slugify_matches_the_oracle_for_ascii() {
    // Every ASCII byte alone and in every ordered pair: covers the lowercase
    // fold, the separator collapse, the trailing trim, and the run boundaries.
    let mut input = String::new();
    for first in 0u8..0x80 {
        input.clear();
        input.push(first as char);
        assert_matches_oracle(&input);
        for second in 0u8..0x80 {
            input.clear();
            input.push(first as char);
            input.push(second as char);
            assert_matches_oracle(&input);
        }
    }
}

#[test]
fn byte_cursor_slugify_matches_the_oracle_for_empty_and_punctuation_only() {
    for input in [
        "",
        " ",
        "   ",
        "-",
        "---",
        "!!!",
        "?!.,;:",
        "&<>\"'",
        "\r\n",
        "\0\0",
        "   ---   ",
        "—",      // EM DASH: non-ASCII and not alphanumeric
        "……",     // HORIZONTAL ELLIPSIS
        "・「」", // CJK punctuation
    ] {
        assert_matches_oracle(input);
    }
}

#[test]
fn byte_cursor_slugify_matches_the_oracle_for_mixed_scripts() {
    for input in [
        "はじめに",
        "設定 Options & 既定値",
        "Über — Größe & Maß",
        "Ελληνικά ΚΕΦΑΛΑΙΑ",
        "Русский ЗАГОЛОВОК",
        "العربية عنوان",
        "עברית כותרת",
        "한국어 제목",
        "中文标题 API v2",
        "Emoji 🎉 in a heading 🚀 end",
        "\u{0130}stanbul",
        "\u{212A}elvin",
        "A\u{0130}B\u{212A}C",
        "a\u{0301}b",
        "\u{FF21}\u{FF22}\u{FF23} 123",
        "mixed\u{00E9}ascii\u{00E9}mixed",
        "\u{10400}\u{10428} deseret",
    ] {
        assert_matches_oracle(input);
    }
}

#[test]
fn byte_cursor_slugify_matches_the_oracle_for_very_long_headings() {
    assert_matches_oracle(&"Configuring the Renderer, Part 3: Options & Defaults! ".repeat(300));
    assert_matches_oracle(&"設定 Options & 既定値 — part 3 ".repeat(300));
    assert_matches_oracle(&"-".repeat(4096));
    assert_matches_oracle(&"a".repeat(4096));
    assert_matches_oracle(&"a-".repeat(2048));
    assert_matches_oracle(&"\u{0130}".repeat(1024));
}

#[test]
fn byte_cursor_slugify_matches_the_oracle_for_random_mixed_runs() {
    // Random strings drawn from an alphabet that forces frequent transitions
    // between the byte cursor and the Unicode path, including the scalars whose
    // lowercase mapping expands past their encoded length.
    const ALPHABET: [char; 24] = [
        'a',
        'Z',
        '0',
        '9',
        '-',
        ' ',
        '_',
        '.',
        '&',
        '<',
        '\r',
        '\n',
        'é',
        'É',
        'ß',
        'ẞ',
        'İ',
        'K',
        'は',
        '中',
        '🚀',
        '\u{0301}',
        '\u{FF21}',
        '\u{10400}',
    ];
    let mut rng = Rng(0x5EED_1234_ABCD_9876);
    let mut input = String::new();
    for _ in 0..20_000 {
        input.clear();
        let len = (rng.next() % 40) as usize;
        for _ in 0..len {
            let index = (rng.next() % ALPHABET.len() as u64) as usize;
            input.push(ALPHABET[index]);
        }
        assert_matches_oracle(&input);
    }
}

#[test]
fn byte_cursor_slugify_matches_the_oracle_for_every_sampled_scalar() {
    // Sampled sweep of the whole scalar range, each scalar surrounded by ASCII
    // so the ASCII/Unicode run handoff is exercised in both directions.
    let mut input = String::new();
    for code in (0u32..=0x0010_FFFF).step_by(53) {
        let Some(ch) = char::from_u32(code) else {
            continue;
        };
        input.clear();
        input.push_str("A ");
        input.push(ch);
        input.push_str(" b");
        input.push(ch);
        assert_matches_oracle(&input);
    }
}

#[test]
fn heading_id_planner_skips_taken_suffixes_and_deduplicates_explicit_ids() {
    let mut planner = HeadingIdPlanner::new();
    let ids = ["a", "a", "a-1", "a-1", "b", "b"].map(|base| planner.plan(base));

    assert_eq!(ids, ["a", "a-1", "a-1-1", "a-1-2", "b", "b-1"]);

    planner.clear();
    assert_eq!(planner.plan("a"), "a");
    assert_eq!(planner.plan("日本語"), "日本語");
    assert_eq!(planner.plan("日本語"), "日本語-1");
}

#[test]
fn permalink_marker_matches_the_emitted_id_however_the_prefix_splits_it() {
    // The renderer passes the configured prefix and the planned ID separately
    // instead of concatenating them. Every split of the emitted ID has to
    // answer like the whole ID with an empty prefix, which is the plain
    // `url == "#" + id` comparison.
    for (source, emitted, expected) in [
        ("## Hello [#](#docs-hello)", "docs-hello", true),
        ("## Hello *[#](#docs-hello)*", "docs-hello", true),
        ("## Hello [#](#hello)", "docs-hello", false),
        ("## Hello [#](#docs-hello-1)", "docs-hello", false),
        ("## Hello [#](#docs-hell)", "docs-hello", false),
        ("## Hello [#](docs-hello)", "docs-hello", false),
        ("## Hello [x](#docs-hello)", "docs-hello", false),
        ("## はじめに [#](#p_はじめに)", "p_はじめに", true),
        (
            "## Hello <a class=\"header-anchor\">#</a>",
            "docs-hello",
            true,
        ),
        ("## Hello", "docs-hello", false),
    ] {
        let allocator = Allocator::new();
        let document = Parser::new(&allocator, source).parse().unwrap();
        let [Node::Heading(heading)] = &document.children[..] else {
            panic!("{source:?} is not a single heading");
        };
        assert_eq!(
            heading_has_permalink_marker(&heading.children, "", emitted),
            expected,
            "{source:?}"
        );
        for split in (0..=emitted.len()).filter(|&split| emitted.is_char_boundary(split)) {
            let (prefix, id) = emitted.split_at(split);
            assert_eq!(
                heading_has_permalink_marker(&heading.children, prefix, id),
                expected,
                "{source:?} split as {prefix:?} + {id:?}"
            );
        }
    }
}
