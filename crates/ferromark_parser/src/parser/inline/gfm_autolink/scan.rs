//! Needle searches for the GFM autolink post-pass.
//!
//! Everything here is a pure scan over one string: the block-level
//! pre-flight that decides whether the post-pass runs at all, and the
//! per-text-node search for the earliest valid candidate.

use std::sync::LazyLock;

use memchr::memmem;

use super::{AutolinkScan, Candidate};

/// Searchers for the multi-byte autolink needles, built once for the process.
///
/// The one-shot `memmem::find` rebuilds its SIMD prefilter on every call, and
/// this scan runs over every text node of every document — on short prose
/// nodes that setup cost dominated the search itself.
pub(super) static WWW_FINDER: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("www."));

/// The three schemes share one needle: `http://`, `https://` and `ftp://`
/// all end in `://` and differ only in the name in front. One pass for
/// `://` therefore finds every scheme candidate, in position order, in
/// place of three separate whole-node substring scans.
pub(super) static SCHEME_FINDER: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("://"));

pub(super) static MAILTO_FINDER: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("mailto:"));
pub(super) static XMPP_FINDER: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("xmpp:"));

/// Scheme names accepted in front of `://`, longest first so `https://`
/// is not mistaken for a `ttp`-suffixed shorter name.
const SCHEMES: [&str; 3] = ["https", "http", "ftp"];

/// Length of the longest name in [`SCHEMES`], which is how far in front of
/// a `://` a candidate can begin.
pub(super) const LONGEST_SCHEME: usize = 5;

/// Cheap pre-flight over a block's raw inline content: can the autolink
/// pass possibly rewrite anything here, and if so does it need the `www.`
/// search at all?
///
/// Every candidate needs `://` (scheme), `@` (email), or `www.` — and none
/// of those bytes are inline-special, so if they appear in the parsed text
/// nodes they appear verbatim in `content` too. `&` used to keep the pass
/// on for entity-decoded needles, but that made every Rust-doc paragraph
/// containing `` `&str` `` pay the coalesce + rewrite walk. GFM spec
/// examples always include a verbatim `www.` / `://` / `@` alongside `&`
/// in a URL.
///
/// `://` inside a markdown destination (`](https://…)`) cannot become a
/// GFM autolink — the inline parser already turned it into a Link — so it
/// does not keep the pass on.
pub(in crate::parser::inline) fn may_contain_autolink(content: &str) -> Option<AutolinkScan> {
    let bytes = content.as_bytes();
    let has_at = memchr::memchr(b'@', bytes).is_some();
    let may_have_www = has_at || WWW_FINDER.find(bytes).is_some();
    // Both extended schemes require an email separator. Avoid two substring
    // scans on ordinary prose that cannot contain a valid extended address.
    let may_have_extended =
        has_at && (MAILTO_FINDER.find(bytes).is_some() || XMPP_FINDER.find(bytes).is_some());
    (may_have_www || may_have_extended || has_bare_scheme(bytes))
        .then_some(AutolinkScan { may_have_www, may_have_extended })
}

/// True when `://` appears as a bare URL, not only as `](http://…)` /
/// `](https://…)` / `](ftp://…)`.
fn has_bare_scheme(bytes: &[u8]) -> bool {
    let mut from = 0;
    while let Some(offset) = SCHEME_FINDER.find(&bytes[from..]) {
        let at = from + offset;
        if !scheme_is_markdown_destination(bytes, at) {
            return true;
        }
        from = at + 3;
    }
    false
}

fn scheme_is_markdown_destination(bytes: &[u8], colon_slash_slash: usize) -> bool {
    for name in SCHEMES {
        let Some(start) = colon_slash_slash.checked_sub(name.len()) else {
            continue;
        };
        if start >= 2
            && bytes[start - 2] == b']'
            && bytes[start - 1] == b'('
            && bytes[start..colon_slash_slash].eq_ignore_ascii_case(name.as_bytes())
        {
            return true;
        }
    }
    false
}

/// Length of the whole `scheme://` prefix ending at the `://` that starts
/// at `at`, or `None` when the bytes in front are not a known scheme.
pub(super) fn scheme_prefix_len(bytes: &[u8], at: usize) -> Option<usize> {
    SCHEMES.iter().find(|name| bytes[..at].ends_with(name.as_bytes())).map(|name| name.len() + 3)
}

/// Start-of-text, whitespace, or common delimiter punctuation may precede an
/// autolink.
fn valid_boundary(value: &str, start: usize) -> bool {
    value[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| ch.is_whitespace() || matches!(ch, '*' | '_' | '~' | '(' | '\'' | '"'))
}

pub(super) fn validate_url(value: &str, start: usize, prefix_len: usize) -> Option<Candidate> {
    if !valid_boundary(value, start) {
        return None;
    }
    let bytes = value.as_bytes();
    // Validate the domain: alphanumerics, `-`, `_`, `.`; at least one
    // dot; no underscore in the last two segments.
    let domain_start = start + prefix_len;
    let mut domain_end = domain_start;
    while domain_end < bytes.len()
        && (bytes[domain_end].is_ascii_alphanumeric()
            || matches!(bytes[domain_end], b'-' | b'_' | b'.'))
    {
        domain_end += 1;
    }
    // Trailing dots belong to the surrounding sentence, not the domain.
    let domain = value[domain_start..domain_end].trim_end_matches('.');
    if domain.split('.').count() < 2
        || domain.rsplit('.').take(2).any(|segment| segment.is_empty() || segment.contains('_'))
    {
        return None;
    }

    // The link runs to whitespace, `<`, or CJK sentence punctuation, then
    // trailing punctuation is trimmed (unbalanced `)` and entity-like `&x;`
    // suffixes included).
    let end = trim_trailing_punctuation(value, start, scan_url_end(value, domain_end));
    (end > domain_start).then_some(Candidate {
        start,
        end,
        href_prefix: if prefix_len == 4 { "http://" } else { "" },
    })
}

/// Extends a URL from `from` to the offset where it stops.
///
/// Non-ASCII characters normally belong to the URL — an IRI carries them
/// verbatim (`/wiki/日本語`) — with CJK sentence punctuation the exception.
fn scan_url_end(value: &str, from: usize) -> usize {
    let bytes = value.as_bytes();
    let mut end = from;
    while end < bytes.len() {
        let byte = bytes[end];
        if byte.is_ascii() {
            if byte.is_ascii_whitespace() || byte == b'<' {
                break;
            }
            end += 1;
            continue;
        }
        let Some(ch) = value.get(end..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        if ends_url(ch) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

/// Punctuation that ends a bare URL the way ASCII whitespace does.
///
/// The GFM autolink extension defines trailing-punctuation trimming for
/// ASCII only, and CJK prose puts no space between a URL and the `。` that
/// closes the sentence — so without this the rest of the sentence is
/// swallowed into the link.
///
/// Mirrored by the bare-URL scanner in `ferromark_renderer`.
const fn ends_url(ch: char) -> bool {
    matches!(
        ch,
        // CJK symbols and punctuation: the ideographic space, 、。〈〉《》
        // 「」『』【】 and friends.
        '\u{3000}'..='\u{303F}'
        // Fullwidth ！＂＃＄％＆＇（）＊＋，－．／
        | '\u{FF01}'..='\u{FF0F}'
        // Fullwidth ：；＜＝＞？＠
        | '\u{FF1A}'..='\u{FF20}'
        // Fullwidth ［＼］＾＿｀
        | '\u{FF3B}'..='\u{FF40}'
        // Fullwidth ｛｜｝～｟｠ and halfwidth ｡｢｣､･
        | '\u{FF5B}'..='\u{FF65}'
    )
}

fn trim_trailing_punctuation(value: &str, start: usize, mut end: usize) -> usize {
    let bytes = value.as_bytes();
    loop {
        if end <= start {
            return end;
        }
        match bytes[end - 1] {
            b'?' | b'!' | b'.' | b',' | b':' | b'*' | b'_' | b'~' | b'\'' | b'"' => end -= 1,
            b')' => {
                let opens = value[start..end].bytes().filter(|&b| b == b'(').count();
                let closes = value[start..end].bytes().filter(|&b| b == b')').count();
                if closes > opens {
                    end -= 1;
                } else {
                    return end;
                }
            }
            b';' => {
                // Strip an entity-like `&name;` suffix entirely.
                let entity_start = value[start..end - 1].rfind('&').map(|found| start + found);
                match entity_start {
                    Some(amp)
                        if value[amp + 1..end - 1]
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric())
                            && amp + 1 < end - 1 =>
                    {
                        end = amp;
                    }
                    _ => end -= 1,
                }
            }
            _ => return end,
        }
    }
}

pub(super) fn validate_email(value: &str, at: usize) -> Option<Candidate> {
    let candidate = validate_email_parts(value, at)?;
    valid_boundary(value, candidate.start).then_some(candidate)
}

pub(super) fn validate_extended_email(
    value: &str,
    start: usize,
    prefix: &str,
    xmpp: bool,
) -> Option<Candidate> {
    if !valid_boundary(value, start)
        || !value
            .get(start..)?
            .as_bytes()
            .get(..prefix.len())?
            .eq_ignore_ascii_case(prefix.as_bytes())
    {
        return None;
    }
    let body_start = start + prefix.len();
    let mut at = body_start;
    while at < value.len() && is_email_local_byte(value.as_bytes()[at]) {
        at += 1;
    }
    if value.as_bytes().get(at) != Some(&b'@') {
        return None;
    }
    let mut candidate = validate_email_parts(value, at)?;
    // The address must begin immediately after the scheme. Without this
    // check, a later address in `mailto: prose a@b.example` could make the
    // malformed prefix into a link.
    if candidate.start != body_start {
        return None;
    }
    candidate.start = start;
    candidate.href_prefix = "";
    if xmpp && value.as_bytes().get(candidate.end) == Some(&b'/') {
        let resource_start = candidate.end + 1;
        let mut resource_end = resource_start;
        while resource_end < value.len() {
            let byte = value.as_bytes()[resource_end];
            if !(byte.is_ascii_alphanumeric() || byte == b'@' || byte == b'.') {
                break;
            }
            resource_end += 1;
        }
        let end = trim_trailing_punctuation(value, start, resource_end);
        if end > resource_start {
            candidate.end = end;
        }
    }
    Some(candidate)
}

fn validate_email_parts(value: &str, at: usize) -> Option<Candidate> {
    let bytes = value.as_bytes();
    // Local part: alphanumerics plus `.`, `-`, `_`, `+`.
    let mut start = at;
    while start > 0 {
        let byte = bytes[start - 1];
        if is_email_local_byte(byte) {
            start -= 1;
        } else {
            break;
        }
    }
    if start == at {
        return None;
    }

    // Domain: alphanumerics plus `.`, `-`, `_`, with at least one dot;
    // trailing dots are trimmed; a trailing `-` or `_` invalidates.
    let mut end = at + 1;
    while end < bytes.len()
        && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'.' | b'-' | b'_'))
    {
        end += 1;
    }
    while end > at + 1 && bytes[end - 1] == b'.' {
        end -= 1;
    }
    if end <= at + 1 || matches!(bytes[end - 1], b'-' | b'_') {
        return None;
    }
    if !value[at + 1..end].contains('.') {
        return None;
    }
    Some(Candidate { start, end, href_prefix: "mailto:" })
}

fn is_email_local_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+')
}
