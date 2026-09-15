//! Allocation-aware escaping helpers for text and URL attributes.
//!
//! These routines are on the renderer's hottest path — escaping accounts for
//! roughly a seventh of a full parse+render pipeline on the bundled corpora.
//! They scan for the handful of bytes that must be replaced using SWAR
//! (SIMD-within-a-register) word tests, so long runs of safe text are proven
//! clean eight bytes at a time and then copied with one `push_str`.

// Replacement strings for the bytes that must be escaped. `ESCAPE_FLAG[b]`
// answers "does byte `b` need replacing" for the scalar tail; the hot loop
// uses the SWAR word tests below instead of per-byte lookups.
static ESCAPE_TABLE: [&str; 256] = {
    let mut table: [&str; 256] = [""; 256];
    table[b'&' as usize] = "&amp;";
    table[b'<' as usize] = "&lt;";
    table[b'>' as usize] = "&gt;";
    table[b'"' as usize] = "&quot;";
    table[b'\'' as usize] = "&#39;";
    table
};

static ESCAPE_FLAG: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'&' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'>' as usize] = 1;
    t[b'"' as usize] = 1;
    t[b'\'' as usize] = 1;
    t
};

static URL_ESCAPE_TABLE: [&str; 256] = {
    let mut table: [&str; 256] = [""; 256];
    table[b'&' as usize] = "&amp;";
    table[b'<' as usize] = "%3C";
    table[b'>' as usize] = "%3E";
    table[b'"' as usize] = "%22";
    table[b' ' as usize] = "%20";
    table[b'\\' as usize] = "%5C";
    table[b'[' as usize] = "%5B";
    table[b']' as usize] = "%5D";
    table[b'`' as usize] = "%60";
    table
};

// Non-ASCII bytes are flagged too: they are percent encoded, so one scan
// finds both kinds of replacement instead of bounding ASCII runs first.
static URL_ESCAPE_FLAG: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'&' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'>' as usize] = 1;
    t[b'"' as usize] = 1;
    t[b' ' as usize] = 1;
    t[b'\\' as usize] = 1;
    t[b'[' as usize] = 1;
    t[b']' as usize] = 1;
    t[b'`' as usize] = 1;
    let mut b = 0x80;
    while b < 256 {
        t[b] = 1;
        b += 1;
    }
    t
};

mod nibble;

use nibble::{ESCAPE_NIBBLES, NibbleTables, URL_ESCAPE_NIBBLES, first_flagged_simd};

const ONES: u64 = 0x0101_0101_0101_0101;
const HIGH: u64 = 0x8080_8080_8080_8080;

const fn splat(byte: u8) -> u64 {
    (byte as u64) * ONES
}

/// Sets `0x80` in every byte lane of `word` that is zero.
///
/// The classic bit-twiddling zero test. It can also light up a lane holding
/// `0x01` when a lower lane borrowed into it, which is why every caller below
/// only ever consumes the *lowest* set bit: a spurious lane can only sit
/// above a genuine zero lane that produced the borrow, so the lowest set bit
/// is always a true match.
#[inline]
const fn has_zero(word: u64) -> u64 {
    word.wrapping_sub(ONES) & !word & HIGH
}

/// Nonzero iff `word` holds any of `&`, `<`, `>`, `"`, `'`.
///
/// Five needles fold into three zero tests: `&`(0x26)/`'`(0x27) differ only in
/// bit 0 and `<`(0x3C)/`>`(0x3E) only in bit 1, so forcing that bit on maps
/// each pair onto a single value. `y | 0x01 == 0x27` holds for exactly
/// {0x26, 0x27} and `y | 0x02 == 0x3E` for exactly {0x3C, 0x3E}, so the fold
/// admits no bytes beyond the five.
#[inline]
const fn escape_mask(word: u64) -> u64 {
    has_zero((word | splat(0x01)) ^ splat(b'\''))
        | has_zero((word | splat(0x02)) ^ splat(b'>'))
        | has_zero(word ^ splat(b'"'))
}

/// Nonzero iff `word` holds any URL-sensitive byte or a non-ASCII byte.
///
/// The first three tests retain the compact folds used by text escaping;
/// syntax-sensitive URL bytes use exact one-needle tests so reserved URL
/// delimiters such as `?`, `#`, and `:` remain untouched. The high bit of
/// every lane flags UTF-8 bytes, which are percent encoded.
#[inline]
const fn url_escape_mask(word: u64) -> u64 {
    (word & HIGH)
        | has_zero((word | splat(0x02)) ^ splat(b'>'))
        | has_zero((word | splat(0x02)) ^ splat(b'"'))
        | has_zero(word ^ splat(b'&'))
        | has_zero(word ^ splat(b'\\'))
        | has_zero(word ^ splat(b'['))
        | has_zero(word ^ splat(b']'))
        | has_zero(word ^ splat(b'`'))
}

/// Byte offset of the lowest flagged lane in a nonzero mask.
#[inline]
const fn first_flagged_lane(mask: u64) -> usize {
    (mask.trailing_zeros() / 8) as usize
}

/// Offset of the first byte at or after `from` that needs replacing, or
/// `bytes.len()` when the rest is clean.
///
/// Whole words are cleared by `mask_of`. What is left over when the length
/// is not a multiple of eight used to be walked a byte at a time, and that
/// walk is most of the work: the strings reaching these escapers have a
/// median length of 15 bytes on the bundled corpora, so a *typical* call
/// tested one word and then seven bytes one by one, ending on a loop the
/// branch predictor cannot learn. Re-reading the final eight bytes and
/// discarding the lanes the loop already cleared replaces that walk with
/// one more word test.
///
/// Masking off the low lanes gives up `has_zero`'s "lowest set bit is
/// always a true match" guarantee — a masked-off lane that *is* a match can
/// borrow into the lane above it — so surviving lanes are confirmed against
/// `flags` before being reported. That check is off the hot path: it only
/// runs on the sub-word tail of a string that still holds a match.
#[inline]
fn first_flagged(
    bytes: &[u8],
    from: usize,
    mask_of: impl Fn(u64) -> u64,
    flags: &[u8; 256],
    tables: &NibbleTables,
) -> usize {
    if let Some(found) = first_flagged_simd(bytes, from, tables) {
        return found;
    }
    let len = bytes.len();
    let mut i = from;

    while i + 8 <= len {
        let word = u64::from_le_bytes(copy_eight(bytes, i));
        let mask = mask_of(word);
        if mask != 0 {
            return i + first_flagged_lane(mask);
        }
        i += 8;
    }

    if i < len && len >= 8 {
        // `base < i` here: the loop above only stops with bytes left when
        // fewer than eight remain, so the re-read always overlaps.
        let base = len - 8;
        let word = u64::from_le_bytes(copy_eight(bytes, base));
        let mut mask = mask_of(word) & (u64::MAX << ((i - base) * 8));
        while mask != 0 {
            let lane = base + first_flagged_lane(mask);
            if flags[bytes[lane] as usize] != 0 {
                return lane;
            }
            mask &= mask - 1;
        }
        return len;
    }

    while i < len && flags[bytes[i] as usize] == 0 {
        i += 1;
    }
    i
}

#[inline]
fn copy_eight(bytes: &[u8], from: usize) -> [u8; 8] {
    let mut chunk = [0u8; 8];
    chunk.copy_from_slice(&bytes[from..from + 8]);
    chunk
}

/// Appends `src` to `out`, copying short runs inline instead of handing them
/// to `memmove`.
///
/// Half of the byte runs this module copies are under 16 bytes long — text
/// nodes are short, and every escape replacement is 5 bytes or fewer. At that
/// size libc's `memmove` spends most of its time picking a strategy, and
/// `push_str` cannot avoid it: any length the compiler cannot see lowers to a
/// `memcpy` call. Two overlapping loads and stores cover every run up to 16
/// bytes with no length-dependent branching inside the copy.
#[allow(unsafe_code)]
#[inline]
fn push_run(out: &mut String, src: &str) {
    let len = src.len();
    if len > 16 {
        push_run_long(out, src);
        return;
    }
    out.reserve(len);

    // SAFETY: `reserve` above guarantees `len` spare bytes past the current
    // length, and every write below lands inside `[0, len)` of that spare
    // region, so `set_len` covers exactly the bytes written. `src` is a `&str`
    // appended at the end of `out`, which is a char boundary, so the result
    // stays valid UTF-8. The two pointers cannot overlap: `out` is borrowed
    // uniquely, so no live `&str` can alias its buffer.
    unsafe {
        let vec = out.as_mut_vec();
        let at = vec.len();
        let dst = vec.as_mut_ptr().add(at);
        let src = src.as_ptr();
        if len >= 8 {
            std::ptr::copy_nonoverlapping(src, dst, 8);
            std::ptr::copy_nonoverlapping(src.add(len - 8), dst.add(len - 8), 8);
        } else if len >= 4 {
            std::ptr::copy_nonoverlapping(src, dst, 4);
            std::ptr::copy_nonoverlapping(src.add(len - 4), dst.add(len - 4), 4);
        } else if len > 0 {
            *dst = *src;
            *dst.add(len / 2) = *src.add(len / 2);
            *dst.add(len - 1) = *src.add(len - 1);
        }
        vec.set_len(at + len);
    }
}

/// Appends a run longer than 16 bytes.
///
/// Runs of 17-64 bytes -- the prose between two escaped bytes, most `href`
/// values -- are still short enough that `memmove` spends most of its time
/// choosing a strategy, so they are copied with two or four overlapping
/// 16-byte moves instead. Longer runs go to `push_str`. This stays out of
/// line on purpose: inlining the wider copies into the escape loops made
/// their bodies larger and measurably slowed escape-dense code blocks.
#[allow(unsafe_code)]
#[inline(never)]
fn push_run_long(out: &mut String, src: &str) {
    let len = src.len();
    if len > 64 {
        out.push_str(src);
        return;
    }
    out.reserve(len);

    // SAFETY: `reserve` above guarantees `len` spare bytes past the current
    // length; `len` is in 17..=64, so every 16-byte block below lands inside
    // `[0, len)` of that spare region and `set_len` covers exactly the bytes
    // written. `src` is a `&str` appended at a char boundary, so the result
    // stays valid UTF-8, and the buffers cannot overlap because `out` is
    // borrowed uniquely.
    unsafe {
        let vec = out.as_mut_vec();
        let at = vec.len();
        let dst = vec.as_mut_ptr().add(at);
        let src = src.as_ptr();
        std::ptr::copy_nonoverlapping(src, dst, 16);
        std::ptr::copy_nonoverlapping(src.add(len - 16), dst.add(len - 16), 16);
        if len > 32 {
            std::ptr::copy_nonoverlapping(src.add(16), dst.add(16), 16);
            std::ptr::copy_nonoverlapping(src.add(len - 32), dst.add(len - 32), 16);
        }
        vec.set_len(at + len);
    }
}

/// Shared scan/copy loop for both escapers.
///
/// `mask_of` proves a whole 8-byte word clean in one test; when a word is
/// dirty its mask names the exact byte, so no byte is ever examined twice.
/// `flags`/`table` drive the sub-word tail and the replacement itself.
#[inline]
fn escape_into(
    out: &mut String,
    s: &str,
    mask_of: impl Fn(u64) -> u64,
    flags: &[u8; 256],
    table: &[&'static str; 256],
    tables: &NibbleTables,
) {
    // The invariant: bytes in `s[start..i]` have not been copied yet, and
    // everything before `start` has already been emitted in escaped form.
    let bytes = s.as_bytes();
    let mut start = 0usize;

    loop {
        let i = first_flagged(bytes, start, &mask_of, flags, tables);
        if i >= bytes.len() {
            break;
        }
        if start < i {
            push_run(out, &s[start..i]);
        }
        push_run(out, table[bytes[i] as usize]);
        start = i + 1;
    }

    if start < bytes.len() {
        push_run(out, &s[start..]);
    }
}

#[inline]
pub(super) fn write_escaped_into(out: &mut String, s: &str) {
    // `reserve(s.len())` covers the no-escape case exactly and reduces growth
    // even when replacements make the final output longer.
    out.reserve(s.len());
    escape_into(
        out,
        s,
        escape_mask,
        &ESCAPE_FLAG,
        &ESCAPE_TABLE,
        &ESCAPE_NIBBLES,
    );
}

pub(super) fn write_url_escaped_into(out: &mut String, s: &str) {
    // Same scanner as `write_escaped_into`, but with URL attribute semantics.
    // Ampersand stays HTML-escaped because the result is written inside an
    // HTML attribute, while spaces and tag delimiters are percent encoded to
    // keep the URL value itself stable.
    // Brackets in an IPv6 authority are URL syntax and must remain delimiters;
    // brackets in a path or query are encoded. Split around the one authority
    // pair when present so the common scanner remains branch-free.
    out.reserve(s.len());
    if let Some((open, close)) = ipv6_authority_brackets(s) {
        write_url_segment(out, &s[..open]);
        out.push_str(&s[open..=close]);
        write_url_segment(out, &s[close + 1..]);
    } else {
        write_url_segment(out, s);
    }
}

/// Escapes URL syntax and UTF-8 bytes with one scan. The classifier flags
/// replacement bytes and non-ASCII bytes alike, so a URL is never walked
/// byte by byte just to bound its ASCII runs. Percent encoding operates on
/// the UTF-8 bytes, as required by cmark's URI renderer, and leaves existing
/// `%HH` sequences alone.
fn write_url_segment(out: &mut String, s: &str) {
    let bytes = s.as_bytes();
    let mut start = 0usize;
    loop {
        let i = next_url_flagged(bytes, start);
        if i >= bytes.len() {
            break;
        }
        if start < i {
            push_run(out, &s[start..i]);
        }
        let byte = bytes[i];
        if byte >= 0x80 {
            // `start` sits on a char boundary and this is the first non-ASCII
            // byte after it, so the run begins at a leading byte and ends at
            // the next ASCII byte or the end of the string.
            let unicode_end = bytes[i..]
                .iter()
                .position(|&byte| byte < 0x80)
                .map_or(bytes.len(), |offset| i + offset);
            for &byte in &bytes[i..unicode_end] {
                push_percent_byte(out, byte);
            }
            start = unicode_end;
        } else {
            push_run(out, URL_ESCAPE_TABLE[byte as usize]);
            start = i + 1;
        }
    }
    if start < bytes.len() {
        push_run(out, &s[start..]);
    }
}

/// Offset of the next URL byte that needs replacing or percent encoding.
///
/// The first scan of a segment goes straight to the vector scanner: most
/// URLs flag nothing. After a hit, a short table walk runs first, because
/// re-entering the scanner would reload its tables and read a full vector
/// for what is often a one- or two-byte gap. Entity- or Unicode-dense URLs
/// flag a byte every few positions and stayed scalar in the original
/// two-pass escaper; this keeps them there while a long clean run after a
/// hit still hands over to the vector scan after eight bytes.
#[inline]
fn next_url_flagged(bytes: &[u8], from: usize) -> usize {
    const SHORT_RUN_PREFIX: usize = 8;
    let mut i = from;
    if from != 0 {
        let quick = from.saturating_add(SHORT_RUN_PREFIX).min(bytes.len());
        while i < quick && URL_ESCAPE_FLAG[bytes[i] as usize] == 0 {
            i += 1;
        }
        if i < quick {
            return i;
        }
    }
    first_flagged(
        bytes,
        i,
        url_escape_mask,
        &URL_ESCAPE_FLAG,
        &URL_ESCAPE_NIBBLES,
    )
}

#[inline]
fn push_percent_byte(out: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push('%');
    out.push(HEX[(byte >> 4) as usize] as char);
    out.push(HEX[(byte & 0x0F) as usize] as char);
}

/// Returns the bracket pair delimiting an IPv6 host in a URL authority.
/// Brackets elsewhere are data and remain subject to URL escaping.
fn ipv6_authority_brackets(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    memchr::memchr(b'[', bytes)?;
    let authority_start = if bytes.starts_with(b"//") {
        2
    } else {
        let first = *bytes.first()?;
        if !first.is_ascii_alphabetic() {
            return None;
        }
        let mut scheme_end = 1;
        while let Some(&byte) = bytes.get(scheme_end) {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.') {
                scheme_end += 1;
            } else {
                break;
            }
        }
        (bytes.get(scheme_end..scheme_end + 3) == Some(b"://")).then_some(scheme_end + 3)?
    };
    let authority_end = bytes[authority_start..]
        .iter()
        .position(|&byte| matches!(byte, b'/' | b'?' | b'#'))
        .map_or(bytes.len(), |offset| authority_start + offset);
    let userinfo_end = bytes[authority_start..authority_end]
        .iter()
        .rposition(|&byte| byte == b'@')
        .map_or(authority_start, |offset| authority_start + offset + 1);
    let open = userinfo_end;
    if bytes.get(open) != Some(&b'[') {
        return None;
    }
    let close = bytes[open + 1..authority_end]
        .iter()
        .position(|&byte| byte == b']')?
        + open
        + 1;
    let host = std::str::from_utf8(&bytes[open + 1..close]).ok()?;
    host.parse::<std::net::Ipv6Addr>().ok()?;
    match bytes.get(close + 1..authority_end)? {
        [] => Some((open, close)),
        [b':', port @ ..] if port.iter().all(u8::is_ascii_digit) => Some((open, close)),
        _ => None,
    }
}

/// Escapes attribute values, including line endings, directly into the output.
/// Separate field borrows let heading IDs stay in their reusable scratch buffer.
pub(super) fn write_attribute_escaped_into(out: &mut String, s: &str) {
    // Attribute escaping differs from text escaping only at CR/LF. Split
    // at those ASCII boundaries and reuse the vectorized text escaper for
    // whole runs, preserving UTF-8 without decoding and pushing each char.
    let mut start = 0;
    for index in memchr::memchr2_iter(b'\r', b'\n', s.as_bytes()) {
        write_escaped_into(out, &s[start..index]);
        out.push_str(if s.as_bytes()[index] == b'\r' {
            "&#13;"
        } else {
            "&#10;"
        });
        start = index + 1;
    }
    write_escaped_into(out, &s[start..]);
}

#[cfg(test)]
mod tests;
