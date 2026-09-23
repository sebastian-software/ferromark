//! URL-specific regression cases and the existing scalar reference.

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
use super::super::tests::assert_lanes_match_flags;
use super::super::tests::assert_scan_matches_flags;
use super::super::write_url_escaped_into;
use super::*;

pub(in crate::renderer::html::escape) fn reference_url(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        if byte >= 0x80 {
            push_percent_byte(&mut out, byte);
        } else if URL_ESCAPE_FLAG[byte as usize] != 0 {
            out.push_str(URL_ESCAPE_TABLE[byte as usize]);
        } else {
            out.push(byte as char);
        }
    }
    out
}

#[test]
fn url_escape_matches_reference_at_every_offset_and_tail() {
    for needle in ["é", "中", "🙂", "é中🙂", "&", "[", "%", " ", "`", "a"] {
        for offset in 0..=40 {
            for tail in [0, 1, 7, 8, 9, 15, 16, 17, 31, 32, 33] {
                let source = format!("{}{}{}", "u".repeat(offset), needle, "v".repeat(tail));
                let mut actual = String::new();
                write_url_escaped_into(&mut actual, &source);
                assert_eq!(actual, reference_url(&source), "source: {source:?}");
            }
        }
    }
}

#[test]
fn commonmark_url_syntax_bytes_are_percent_encoded() {
    for (source, expected) in [
        (r"foo\bar", "foo%5Cbar"),
        ("https://foo.bar.`baz", "https://foo.bar.%60baz"),
        (
            "https://example.com/?search=][ref]",
            "https://example.com/?search=%5D%5Bref%5D",
        ),
        ("https://example.com/\\[\\", "https://example.com/%5C%5B%5C"),
    ] {
        let mut actual = String::new();
        write_url_escaped_into(&mut actual, source);
        assert_eq!(actual, expected, "source: {source:?}");
    }
}

#[test]
fn url_escape_handles_escape_heavy_ascii_before_unicode_linearly() {
    let mut source = String::with_capacity(4097);
    for _ in 0..2048 {
        source.push('&');
    }
    source.push('é');

    let mut actual = String::new();
    write_url_escaped_into(&mut actual, &source);
    assert_eq!(actual, reference_url(&source));
}

#[test]
fn url_scan_matches_flag_table_across_vector_steps_and_tails() {
    // The URL-specific members, then non-ASCII bytes from both ends of the
    // high half and a few leading bytes, which the scan meets inside UTF-8.
    assert_scan_matches_flags(
        url_escape_mask,
        &URL_ESCAPE_FLAG,
        UrlEscapeNeedles,
        b" \"&<>[\\]`\x80\xBF\xC3\xE4\xF0\xFF",
    );
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[test]
fn url_sse2_classifier_matches_flag_table_in_every_lane() {
    assert_lanes_match_flags(UrlEscapeNeedles, &URL_ESCAPE_FLAG);
}

#[test]
fn invalid_ipv6_authority_is_escaped_without_html_injection() {
    let source = r#"https://[::1\"onerror=\"x]/"#;
    let mut actual = String::new();
    write_url_escaped_into(&mut actual, source);
    assert_eq!(actual, "https://%5B::1%5C%22onerror=%5C%22x%5D/");
}
