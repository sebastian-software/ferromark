//! Unique document heading IDs shared by HTML rendering, TOCs, and metadata.

use std::collections::hash_map::Entry;
use std::fmt::Write as _;

use compact_str::CompactString;
use rustc_hash::FxHashMap;

use crate::ast::{Document, FootnoteDefinition, FootnoteReference, Visit};

use super::heading::{collect_heading_text, slugify_heading};

/// Plans unique heading IDs in document order.
///
/// The first heading that requests an ID keeps it. Later headings that request
/// the same ID receive `-1`, `-2`, and so on, skipping IDs already emitted or
/// reserved for footnotes. Use one planner for each document so HTML headings,
/// TOC links, and metadata use the same values.
#[derive(Clone, Default)]
pub struct HeadingIdPlanner {
    /// Every key is already used or reserved. A positive value is the next
    /// suffix to try when the key is requested again; zero marks an ID that
    /// was reserved or emitted as another base's suffix.
    ids: FxHashMap<CompactString, usize>,
}

impl HeadingIdPlanner {
    /// Creates an empty planner.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a planner with this document's footnote IDs reserved.
    ///
    /// Reserve these before calling [`Self::unique_id`] so headings cannot
    /// claim IDs emitted by footnote definitions or references.
    #[must_use]
    pub fn for_document(document: &Document<'_>, semantic_footnotes: bool) -> Self {
        let mut planner = Self::new();
        planner.reserve_document_footnote_ids(document, semantic_footnotes);
        planner
    }

    /// Returns an unused version of `base`, registering the result as used.
    ///
    /// The first request for an unused base returns it unchanged. Later
    /// requests append an increasing numeric suffix, skipping any value that
    /// another base or a reserved footnote already occupies.
    #[must_use]
    pub fn unique_id(&mut self, base: &str) -> CompactString {
        let mut id = String::new();
        self.write_unique_id(base, &mut id);
        CompactString::from(id.as_str())
    }

    pub(super) fn write_unique_id(&mut self, base: &str, id: &mut String) {
        id.clear();
        let mut suffix = match self.ids.entry(CompactString::from(base)) {
            Entry::Vacant(entry) => {
                entry.insert(1);
                id.push_str(base);
                return;
            }
            Entry::Occupied(entry) => entry.get().max(&1).to_owned(),
        };
        let mut candidate = String::with_capacity(base.len() + 12);
        loop {
            candidate.clear();
            candidate.push_str(base);
            let _ = write!(candidate, "-{suffix}");
            suffix = suffix.saturating_add(1);
            if !self.ids.contains_key(candidate.as_str()) {
                self.ids.insert(CompactString::from(candidate.as_str()), 0);
                self.ids.insert(CompactString::from(base), suffix);
                id.push_str(&candidate);
                return;
            }
        }
    }

    pub(super) fn clear(&mut self) {
        self.ids.clear();
    }

    pub(super) fn reserve_capacity(&mut self, heading_count: usize) {
        self.ids.reserve(heading_count);
    }

    pub(super) fn reserve_document_footnote_ids(
        &mut self,
        document: &Document<'_>,
        semantic_footnotes: bool,
    ) {
        let mut scan = FootnoteIdScan::new(self, semantic_footnotes);
        scan.visit_document(document);
    }
}

struct FootnoteIdScan<'p> {
    planner: &'p mut HeadingIdPlanner,
    semantic: bool,
    legacy_reference_counts: FxHashMap<String, usize>,
    semantic_reference_counts: FxHashMap<CompactString, usize>,
    semantic_slugs: FxHashMap<CompactString, CompactString>,
    semantic_slug_counts: FxHashMap<CompactString, usize>,
    id_scratch: String,
}

impl<'p> FootnoteIdScan<'p> {
    fn new(planner: &'p mut HeadingIdPlanner, semantic: bool) -> Self {
        Self {
            planner,
            semantic,
            legacy_reference_counts: FxHashMap::default(),
            semantic_reference_counts: FxHashMap::default(),
            semantic_slugs: FxHashMap::default(),
            semantic_slug_counts: FxHashMap::default(),
            id_scratch: String::new(),
        }
    }

    fn reserve_legacy_definition(&mut self, identifier: &str) {
        self.reserve_with_prefix("fn-", identifier, None);
        // The definition's backlink targets the first reference ID.
        self.reserve_with_prefix("fnref-", identifier, None);
    }

    fn reserve_legacy_reference(&mut self, identifier: &str) {
        self.reserve_with_prefix("fn-", identifier, None);
        let occurrence = {
            let count = self
                .legacy_reference_counts
                .entry(identifier.to_owned())
                .or_default();
            *count += 1;
            *count
        };
        if occurrence == 1 {
            self.reserve_with_prefix("fnref-", identifier, None);
        } else {
            self.reserve_with_prefix("fnref-", identifier, Some(occurrence));
        }
    }

    fn reserve_semantic_definition(&mut self, identifier: &str) {
        let slug = self.semantic_slug(identifier);
        self.reserve_with_prefix("fn-", &slug, None);
    }

    fn reserve_semantic_reference(&mut self, identifier: &str) {
        let slug = self.semantic_slug(identifier);
        self.reserve_with_prefix("fn-", &slug, None);
        let occurrence = {
            let count = self
                .semantic_reference_counts
                .entry(slug.clone())
                .or_default();
            *count += 1;
            *count
        };
        if occurrence == 1 {
            self.reserve_with_prefix("fnref-", &slug, None);
        } else {
            self.reserve_with_prefix("fnref-", &slug, Some(occurrence));
        }
    }

    fn semantic_slug(&mut self, identifier: &str) -> CompactString {
        if let Some(slug) = self.semantic_slugs.get(identifier) {
            return slug.clone();
        }

        let index = self.semantic_slugs.len();
        let mut slug = slugify_heading(identifier);
        if slug == "section" && !identifier.chars().any(char::is_alphanumeric) {
            slug.clear();
            let _ = write!(slug, "footnote-{}", index + 1);
        }

        if let Some(next) = self.semantic_slug_counts.get(slug.as_str()).copied() {
            let base = CompactString::from(slug.as_str());
            let mut suffix = next;
            loop {
                slug.clear();
                slug.push_str(&base);
                let _ = write!(slug, "-{suffix}");
                suffix = suffix.saturating_add(1);
                if !self.semantic_slug_counts.contains_key(slug.as_str()) {
                    self.semantic_slug_counts
                        .insert(CompactString::from(slug.as_str()), 2);
                    self.semantic_slug_counts.insert(base, suffix);
                    break;
                }
            }
        } else {
            self.semantic_slug_counts
                .insert(CompactString::from(slug.as_str()), 2);
        }

        let slug = CompactString::from(slug.as_str());
        self.semantic_slugs
            .insert(CompactString::from(identifier), slug.clone());
        slug
    }

    fn reserve_with_prefix(&mut self, prefix: &str, value: &str, suffix: Option<usize>) {
        self.id_scratch.clear();
        self.id_scratch.push_str(prefix);
        self.id_scratch.push_str(value);
        if let Some(suffix) = suffix {
            let _ = write!(self.id_scratch, "-{suffix}");
        }
        self.planner
            .ids
            .entry(CompactString::from(self.id_scratch.as_str()))
            .or_insert(0);
    }
}

impl<'a> Visit<'a> for FootnoteIdScan<'_> {
    fn visit_footnote_reference(&mut self, footnote_ref: &FootnoteReference<'a>) {
        if self.semantic {
            self.reserve_semantic_reference(footnote_ref.identifier);
        } else {
            self.reserve_legacy_reference(footnote_ref.identifier);
        }
    }

    fn visit_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'a>) {
        if self.semantic {
            self.reserve_semantic_definition(footnote_def.identifier);
        } else {
            self.reserve_legacy_definition(footnote_def.identifier);
        }
        crate::ast::walk_footnote_definition(self, footnote_def);
    }
}

/// Computes the unprefixed requested ID for a heading.
pub(super) fn heading_id_base(heading: &crate::ast::Heading<'_>) -> String {
    heading.id.map_or_else(
        || slugify_heading(&collect_heading_text(&heading.children)),
        str::to_owned,
    )
}

#[cfg(test)]
mod tests {
    use super::HeadingIdPlanner;

    #[test]
    fn skips_ids_claimed_by_other_heading_bases() {
        let mut ids = HeadingIdPlanner::new();
        assert_eq!(ids.unique_id("a"), "a");
        assert_eq!(ids.unique_id("a"), "a-1");
        assert_eq!(ids.unique_id("a-1"), "a-1-1");
        assert_eq!(ids.unique_id("a-1"), "a-1-2");
    }

    #[test]
    fn skips_reserved_footnote_ids() {
        let mut ids = HeadingIdPlanner::new();
        ids.ids.insert("fn-1".into(), 0);
        assert_eq!(ids.unique_id("fn-1"), "fn-1-1");
    }
}
