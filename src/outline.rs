//! Read-only heading outlines derived from a parsed document.
//!
//! An outline is computed only when [`Document::outline`] is called. It uses
//! the renderer's heading text, level, and ID rules, and returns owned display
//! data alongside each heading's source span.

use std::ops::RangeInclusive;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::ast::{
    Document, FootnoteDefinition, FootnoteReference, Heading, Span, Visit, walk_document,
    walk_footnote_definition, walk_heading,
};
use crate::renderer::{
    HeadingIdPlanner, HtmlRenderer, InvalidHeadingIdPrefix, collect_heading_text,
    map_heading_level, slugify_heading,
};

/// One heading in a document outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineEntry {
    /// Effective HTML heading level after the configured offset and clamping.
    pub level: u8,
    /// Display text collected with the same rules used to generate heading IDs.
    pub text: String,
    /// Resolved, document-unique heading ID, or `None` when heading IDs are off.
    pub id: Option<String>,
    /// Original AST source span for the heading, including its Markdown syntax.
    pub span: Span,
}

/// Options used to derive a read-only outline from one [`Document`].
///
/// Defaults match the renderer's default ID behavior: heading IDs are enabled,
/// semantic footnotes are disabled, the prefix is empty, the level offset is
/// zero and all effective levels (`1..=6`) are included.
///
/// ID-producing settings should match the [`HtmlRenderer`] used for that same
/// document when callers need outline IDs to match rendered HTML.
#[derive(Debug, Clone)]
pub struct OutlineOptions {
    heading_ids: bool,
    semantic_footnotes: bool,
    heading_id_prefix: String,
    heading_level_offset: i32,
    level_filter: RangeInclusive<u8>,
}

impl Default for OutlineOptions {
    fn default() -> Self {
        Self {
            heading_ids: true,
            semantic_footnotes: false,
            heading_id_prefix: String::new(),
            heading_level_offset: 0,
            level_filter: 1..=6,
        }
    }
}

impl OutlineOptions {
    /// Returns these options with heading ID generation enabled or disabled.
    ///
    /// This corresponds to [`crate::renderer::HtmlRendererOptions::heading_ids`].
    #[must_use]
    pub fn with_heading_ids(mut self, enabled: bool) -> Self {
        self.heading_ids = enabled;
        self
    }

    /// Returns these options with the renderer's semantic footnote ID mode.
    ///
    /// This corresponds to
    /// [`crate::renderer::HtmlRendererOptions::semantic_footnotes`]. Footnote
    /// target and reference IDs share the emitted ID namespace with headings,
    /// so this setting can change resolved heading IDs.
    #[must_use]
    pub fn with_semantic_footnotes(mut self, enabled: bool) -> Self {
        self.semantic_footnotes = enabled;
        self
    }

    /// Returns these options with the heading-level offset used by HTML rendering.
    ///
    /// Positive values move headings toward `h6`; negative values move them
    /// toward `h1`. Values clamp to `1..=6` after the offset is applied.
    #[must_use]
    pub fn with_heading_level_offset(mut self, offset: i32) -> Self {
        self.heading_level_offset = offset;
        self
    }

    /// Returns these options with a filter over effective heading levels.
    ///
    /// The filter is applied after offset and clamping. Headings outside the
    /// range are omitted from the returned entries, but still participate in
    /// ID planning so later IDs match the renderer.
    #[must_use]
    pub fn with_level_filter(mut self, levels: RangeInclusive<u8>) -> Self {
        self.level_filter = levels;
        self
    }

    /// Returns these options with a validated prefix for heading IDs.
    ///
    /// This follows [`HtmlRenderer::try_with_heading_id_prefix`]: prefixes may
    /// contain ASCII letters, digits, `_`, and `-`; footnote IDs are unchanged.
    pub fn try_with_heading_id_prefix(
        mut self,
        prefix: impl Into<String>,
    ) -> Result<Self, InvalidHeadingIdPrefix> {
        let prefix = prefix.into();
        HtmlRenderer::validate_heading_id_prefix(&prefix)?;
        self.heading_id_prefix = prefix;
        Ok(self)
    }
}

impl Document<'_> {
    /// Computes a read-only outline for this exact document tree.
    ///
    /// The call walks the document on demand and returns owned display
    /// strings; it does not mutate nodes or cache state on the document. IDs
    /// and filters apply only to this tree. Call this again for a complete or
    /// updated document; outline planning does not carry state between
    /// fragments.
    #[must_use]
    pub fn outline(&self, options: &OutlineOptions) -> Vec<OutlineEntry> {
        let mut collector = OutlineCollector::new(options);
        walk_document(&mut collector, self);
        collector.entries
    }
}

struct OutlineCollector<'options> {
    options: &'options OutlineOptions,
    id_planner: HeadingIdPlanner,
    candidate: String,
    claimed_id: String,
    entries: Vec<OutlineEntry>,
    legacy: LegacyFootnotes,
    semantic: SemanticFootnotes,
}

impl<'options> OutlineCollector<'options> {
    fn new(options: &'options OutlineOptions) -> Self {
        Self {
            options,
            id_planner: HeadingIdPlanner::new(),
            candidate: String::new(),
            claimed_id: String::new(),
            entries: Vec::new(),
            legacy: LegacyFootnotes::default(),
            semantic: SemanticFootnotes::default(),
        }
    }

    fn claim_legacy_target(&mut self, identifier: &str) {
        if self.legacy.targets.insert(identifier.to_owned()) {
            self.candidate.clear();
            self.candidate.push_str("fn-");
            self.candidate.push_str(identifier);
            self.id_planner
                .plan_into(&self.candidate, &mut self.claimed_id);
        }
    }

    fn claim_legacy_reference(&mut self, identifier: &str, occurrence: usize) {
        if occurrence == 1 && !self.legacy.first_references.insert(identifier.to_owned()) {
            return;
        }
        self.candidate.clear();
        self.candidate.push_str("fnref-");
        self.candidate.push_str(identifier);
        if occurrence > 1 {
            self.candidate.push('-');
            self.candidate.push_str(&occurrence.to_string());
        }
        self.id_planner
            .plan_into(&self.candidate, &mut self.claimed_id);
    }

    fn semantic_footnote_index(&mut self, identifier: &str) -> usize {
        if let Some(&index) = self.semantic.indexes.get(identifier) {
            return index;
        }
        let index = self.semantic.slugs.len();
        let mut slug = slugify_heading(identifier);
        if slug == "section" && !identifier.chars().any(char::is_alphanumeric) {
            slug.clear();
            slug.push_str("footnote-");
            slug.push_str(&(index + 1).to_string());
        }
        let base_len = slug.len();
        let mut suffix = 2_usize;
        while !self.semantic.slugs_seen.insert(slug.clone()) {
            slug.truncate(base_len);
            slug.push('-');
            slug.push_str(&suffix.to_string());
            suffix += 1;
        }
        self.candidate.clear();
        self.candidate.push_str("fn-");
        self.candidate.push_str(&slug);
        self.id_planner
            .plan_into(&self.candidate, &mut self.claimed_id);
        self.semantic.indexes.insert(identifier.to_owned(), index);
        self.semantic.slugs.push(slug);
        self.semantic.reference_counts.push(0);
        index
    }

    fn visit_legacy_reference(&mut self, footnote_ref: &FootnoteReference<'_>) {
        let occurrence = self
            .legacy
            .reference_counts
            .entry(footnote_ref.identifier.to_owned())
            .or_default();
        *occurrence += 1;
        let occurrence = *occurrence;
        self.claim_legacy_target(footnote_ref.identifier);
        self.claim_legacy_reference(footnote_ref.identifier, occurrence);
    }

    fn visit_legacy_definition(&mut self, footnote_def: &FootnoteDefinition<'_>) {
        self.claim_legacy_target(footnote_def.identifier);
        walk_footnote_definition(self, footnote_def);
        self.claim_legacy_reference(footnote_def.identifier, 1);
    }

    fn visit_semantic_reference(&mut self, footnote_ref: &FootnoteReference<'_>) {
        let index = self.semantic_footnote_index(footnote_ref.identifier);
        self.semantic.reference_counts[index] += 1;
        self.candidate.clear();
        self.candidate.push_str("fnref-");
        self.candidate.push_str(&self.semantic.slugs[index]);
        let occurrence = self.semantic.reference_counts[index];
        if occurrence > 1 {
            self.candidate.push('-');
            self.candidate.push_str(&occurrence.to_string());
        }
        self.id_planner
            .plan_into(&self.candidate, &mut self.claimed_id);
    }

    fn visit_semantic_definition(&mut self, footnote_def: &FootnoteDefinition<'_>) {
        let _ = self.semantic_footnote_index(footnote_def.identifier);
        if !self
            .semantic
            .definitions
            .insert(footnote_def.identifier.to_owned())
        {
            return;
        }
        walk_footnote_definition(self, footnote_def);
    }
}

impl Visit<'_> for OutlineCollector<'_> {
    fn visit_heading(&mut self, heading: &Heading<'_>) {
        let level = map_heading_level(heading.depth, self.options.heading_level_offset);
        let include = self.options.level_filter.contains(&level);
        let needs_text = include || (self.options.heading_ids && heading.id.is_none());
        let text = if needs_text {
            collect_heading_text(&heading.children)
        } else {
            String::new()
        };

        let id = if self.options.heading_ids {
            self.candidate.clear();
            self.candidate.push_str(&self.options.heading_id_prefix);
            if let Some(explicit_id) = heading.id {
                self.candidate.push_str(explicit_id);
            } else {
                self.candidate.push_str(&slugify_heading(&text));
            }
            self.id_planner
                .plan_into(&self.candidate, &mut self.claimed_id);
            include.then(|| self.claimed_id.clone())
        } else {
            None
        };

        if include {
            self.entries.push(OutlineEntry {
                level,
                text,
                id,
                span: heading.span,
            });
        }
        walk_heading(self, heading);
    }

    fn visit_footnote_reference(&mut self, footnote_ref: &FootnoteReference<'_>) {
        if self.options.semantic_footnotes {
            self.visit_semantic_reference(footnote_ref);
        } else {
            self.visit_legacy_reference(footnote_ref);
        }
    }

    fn visit_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'_>) {
        if self.options.semantic_footnotes {
            self.visit_semantic_definition(footnote_def);
        } else {
            self.visit_legacy_definition(footnote_def);
        }
    }
}

#[derive(Default)]
struct LegacyFootnotes {
    targets: FxHashSet<String>,
    first_references: FxHashSet<String>,
    reference_counts: FxHashMap<String, usize>,
}

#[derive(Default)]
struct SemanticFootnotes {
    indexes: FxHashMap<String, usize>,
    slugs: Vec<String>,
    slugs_seen: FxHashSet<String>,
    reference_counts: Vec<usize>,
    definitions: FxHashSet<String>,
}
