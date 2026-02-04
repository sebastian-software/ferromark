//! Link reference definitions (CommonMark).

use crate::Range;
use std::collections::HashMap;

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
    by_label: HashMap<String, usize>,
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
    let label_str = std::str::from_utf8(bytes).unwrap_or("");
    let decoded = html_escape::decode_html_entities(label_str);
    let decoded_bytes = decoded.as_bytes();

    let mut unescaped = Vec::with_capacity(decoded_bytes.len());
    let mut i = 0;
    while i < decoded_bytes.len() {
        if decoded_bytes[i] == b'\\' && i + 1 < decoded_bytes.len() && is_label_escapable(decoded_bytes[i + 1]) {
            i += 1;
            unescaped.push(decoded_bytes[i]);
            i += 1;
        } else {
            unescaped.push(decoded_bytes[i]);
            i += 1;
        }
    }

    let unescaped_str = std::str::from_utf8(&unescaped).unwrap_or("");
    let mut out = String::new();
    let mut last_was_space = true;

    for ch in unescaped_str.chars() {
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
    out
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
