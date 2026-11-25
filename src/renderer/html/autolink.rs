//! Bare-URL autolink scanner.
//!
//! Text nodes can optionally turn recognized URL prefixes into anchors. The scanner
//! first indexes possible leading bytes so most prose is skipped without repeated
//! prefix checks, then validates word boundaries and trims punctuation around matches.

use std::ops::Range;

mod index;

pub(in crate::renderer::html) use index::FirstByteIndex;

/// Scans `s` from `from` for the next position that begins one of the
/// registered URL prefixes at a word boundary, and returns the
/// `(match_start, url_end)` byte range with trailing punctuation trimmed.
///
/// The boundary rule mirrors common autolinkers: a match is only accepted
/// when the preceding byte (if any) is not an ASCII alphanumeric — so
/// `"see http://x"` matches but `"shttp://x"` doesn't. The URL extends to
/// the next whitespace, `<`, `>`, `"`, `'`, or backtick, and we then strip
/// trailing `.,;:!?` plus an unbalanced `)`, `]`, or `}`.
///
/// `index` skips ahead to the next byte that could start a pattern, so the
/// per-byte boundary and prefix checks below only run at real candidates
/// rather than across every byte of non-URL prose.
pub(super) fn find_autolink_match<P: AsRef<str>>(
    s: &str,
    from: usize,
    patterns: &[P],
    index: &FirstByteIndex,
) -> Option<(usize, usize)> {
    if index.is_default_http() {
        return find_default_http_autolink_match(s, from);
    }
    let bytes = s.as_bytes();
    let mut base = from;
    while base < bytes.len() {
        let rel = index.next(&bytes[base..])?;
        let i = base + rel;
        // One-byte look-ahead: most prose hits on a frequent candidate byte
        // ("the", "words"…) die here before any prefix comparison.
        if index.rejects_second(bytes[i], bytes.get(i + 1)) {
            base = i + 1;
            continue;
        }
        // Word boundary: the previous byte must not be ASCII alphanumeric.
        let is_boundary = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if is_boundary {
            for pat in patterns {
                let pat = pat.as_ref();
                let pat_bytes = pat.as_bytes();
                if pat_bytes.is_empty() {
                    continue;
                }
                if i + pat_bytes.len() <= bytes.len()
                    && bytes[i..i + pat_bytes.len()].eq_ignore_ascii_case(pat_bytes)
                {
                    let url_start = i;
                    let prefix_end = i + pat_bytes.len();
                    let url_end = scan_url_end(s, prefix_end);
                    // Require at least one byte beyond the scheme/prefix after
                    // trimming trailing punctuation so `"http://."` or `"mailto:."`
                    // is never autolinked, and punctuation-only custom prefixes
                    // cannot trim to an empty match and loop forever.
                    if url_end == prefix_end {
                        continue;
                    }
                    let trimmed_end = trim_trailing_punct(bytes, url_start, url_end);
                    if trimmed_end <= prefix_end {
                        continue;
                    }
                    return Some((url_start, trimmed_end));
                }
            }
        }
        base = i + 1;
    }
    None
}

/// Anchored fast path for the default `["http://", "https://"]` patterns.
///
/// Jumps directly to `://` via SIMD `memmem` instead of visiting every `'h'`
/// and `'H'` in surrounding prose, then checks the 4- or 5-byte scheme and
/// word boundary immediately preceding `://`.
pub(super) fn find_default_http_autolink_match(s: &str, from: usize) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut search = from.saturating_add(4);
    let mut rejected_schemes = 0u32;
    while search < bytes.len() {
        let rel = FirstByteIndex::find_colon_slash_slash(&bytes[search..])?;
        let colon = search + rel;
        let prefix_end = colon + 3;
        let url_start =
            if colon >= from + 5 && bytes[colon - 5..colon].eq_ignore_ascii_case(b"https") {
                Some(colon - 5)
            } else if colon >= from + 4 && bytes[colon - 4..colon].eq_ignore_ascii_case(b"http") {
                Some(colon - 4)
            } else {
                None
            };
        if let Some(url_start) = url_start {
            rejected_schemes = 0;
            let is_boundary = url_start == 0 || !bytes[url_start - 1].is_ascii_alphanumeric();
            if is_boundary {
                let url_end = scan_url_end(s, prefix_end);
                if url_end > prefix_end {
                    let trimmed_end = trim_trailing_punct(bytes, url_start, url_end);
                    if trimmed_end > prefix_end {
                        return Some((url_start, trimmed_end));
                    }
                }
            }
            search = prefix_end;
            continue;
        }
        rejected_schemes += 1;
        if rejected_schemes >= 4 {
            // Input holds repeated non-HTTP `://` schemes (such as `ftp://`).
            // Skip straight to the next `ht` prefix via SIMD `memchr2` so
            // inputs with no `h`/`H` exit in one vector scan instead of
            // restarting on every `://`.
            let mut h_scan = prefix_end;
            let next_search = loop {
                let h_rel = memchr::memchr2(b'h', b'H', &bytes[h_scan..])?;
                let h = h_scan + h_rel;
                if bytes
                    .get(h + 1)
                    .is_some_and(|byte| byte.eq_ignore_ascii_case(&b't'))
                {
                    break h + 4;
                }
                h_scan = h + 1;
            };
            rejected_schemes = 0;
            search = next_search;
        } else {
            search = prefix_end;
        }
    }
    None
}

/// Reusable bare-URL matcher configured with the renderer's prefix rules.
///
/// Construct this once when a caller needs to scan several text nodes. The
/// matcher owns its configured prefixes and candidate index, so each scan
/// reuses the index while applying the same word-boundary and punctuation
/// rules as HTML rendering. The normal parser and renderer do not use this
/// public helper.
pub struct AutolinkMatcher {
    patterns: Vec<String>,
    index: FirstByteIndex,
}

impl AutolinkMatcher {
    /// Creates a matcher for the provided URL prefixes.
    #[must_use]
    pub fn new<P: AsRef<str>>(patterns: &[P]) -> Self {
        let patterns = patterns
            .iter()
            .map(|pattern| pattern.as_ref().to_owned())
            .collect::<Vec<_>>();
        let index = FirstByteIndex::from_patterns(&patterns);
        Self { patterns, index }
    }

    /// Finds the URL ranges in one text node.
    ///
    /// Call this separately for each AST text node when reproducing renderer
    /// behavior: autolinks never span node boundaries.
    #[must_use]
    pub fn find_ranges(&self, text: &str) -> Vec<Range<usize>> {
        find_autolink_ranges_with_index(text, &self.patterns, &self.index)
    }
}

/// Finds the byte ranges that the HTML renderer would recognize as bare URLs.
///
/// This explicit helper lets optional AST transforms protect the same URL
/// spellings without adding URL scanning to parsing or rendering. Each range
/// starts at the registered prefix and ends before trailing punctuation, using
/// the renderer's existing boundary and trimming rules. Calling this function
/// is the only work it performs; the ordinary parser and renderer never call it.
///
/// `patterns` uses the same prefix format as
/// [`crate::renderer::HtmlRendererOptions::autolink_patterns`].
#[must_use]
pub fn find_autolink_ranges<P: AsRef<str>>(s: &str, patterns: &[P]) -> Vec<Range<usize>> {
    let index = FirstByteIndex::from_patterns(patterns);
    find_autolink_ranges_with_index(s, patterns, &index)
}

fn find_autolink_ranges_with_index<P: AsRef<str>>(
    s: &str,
    patterns: &[P],
    index: &FirstByteIndex,
) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut cursor = 0;

    while let Some((start, end)) = find_autolink_match(s, cursor, patterns, index) {
        ranges.push(start..end);
        // Matches always consume at least the pattern and one URL byte, but
        // keep the public iterator robust if that recognizer contract changes.
        cursor = end.max(start + 1);
    }

    ranges
}

/// Extends a URL from `from` to the offset where it stops.
///
/// Non-ASCII characters normally belong to the URL — an IRI carries them
/// verbatim (`/wiki/日本語`) — with CJK sentence punctuation the exception.
fn scan_url_end(s: &str, from: usize) -> usize {
    let bytes = s.as_bytes();
    // An IRI host can start with Unicode immediately after its scheme. Avoid
    // SIMD setup when there is no ASCII prefix to skip.
    let mut end = if bytes.get(from).is_some_and(u8::is_ascii) {
        scan_ascii_url_prefix(bytes, from)
    } else {
        from
    };
    if end >= bytes.len() || bytes[end].is_ascii() {
        return end;
    }

    // A wide ASCII pass stops at the first high-bit byte. Resume the original
    // Unicode-aware walk there so IRI characters remain part of the URL while
    // CJK/fullwidth sentence punctuation still terminates it.
    while end < bytes.len() {
        let byte = bytes[end];
        if byte.is_ascii() {
            if !is_url_byte(byte) {
                break;
            }
            end += 1;
            continue;
        }
        let Some(ch) = s.get(end..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        if ends_url(ch) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

/// Nibble-pair lookup tables for `scan_ascii_url_prefix` on AArch64 NEON.
///
/// A byte stops the ASCII URL scan iff `LOW[b & 0x0F] & HIGH[b >> 4] != 0`:
/// - `0x01`: `\t` (0x09), `\n` (0x0A), `\r` (0x0D)
/// - `0x02`: `' '` (0x20), `'"'` (0x22), `'\''` (0x27)
/// - `0x04`: `'<'` (0x3C), `'>'` (0x3E)
/// - `0x08`: ``'`'`` (0x60)
/// - `0x10`: every non-ASCII byte (`0x80..=0xFF`)
#[cfg(target_arch = "aarch64")]
const URL_STOP_LOW: [u8; 16] = [
    0x1A, 0x10, 0x12, 0x10, 0x10, 0x10, 0x10, 0x12, 0x10, 0x11, 0x11, 0x10, 0x14, 0x11, 0x14, 0x10,
];
#[cfg(target_arch = "aarch64")]
const URL_STOP_HIGH: [u8; 16] = [
    0x01, 0x00, 0x02, 0x04, 0x00, 0x00, 0x08, 0x00, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
];

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn scan_ascii_url_prefix(bytes: &[u8], from: usize) -> usize {
    use std::arch::aarch64::*;

    let end = bytes.len();
    let mut i = from;
    // SAFETY: full vectors are loaded only when 16 bytes remain; the overlapping
    // tail starts at len - 16 and masks positions before the unconsumed cursor.
    // AArch64 guarantees NEON support. No pointer outlives the borrowed slice.
    unsafe {
        let low = vld1q_u8(URL_STOP_LOW.as_ptr());
        let high = vld1q_u8(URL_STOP_HIGH.as_ptr());
        let nibble = vdupq_n_u8(0x0F);
        let classify = |v: uint8x16_t| {
            let lo = vqtbl1q_u8(low, vandq_u8(v, nibble));
            let hi = vqtbl1q_u8(high, vshrq_n_u8(v, 4));
            let stop = vtstq_u8(lo, hi);
            let narrow = vshrn_n_u16(vreinterpretq_u16_u8(stop), 4);
            vget_lane_u64(vreinterpret_u64_u8(narrow), 0)
        };

        while i + 16 <= end {
            let mask = classify(vld1q_u8(bytes.as_ptr().add(i)));
            if mask != 0 {
                return i + (mask.trailing_zeros() / 4) as usize;
            }
            i += 16;
        }
        if i < end && end >= 16 {
            let base = end - 16;
            let mask =
                classify(vld1q_u8(bytes.as_ptr().add(base))) & (u64::MAX << ((i - base) * 4));
            if mask != 0 {
                return base + (mask.trailing_zeros() / 4) as usize;
            }
            return end;
        }
    }
    scan_ascii_url_prefix_scalar(bytes, i)
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[allow(unsafe_code)]
#[inline]
fn scan_ascii_url_prefix(bytes: &[u8], from: usize) -> usize {
    use std::arch::x86_64::*;

    let end = bytes.len();
    let mut i = from;
    // SAFETY: `cfg` guarantees SSE2, and every 16-byte load is bounded by
    // `i + 16 <= end` or reads the final full vector at `end - 16` when
    // `end >= 16`. `_mm_loadu_si128` has no alignment requirement.
    unsafe {
        let classify = |v: __m128i| -> u32 {
            // Bit-1 fold merges `' '` (0x20)/`'"'` (0x22) -> 0x22 and
            // `'<'` (0x3C)/`'>'` (0x3E) -> 0x3E; bit-2 fold merges
            // `'\t'` (0x09)/`'\r'` (0x0D) -> 0x0D. ORing `v` directly adds
            // bit 7 for all non-ASCII bytes (`>= 0x80`) before `movemask`.
            let or2 = _mm_or_si128(v, _mm_set1_epi8(0x02));
            let space_quote = _mm_cmpeq_epi8(or2, _mm_set1_epi8(0x22));
            let angle = _mm_cmpeq_epi8(or2, _mm_set1_epi8(0x3E));
            let tab_cr = _mm_cmpeq_epi8(_mm_or_si128(v, _mm_set1_epi8(0x04)), _mm_set1_epi8(0x0D));
            let lf = _mm_cmpeq_epi8(v, _mm_set1_epi8(0x0A));
            let apos = _mm_cmpeq_epi8(v, _mm_set1_epi8(0x27));
            let tick = _mm_cmpeq_epi8(v, _mm_set1_epi8(0x60));
            let stop = _mm_or_si128(
                _mm_or_si128(_mm_or_si128(space_quote, angle), _mm_or_si128(tab_cr, lf)),
                _mm_or_si128(_mm_or_si128(apos, tick), v),
            );
            _mm_movemask_epi8(stop).cast_unsigned()
        };

        while i + 16 <= end {
            let mask = classify(_mm_loadu_si128(bytes.as_ptr().add(i).cast()));
            if mask != 0 {
                return i + mask.trailing_zeros() as usize;
            }
            i += 16;
        }
        if i < end && end >= 16 {
            let base = end - 16;
            let mask = classify(_mm_loadu_si128(bytes.as_ptr().add(base).cast()))
                & (u32::MAX << (i - base));
            if mask != 0 {
                return base + mask.trailing_zeros() as usize;
            }
            return end;
        }
    }
    scan_ascii_url_prefix_scalar(bytes, i)
}

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "sse2")
)))]
#[inline]
fn scan_ascii_url_prefix(bytes: &[u8], from: usize) -> usize {
    scan_ascii_url_prefix_scalar(bytes, from)
}

#[inline]
fn scan_ascii_url_prefix_scalar(bytes: &[u8], from: usize) -> usize {
    let end = bytes.len();
    let mut i = from;
    while i + 8 <= end {
        let chunk = &bytes[i..i + 8];
        let mut stop = 0u8;
        for (offset, &byte) in chunk.iter().enumerate() {
            if !byte.is_ascii() || !is_url_byte(byte) {
                stop |= 1 << offset;
            }
        }
        if stop != 0 {
            return i + stop.trailing_zeros() as usize;
        }
        i += 8;
    }
    while i < end && bytes[i].is_ascii() && is_url_byte(bytes[i]) {
        i += 1;
    }
    i
}

#[inline]
fn is_url_byte(byte: u8) -> bool {
    !matches!(
        byte,
        b' ' | b'\t' | b'\n' | b'\r' | b'<' | b'>' | b'"' | b'\'' | b'`'
    )
}

/// Punctuation that ends a bare URL the way ASCII whitespace does.
///
/// Trailing-punctuation handling is defined for ASCII only, and CJK prose
/// puts no space between a URL and the `。` that closes the sentence — so
/// without this the rest of the sentence is swallowed into the link.
///
/// Mirrored by the GFM autolink scan in `ferromark_parser`.
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

fn trim_trailing_punct(bytes: &[u8], start: usize, mut end: usize) -> usize {
    // Count lazily: the common case has no closing bracket. Each bracket type
    // is counted at most once, then updated as its trailing closers are removed.
    // This bounds trimming to three linear scans even for mixed bracket runs.
    let mut counts: [Option<(usize, usize)>; 3] = [None; 3];
    while end > start {
        match bytes[end - 1] {
            b'.' | b',' | b';' | b':' | b'!' | b'?' => end -= 1,
            close @ (b')' | b']' | b'}') => {
                let (slot, open) = match close {
                    b')' => (0, b'('),
                    b']' => (1, b'['),
                    _ => (2, b'{'),
                };
                let (opens, closes) = counts[slot].get_or_insert_with(|| {
                    let mut opens = 0;
                    let mut closes = 0;
                    for &byte in &bytes[start..end] {
                        if byte == open {
                            opens += 1;
                        } else if byte == close {
                            closes += 1;
                        }
                    }
                    (opens, closes)
                });
                if *closes > *opens {
                    *closes -= 1;
                    end -= 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    end
}

#[cfg(test)]
mod trimming_tests {
    use super::trim_trailing_punct;

    fn original(bytes: &[u8], start: usize, mut end: usize) -> usize {
        while end > start {
            let b = bytes[end - 1];
            match b {
                b'.' | b',' | b';' | b':' | b'!' | b'?' => end -= 1,
                b')' | b']' | b'}' => {
                    let (open, close) = match b {
                        b')' => (b'(', b')'),
                        b']' => (b'[', b']'),
                        _ => (b'{', b'}'),
                    };
                    // Strip the closing bracket only when it has no unmatched
                    // partner inside the URL — a single pass over the slice is
                    // simpler than two `filter().count()` walks and avoids the
                    // `naive_bytecount` clippy lint.
                    let mut opens = 0usize;
                    let mut closes = 0usize;
                    for &x in &bytes[start..end - 1] {
                        if x == open {
                            opens += 1;
                        } else if x == close {
                            closes += 1;
                        }
                    }
                    if closes >= opens {
                        end -= 1;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        end
    }

    #[test]
    fn matches_original_for_exhaustive_bracket_and_punctuation_tails() {
        const ALPHABET: &[u8] = b"()[]{}.!a";
        let mut bytes = [0u8; 9];
        for length in 0..=6u32 {
            for mut code in 0..ALPHABET.len().pow(length) {
                bytes[..3].copy_from_slice(b"x:/");
                for byte in &mut bytes[3..3 + length as usize] {
                    *byte = ALPHABET[code % ALPHABET.len()];
                    code /= ALPHABET.len();
                }
                let input = &bytes[..3 + length as usize];
                for start in [0, 3] {
                    assert_eq!(
                        trim_trailing_punct(input, start, input.len()),
                        original(input, start, input.len()),
                        "{input:?}, start {start}"
                    );
                }
            }
        }
    }

    #[test]
    fn matches_original_for_long_unbalanced_and_unicode_urls() {
        let mut bytes =
            b"prefix https://example.org/\xe6\x97\xa5\xe6\x9c\xac\xe8\xaa\x9e/([{".to_vec();
        for i in 0..4_096 {
            bytes.push(b")]}.!?"[i % 6]);
        }
        for end in (8..bytes.len()).step_by(37) {
            assert_eq!(
                trim_trailing_punct(&bytes, 7, end),
                original(&bytes, 7, end)
            );
        }
    }
}

#[cfg(test)]
mod range_tests {
    use super::find_autolink_ranges;

    #[test]
    fn finds_renderer_ranges_and_excludes_trailing_punctuation() {
        let text = "See https://example.test/a_(b)). and http://example.test/x...";
        let ranges = find_autolink_ranges(text, &["http://", "https://"]);

        assert_eq!(
            ranges
                .iter()
                .map(|range| &text[range.clone()])
                .collect::<Vec<_>>(),
            ["https://example.test/a_(b)", "http://example.test/x"]
        );
    }

    #[test]
    fn uses_registered_patterns_and_renderer_word_boundaries() {
        let text = "xhttps://bad.test https://good.test ftp://also-good.test";

        let ranges = find_autolink_ranges(text, &["https://"]);
        assert_eq!(ranges.len(), 1);
        assert_eq!(
            ranges[0],
            text.find("https://good").unwrap()..text.find(" ftp://").unwrap()
        );

        let ranges = find_autolink_ranges(text, &["ftp://"]);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], text.find("ftp://").unwrap()..text.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar_oracle(s: &str, from: usize) -> usize {
        let bytes = s.as_bytes();
        let mut end = from;
        while end < bytes.len() {
            let byte = bytes[end];
            if byte.is_ascii() {
                if !is_url_byte(byte) {
                    break;
                }
                end += 1;
                continue;
            }
            let Some(ch) = s.get(end..).and_then(|rest| rest.chars().next()) else {
                break;
            };
            if ends_url(ch) {
                break;
            }
            end += ch.len_utf8();
        }
        end
    }

    #[test]
    fn matches_scalar_for_every_byte_at_every_offset() {
        for value in 0u8..=0x7f {
            for marker_at in 0..40 {
                let mut bytes = [b'x'; 40];
                bytes[marker_at] = value;
                let text = std::str::from_utf8(&bytes).unwrap();
                for from in 0..=bytes.len() {
                    assert_eq!(scan_url_end(text, from), scalar_oracle(text, from));
                }
            }
        }
    }

    #[test]
    fn matches_scalar_for_ascii_tails_and_terminators() {
        let terminators = [b' ', b'\t', b'\n', b'\r', b'<', b'>', b'"', b'\'', b'`'];
        for len in 0..=48 {
            for &terminator in &terminators {
                let mut bytes = [b'a'; 48];
                if len > 0 {
                    bytes[len - 1] = terminator;
                }
                let text = std::str::from_utf8(&bytes[..len]).unwrap();
                for from in 0..=len {
                    assert_eq!(scan_url_end(text, from), scalar_oracle(text, from));
                }
            }
        }
    }

    #[test]
    fn preserves_mixed_utf8_and_cjk_boundaries() {
        for text in [
            "https://example.test/wiki/日本語。next",
            "https://example.test/é中/path`tail",
            "https://example.test/中\u{3000}next",
            "https://example.test/日本語/fullwidth：tail",
        ] {
            for from in 0..=text.len() {
                assert_eq!(
                    scan_url_end(text, from),
                    scalar_oracle(text, from),
                    "text {text:?}, from {from}"
                );
            }
        }
    }

    #[test]
    fn default_http_anchored_scanner_matches_generic_index() {
        // Reversing the order disables `DEFAULT_HTTP_INDEX` so we compare
        // the anchored `://` scanner against the generic first-byte scanner.
        let generic_patterns = ["https://", "http://"];
        let generic_index = FirstByteIndex::from_patterns(&generic_patterns);
        assert!(!generic_index.is_default_http());
        let default_patterns = ["http://", "https://"];
        let default_index = FirstByteIndex::from_patterns(&default_patterns);
        assert!(default_index.is_default_http());

        for text in [
            "",
            "no urls here, just http and https and colons: a:b:c",
            "http://a",
            "https://a",
            "HTTP://EXAMPLE.COM/Path and HTTPS://EXAMPLE.ORG/x.",
            "xhttp://bad.test shttps://bad.test http://good.test",
            "http://. https://ok.test/a_(b)). tail",
            "a:// http://b://c https://d",
            "hhttps://bad.test (https://ok.test)",
            "http://http://nested.test",
            "The quick brown fox jumps over the lazy dog: see https://example.com/1 and http://example.org/2!",
            "ftp://a.test/1 ftp://b.test/2 ftp://c.test/3 ftp://d.test/4 ftp://e.test/5",
            "ftp://a.test/1 ftp://b.test/2 ftp://c.test/3 ftp://d.test/4 ftp://host.test/5 https://ok.test/6",
            "ftp://a.test/1 ftp://b.test/2 ftp://c.test/3 ftp://d.test/4 HTTP://OK.TEST/5",
        ] {
            for from in 0..=text.len() {
                if !text.is_char_boundary(from) {
                    continue;
                }
                assert_eq!(
                    find_autolink_match(text, from, &default_patterns, &default_index),
                    find_autolink_match(text, from, &generic_patterns, &generic_index),
                    "text {text:?}, from {from}"
                );
            }
        }
    }
}
