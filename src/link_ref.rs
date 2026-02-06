//! Link reference definitions (CommonMark).

use crate::Range;
use memchr::memchr;
use std::borrow::Cow;
use std::collections::HashMap;
use rustc_hash::FxBuildHasher as FastHashBuilder;

/// A link reference definition (URL + optional title).
#[derive(Debug, Clone)]
pub struct LinkRefDef {
    pub url: Vec<u8>,
    pub title: Option<Vec<u8>>,
}

/// Store of link reference definitions, keyed by normalized label.
#[derive(Debug, Default)]
pub struct LinkRefStore {
    defs: Vec<LinkRefDef>,
    by_label: HashMap<String, usize, FastHashBuilder>,
}

impl LinkRefStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a definition if the label is new. First definition wins.
    pub fn insert(&mut self, label: String, def: LinkRefDef) {
        if self.by_label.contains_key(&label) {
            return;
        }
        let idx = self.defs.len();
        self.defs.push(def);
        self.by_label.insert(label, idx);
    }

    pub fn get_index(&self, label: &str) -> Option<usize> {
        self.by_label.get(label).copied()
    }

    pub fn get(&self, idx: usize) -> Option<&LinkRefDef> {
        self.defs.get(idx)
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }
}

/// Normalize a link label per CommonMark: decode entities, process backslash escapes,
/// collapse internal whitespace to single spaces, trim, and case-fold.
pub fn normalize_label(bytes: &[u8]) -> String {
    let mut out = String::new();
    normalize_label_into(bytes, &mut out);
    out
}

/// Normalize a link label into a reusable buffer.
pub fn normalize_label_into(bytes: &[u8], out: &mut String) {
    out.clear();

    let label_str = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return,
    };
    if label_str.is_empty() {
        return;
    }

    let decoded: Cow<'_, str> = if memchr(b'&', bytes).is_some() {
        html_escape::decode_html_entities(label_str)
    } else {
        Cow::Borrowed(label_str)
    };

    let decoded_bytes = decoded.as_bytes();
    if memchr(b'\\', decoded_bytes).is_some() {
        let mut unescaped = Vec::with_capacity(decoded_bytes.len());
        let mut i = 0;
        while i < decoded_bytes.len() {
            if decoded_bytes[i] == b'\\'
                && i + 1 < decoded_bytes.len()
                && is_label_escapable(decoded_bytes[i + 1])
            {
                i += 1;
                unescaped.push(decoded_bytes[i]);
                i += 1;
            } else {
                unescaped.push(decoded_bytes[i]);
                i += 1;
            }
        }

        let Ok(unescaped_str) = std::str::from_utf8(&unescaped) else {
            return;
        };
        normalize_label_text(unescaped_str, out);
    } else {
        normalize_label_text(decoded.as_ref(), out);
    }
}

#[inline]
fn normalize_label_text(input: &str, out: &mut String) {
    if input.is_ascii() {
        normalize_label_text_ascii(input.as_bytes(), out);
        return;
    }

    let mut last_was_space = true;

    for ch in input.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
            continue;
        }

        last_was_space = false;
        if ch == 'ß' || ch == 'ẞ' {
            out.push('s');
            out.push('s');
        } else {
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
        }
    }

    if out.ends_with(' ') {
        out.pop();
    }
}

#[inline]
fn normalize_label_text_ascii(input: &[u8], out: &mut String) {
    let mut last_was_space = true;

    for &b in input {
        if b.is_ascii_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
            continue;
        }

        last_was_space = false;
        out.push((b.to_ascii_lowercase()) as char);
    }

    if out.ends_with(' ') {
        out.pop();
    }
}

#[inline]
fn is_label_escapable(b: u8) -> bool {
    matches!(b, b'[' | b']' | b'\\')
}

/// Convenience helper to create a definition from ranges in input.
pub fn def_from_ranges(input: &[u8], url: Range, title: Option<Range>) -> LinkRefDef {
    let url_bytes = url.slice(input).to_vec();
    let title_bytes = title.map(|r| r.slice(input).to_vec());
    LinkRefDef {
        url: url_bytes,
        title: title_bytes,
    }
}
