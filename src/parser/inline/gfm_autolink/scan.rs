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
///
/// On aarch64 the block parse answers this out of its marker walk instead
/// ([`AutolinkFacts`]); this separate pass remains the path of the other
/// targets and the reference the tracked answer is tested against.
pub(in crate::parser::inline) fn may_contain_autolink(content: &str) -> Option<AutolinkScan> {
    let bytes = content.as_bytes();
    // One pass over `@` and `:` settles the email separator, `://`,
    // `mailto:`, and `xmpp:` together; only `www.` still needs its own
    // substring search, and only when no `@` already keeps the pass on.
    // Colons are rare in prose, so the per-hit compares stay cheap, while
    // a pass over `.` would stop at every sentence.
    let mut has_at = false;
    let mut has_extended_scheme = false;
    let mut has_bare_scheme = false;
    for at in memchr::memchr2_iter(b'@', b':', bytes) {
        if bytes[at] == b'@' {
            has_at = true;
            continue;
        }
        if bytes[at + 1..].starts_with(b"//") {
            has_bare_scheme |= !scheme_is_markdown_destination(bytes, at);
        }
        let prefix = &bytes[..at];
        has_extended_scheme |= prefix.ends_with(b"mailto") || prefix.ends_with(b"xmpp");
    }
    let may_have_www = has_at || WWW_FINDER.find(bytes).is_some();
    // Both extended schemes require an email separator.
    let may_have_extended = has_at && has_extended_scheme;
    (may_have_www || may_have_extended || has_bare_scheme).then_some(AutolinkScan {
        may_have_www,
        may_have_extended,
    })
}

/// [`may_contain_autolink`], answered one byte position at a time while the
/// inline marker scan walks the block, so the pre-flight needs no pass of its
/// own over the text the marker scan already classifies.
///
/// Every fact the pre-flight derives belongs to one byte of the raw content,
/// and each is a pure function of that position:
///
/// - an `@` sets `has_at`;
/// - a `:` sets the bare-scheme and extended-scheme facts from the bytes
///   around it — exactly the per-colon body of the pre-flight's loop;
/// - a `.` preceded by `www` is where a `www.` needle ends, which is what the
///   pre-flight's substring search finds.
///
/// Those are the *trigger* bytes. Nothing else can change the answer, so the
/// answer is [`may_contain_autolink`]'s — value for value, flags included —
/// once every trigger position of the content has been visited. Visiting one
/// twice changes nothing: every fact only ever turns on.
///
/// `seen` is the watermark that makes "every position" checkable: every
/// trigger before it has been visited. The fused marker scan visits the
/// triggers of the text it classifies and moves the watermark behind the
/// marker it stops at; the bytes a construct then consumes (a code span, a
/// link destination) are visited by [`Self::fill_to`] before the next scan
/// starts, and [`Self::finish`] visits whatever is left at the end.
#[derive(Clone, Copy)]
pub(in crate::parser::inline) struct AutolinkFacts {
    seen: usize,
    has_at: bool,
    has_www: bool,
    has_bare_scheme: bool,
    has_extended_scheme: bool,
}

impl AutolinkFacts {
    pub(in crate::parser::inline) const fn new() -> Self {
        Self {
            seen: 0,
            has_at: false,
            has_www: false,
            has_bare_scheme: false,
            has_extended_scheme: false,
        }
    }

    /// Every trigger before this position has been visited.
    #[inline]
    pub(in crate::parser::inline) const fn seen(&self) -> usize {
        self.seen
    }

    /// Records that every trigger before `upto` has been visited.
    #[inline]
    pub(in crate::parser::inline) fn advance_seen(&mut self, upto: usize) {
        self.seen = self.seen.max(upto);
    }

    /// Visits every trigger in `seen..upto` and moves the watermark there.
    #[inline]
    pub(in crate::parser::inline) fn fill_to(&mut self, bytes: &[u8], upto: usize) {
        if upto > self.seen {
            let from = self.seen;
            self.seen = upto;
            crate::parser::inline::scan::visit_autolink_triggers(bytes, from, upto, self);
        }
    }

    /// Applies the pre-flight's rule to the byte at `at`.
    ///
    /// Only trigger bytes set anything; the `:` arm is the per-colon body of
    /// [`may_contain_autolink`]'s loop, and the `.` arm is its `www.` search
    /// seen from the needle's last byte.
    #[inline]
    pub(in crate::parser::inline) fn visit(&mut self, bytes: &[u8], at: usize) {
        match bytes[at] {
            b'@' => self.has_at = true,
            b':' => {
                if bytes[at + 1..].starts_with(b"//") {
                    self.has_bare_scheme |= !scheme_is_markdown_destination(bytes, at);
                }
                let prefix = &bytes[..at];
                self.has_extended_scheme |=
                    prefix.ends_with(b"mailto") || prefix.ends_with(b"xmpp");
            }
            b'.' => self.has_www |= at >= 3 && bytes[at - 3..at] == *b"www",
            _ => {}
        }
    }

    /// The four facts, for tests that compare two ways of visiting.
    #[cfg(test)]
    pub(in crate::parser::inline) const fn facts(&self) -> [bool; 4] {
        [
            self.has_at,
            self.has_www,
            self.has_bare_scheme,
            self.has_extended_scheme,
        ]
    }

    /// Visits the rest of the content and answers the pre-flight.
    pub(in crate::parser::inline) fn finish(&mut self, bytes: &[u8]) -> Option<AutolinkScan> {
        self.fill_to(bytes, bytes.len());
        // The same combination as `may_contain_autolink`.
        let may_have_www = self.has_at || self.has_www;
        let may_have_extended = self.has_at && self.has_extended_scheme;
        (may_have_www || may_have_extended || self.has_bare_scheme).then_some(AutolinkScan {
            may_have_www,
            may_have_extended,
        })
    }
}

/// True when the `://` at `colon_slash_slash` completes a Markdown link
/// destination such as `](http://`, which the inline parser already
/// consumed, rather than a bare URL.
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
    SCHEMES
        .iter()
        .find(|name| bytes[..at].ends_with(name.as_bytes()))
        .map(|name| name.len() + 3)
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
        || domain
            .rsplit('.')
            .take(2)
            .any(|segment| segment.is_empty() || segment.contains('_'))
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
    // The parentheses are counted once and the count of closers is lowered
    // as they are stripped: no other byte this loop removes is a
    // parenthesis, so the counts stay right without rescanning the
    // candidate for every `)` — which made `http://x/` + `)`×n quadratic.
    let mut parens: Option<(usize, usize)> = None;
    loop {
        if end <= start {
            return end;
        }
        match bytes[end - 1] {
            b'?' | b'!' | b'.' | b',' | b':' | b'*' | b'_' | b'~' | b'\'' | b'"' => end -= 1,
            b')' => {
                let (opens, closes) = *parens.get_or_insert_with(|| {
                    let candidate = &bytes[start..end];
                    (
                        memchr::memchr_iter(b'(', candidate).count(),
                        memchr::memchr_iter(b')', candidate).count(),
                    )
                });
                if closes > opens {
                    end -= 1;
                    parens = Some((opens, closes - 1));
                } else {
                    return end;
                }
            }
            b';' => {
                // Strip an entity-like `&name;` suffix entirely. The name is
                // alphanumeric, so walking back over it bounds the search
                // for its `&` to the entity itself.
                let mut name_start = end - 1;
                while name_start > start && bytes[name_start - 1].is_ascii_alphanumeric() {
                    name_start -= 1;
                }
                if name_start < end - 1 && name_start > start && bytes[name_start - 1] == b'&' {
                    end = name_start - 1;
                } else {
                    end -= 1;
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
    Some(Candidate {
        start,
        end,
        href_prefix: "mailto:",
    })
}

fn is_email_local_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+')
}

#[cfg(test)]
mod tests {
    // Owned strings keep the test oracle independent of production arena storage.
    #![allow(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]

    use super::*;

    /// The original three-search pre-flight, kept as the oracle.
    fn reference(content: &str) -> Option<(bool, bool)> {
        let bytes = content.as_bytes();
        let has_at = memchr::memchr(b'@', bytes).is_some();
        let may_have_www = has_at || WWW_FINDER.find(bytes).is_some();
        let may_have_extended =
            has_at && (MAILTO_FINDER.find(bytes).is_some() || XMPP_FINDER.find(bytes).is_some());
        let mut has_bare_scheme = false;
        let mut from = 0;
        while let Some(offset) = SCHEME_FINDER.find(&bytes[from..]) {
            let at = from + offset;
            if !scheme_is_markdown_destination(bytes, at) {
                has_bare_scheme = true;
                break;
            }
            from = at + 3;
        }
        (may_have_www || may_have_extended || has_bare_scheme)
            .then_some((may_have_www, may_have_extended))
    }

    fn check(content: &str) {
        let actual =
            may_contain_autolink(content).map(|scan| (scan.may_have_www, scan.may_have_extended));
        assert_eq!(actual, reference(content), "input: {content:?}");

        // The position-by-position form: one bulk visit of the whole
        // content, and the scalar rule applied at every single position.
        let bulk = AutolinkFacts::new()
            .finish(content.as_bytes())
            .map(|scan| (scan.may_have_www, scan.may_have_extended));
        assert_eq!(bulk, reference(content), "tracked, input: {content:?}");
        let mut each = AutolinkFacts::new();
        for at in 0..content.len() {
            each.visit(content.as_bytes(), at);
        }
        each.advance_seen(content.len());
        let each = each
            .finish(content.as_bytes())
            .map(|scan| (scan.may_have_www, scan.may_have_extended));
        assert_eq!(each, reference(content), "per position, input: {content:?}");
    }

    #[test]
    fn preflight_matches_reference_on_fixtures() {
        for case in [
            "",
            "plain prose. With sentences: and colons.",
            "see www.example.com",
            "WWW.example.com",
            "www",
            "http://example.com",
            "[text](http://example.com)",
            "[text](HTTPS://example.com) and http://bare.example",
            "[text](ftp://x) [y](mailto://z)",
            "mailto:",
            "mailto:user@example.com",
            "xmpp:user@example.com",
            "user@example.com",
            "mailto://x",
            "a:b://c",
            "://",
            ":",
            ".",
            "@",
            "www.:",
            "ends with www",
            "ends with mailto",
            "www\\.example.com",
            "ww.w.",
            "wwww.",
            ".www.",
            "a longer line of prose that crosses one vector: www.example.com",
            "a longer line of prose that crosses one vector: ends with www",
            "a longer line of prose that crosses one vector: http://x.y",
            "a longer line of prose [text](https://example.com) and no bare one",
            "a longer line of prose and an address user@example.com in it",
            "a longer line of prose with mailto:user@example.com in it",
        ] {
            check(case);
        }
    }

    #[test]
    fn preflight_matches_reference_on_mixed_inputs() {
        let tokens = [
            "www.",
            "WWW.",
            "://",
            "](http://",
            "](https://",
            "](ftp://",
            "](HTTP://",
            "mailto:",
            "xmpp:",
            "mailto://",
            "@",
            ".",
            ":",
            "a",
            " ",
            "www",
            "http",
            "mailto",
            "xmpp",
            "]",
            "(",
        ];
        let mut state = 0x7a3c_5e1d_9b2f_4681u64;
        for _ in 0..4000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let len = (state >> 33) as usize % 24;
            let mut input = String::new();
            for _ in 0..len {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                input.push_str(tokens[(state >> 33) as usize % tokens.len()]);
            }
            check(&input);
        }
    }
}
