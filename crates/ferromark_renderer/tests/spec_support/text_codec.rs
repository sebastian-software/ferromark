//! Entity, percent-encoding, and canonical re-encoding helpers shared by
//! the spec HTML normalizer (`normalize.rs`).

/// Decodes the entity spellings cmark emits plus numeric references.
/// Unknown named entities are kept verbatim (both sides see the same text).
pub fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        rest = &rest[pos..];
        let Some(semicolon) = rest.as_bytes()[..rest.len().min(40)]
            .iter()
            .position(|&byte| byte == b';')
        else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let entity = &rest[1..semicolon];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{a0}'),
            _ => decode_numeric_entity(entity),
        };
        if let Some(ch) = decoded {
            out.push(ch);
            rest = &rest[semicolon + 1..];
        } else {
            out.push('&');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

fn decode_numeric_entity(entity: &str) -> Option<char> {
    let digits = entity.strip_prefix('#')?;
    let code = if let Some(hex) = digits.strip_prefix(['x', 'X']) {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        digits.parse::<u32>().ok()?
    };
    // The spec maps U+0000 and out-of-range references to U+FFFD.
    Some(
        char::from_u32(code)
            .filter(|&c| c != '\0')
            .unwrap_or('\u{fffd}'),
    )
}

/// Canonicalizes non-ASCII UTF-8 URL spelling without decoding ASCII escapes.
/// A literal `#`, slash, backslash, or percent sign must never become equivalent
/// to its percent-encoded spelling merely because both appear in an attribute.
pub fn normalize_url(input: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if byte.is_ascii() {
            out.push(char::from(byte));
        } else {
            write!(&mut out, "%{byte:02X}").unwrap();
        }
    }
    out
}

pub fn encode_text_into(out: &mut String, text: &str) {
    for ch in text.chars() {
        encode_char_into(out, ch);
    }
}

pub fn encode_char_into(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        _ => out.push(ch),
    }
}

pub fn encode_attr_into(out: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

#[test]
fn decodes_common_and_numeric_entities() {
    assert_eq!(
        decode_entities("a &amp; b &#35; &#x22; &nbsp;"),
        "a & b # \" \u{a0}"
    );
    assert_eq!(decode_entities("&copy; stays"), "&copy; stays");
    assert_eq!(decode_entities("&#0; becomes"), "\u{fffd} becomes");
    assert_eq!(decode_entities("bare & alone"), "bare & alone");
}

#[test]
fn url_normalization_preserves_ascii_escapes() {
    assert_eq!(normalize_url("/a%20b"), "/a%20b");
    assert_eq!(normalize_url("/café"), "/caf%C3%A9");
    assert_eq!(normalize_url("%C3%A9"), "%C3%A9");
    assert_eq!(normalize_url("50%"), "50%");
    assert_eq!(normalize_url("%2520"), "%2520");
}

#[test]
fn entity_probe_never_splits_utf8() {
    let text = format!("&{}", "é".repeat(30));
    assert_eq!(decode_entities(&text), text);
}
