//! Footnote definitions storage.

use crate::block::BlockEvent;
use rustc_hash::FxBuildHasher as FastHashBuilder;
use std::collections::HashMap;

/// A footnote definition (stores captured block events).
#[derive(Debug, Clone)]
pub struct FootnoteDef {
    /// The original label text (for generating HTML IDs).
    pub label: String,
    /// Block events captured for this footnote's content.
    pub events: Vec<BlockEvent>,
}

/// Store of footnote definitions, keyed by normalized label.
#[derive(Debug, Default)]
pub struct FootnoteStore {
    defs: Vec<FootnoteDef>,
    by_label: HashMap<String, usize, FastHashBuilder>,
}

impl FootnoteStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a definition if the label is new. First definition wins.
    pub fn insert(&mut self, normalized_label: String, label: String, events: Vec<BlockEvent>) {
        if self.by_label.contains_key(&normalized_label) {
            return;
        }
        let idx = self.defs.len();
        self.defs.push(FootnoteDef { label, events });
        self.by_label.insert(normalized_label, idx);
    }

    pub fn get_index(&self, label: &str) -> Option<usize> {
        self.by_label.get(label).copied()
    }

    /// Look up a definition by raw (unnormalized) label bytes.
    ///
    /// Normalizes into a stack buffer, so reference-resolution lookups do not
    /// allocate. Returns `None` for labels with invalid characters.
    pub fn get_index_bytes(&self, label_bytes: &[u8]) -> Option<usize> {
        if label_bytes.is_empty() {
            return None;
        }
        let mut buf = smallvec::SmallVec::<[u8; 32]>::with_capacity(label_bytes.len());
        for &b in label_bytes {
            if !b.is_ascii_alphanumeric() && b != b'-' && b != b'_' {
                return None;
            }
            buf.push(b.to_ascii_lowercase());
        }
        // Labels are validated ASCII above, so this cannot fail.
        let normalized = std::str::from_utf8(&buf).ok()?;
        self.by_label.get(normalized).copied()
    }

    pub fn get(&self, idx: usize) -> Option<&FootnoteDef> {
        self.defs.get(idx)
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }
}

/// Normalize a footnote label: trim, lowercase ASCII.
/// Footnote labels are restricted to `[a-zA-Z0-9_-]` so no entity/escape processing needed.
pub fn normalize_footnote_label(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    // Validate: only alphanumeric, dash, underscore
    for &b in bytes {
        if !b.is_ascii_alphanumeric() && b != b'-' && b != b'_' {
            return None;
        }
    }
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        out.push(b.to_ascii_lowercase() as char);
    }
    Some(out)
}
