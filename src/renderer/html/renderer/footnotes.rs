//! Footnote reference and definition rendering.
//!
//! The default path keeps the established v2 markup: the source identifier is
//! the visible marker, and each definition is an independent
//! `<div class="footnote">`.
//!
//! The `semantic_footnotes` renderer option switches to document-order numeric
//! markers and one `<section class="footnotes"><ol>` of definitions. The
//! identifier is used only for lookup and slug generation. Numbering and
//! collection happen in the same AST walk: one list entry per unique
//! footnote, no extra allocation per later marker.

use std::fmt::Write as _;

use crate::ast::{FootnoteDefinition, FootnoteReference, Span};
use compact_str::CompactString;
use rustc_hash::FxHashMap;

use super::super::escape::write_escaped_into;
use super::super::heading::{claim_footnote_slug, slugify_heading_into};
use super::{HtmlRenderHooks, HtmlRenderer, PlannedId, reserve_heading_scratch};

/// Render-time footnote collections, boxed behind `Option` on [`HtmlRenderer`]
/// so documents without footnotes pay only a pointer check.
#[derive(Clone, Default)]
pub(super) struct FootnoteState {
    /// How many times each footnote identifier has been referenced so far in
    /// this render, so repeated references get unique `fnref-` IDs.
    ref_counts: FxHashMap<CompactString, usize>,
    /// Legacy source identifier → claimed target ID.
    legacy_targets: FxHashMap<CompactString, PlannedId>,
    /// Reserved first reference ID for each legacy footnote.
    legacy_first_references: FxHashMap<CompactString, PlannedId>,
    /// Source identifier → list index for semantic footnotes.
    index: FxHashMap<CompactString, u32>,
    /// Document-order footnote list used when `semantic_footnotes` is on.
    records: Vec<FootnoteRecord>,
    /// Slugs already handed to a footnote, mapped to the next `-N` suffix.
    slug_counts: FxHashMap<CompactString, usize>,
}

#[derive(Clone)]
pub(super) struct FootnoteRecord {
    pub(super) slug: CompactString,
    pub(super) target_id: PlannedId,
    pub(super) reference_ids: Vec<PlannedId>,
    pub(super) ref_count: usize,
    pub(super) body_html: Option<String>,
    pub(super) span: Option<Span>,
}

impl HtmlRenderer {
    #[inline]
    fn footnotes_mut(&mut self) -> &mut FootnoteState {
        self.footnotes.get_or_insert_with(Default::default)
    }

    pub(in crate::renderer::html::renderer) fn render_footnote_reference(
        &mut self,
        footnote_ref: &FootnoteReference<'_>,
    ) {
        if self.options.semantic_footnotes {
            self.render_semantic_footnote_reference(footnote_ref.identifier);
            return;
        }
        self.render_legacy_footnote_reference(footnote_ref.identifier);
    }

    pub(in crate::renderer::html::renderer) fn render_footnote_definition(
        &mut self,
        footnote_def: &FootnoteDefinition<'_>,
    ) {
        if self.options.semantic_footnotes {
            self.collect_semantic_footnote_definition(footnote_def);
            return;
        }
        self.render_legacy_footnote_definition(footnote_def);
    }

    pub(in crate::renderer::html::renderer) fn render_footnote_definition_with_hooks<
        H: HtmlRenderHooks,
    >(
        &mut self,
        footnote_def: &FootnoteDefinition<'_>,
        hooks: &mut H,
    ) {
        if self.options.semantic_footnotes {
            self.collect_semantic_footnote_definition_with_hooks(footnote_def, hooks);
            return;
        }
        self.render_legacy_footnote_definition_with_hooks(footnote_def, hooks);
    }

    pub(in crate::renderer::html::renderer) fn finish_semantic_footnotes(&mut self) {
        if !self.options.semantic_footnotes {
            return;
        }
        let Some(mut state) = self.footnotes.take() else {
            return;
        };
        if !state
            .records
            .iter()
            .any(|record| record.body_html.is_some())
        {
            self.footnotes = Some(state);
            return;
        }
        self.write("<section class=\"footnotes\" aria-label=\"Footnotes\">\n<ol>\n");
        for (index, record) in state.records.iter_mut().enumerate() {
            let Some(body) = record.body_html.take() else {
                continue;
            };
            let display = index + 1;
            let target_id = record.target_id;
            self.write("<li id=\"");
            self.write_footnote_id(target_id);
            self.write("\"");
            if let Some(span) = record.span {
                self.write_source_span_attr(span);
            }
            self.write(">\n");
            self.write(&body);
            self.write_semantic_backlinks(&record.reference_ids, display);
            self.write("</li>\n");
        }
        self.write("</ol>\n</section>\n");
        self.footnotes = Some(state);
    }

    pub(in crate::renderer::html::renderer) fn clear_footnote_state(&mut self) {
        let Some(state) = self.footnotes.as_deref_mut() else {
            return;
        };
        state.ref_counts.clear();
        state.legacy_targets.clear();
        state.legacy_first_references.clear();
        if !self.options.semantic_footnotes {
            debug_assert!(
                state.index.is_empty() && state.records.is_empty() && state.slug_counts.is_empty(),
                "semantic footnote state written while the option is off"
            );
            return;
        }
        state.index.clear();
        state.records.clear();
        state.slug_counts.clear();
    }

    fn render_legacy_footnote_reference(&mut self, identifier: &str) {
        // A footnote may be referenced repeatedly, so each occurrence
        // needs its own id: the first keeps `fnref-<id>` (which the
        // definition's back-link targets) and later ones get a `-N`
        // suffix. Without this the document carries duplicate ids, which
        // is invalid HTML and breaks in-page anchors.
        let occurrence = {
            let state = self.footnotes_mut();
            if let Some(count) = state.ref_counts.get_mut(identifier) {
                *count += 1;
                *count
            } else {
                state.ref_counts.insert(CompactString::from(identifier), 1);
                1
            }
        };
        let target_id = self.legacy_footnote_target_id(identifier);
        let reference_id = self.legacy_footnote_reference_id(identifier, occurrence);

        self.write("<sup><a href=\"#");
        self.write_footnote_id(target_id);
        self.write("\" id=\"");
        self.write_footnote_id(reference_id);
        self.write("\">");
        self.write_escaped(identifier);
        self.write("</a></sup>");
    }

    fn render_legacy_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'_>) {
        let target_id = self.legacy_footnote_target_id(footnote_def.identifier);
        self.write("<div id=\"");
        self.write_footnote_id(target_id);
        self.write("\" class=\"footnote\"");
        self.write_source_span_attr(footnote_def.span);
        self.write(">\n");
        for child in &footnote_def.children {
            self.render_node(child);
        }
        // The definition emits a backlink even when its reference appears
        // later in the document. Reserve that anchor now so a heading between
        // the definition and reference cannot take the link's ID.
        let reference_id = self.legacy_footnote_reference_id(footnote_def.identifier, 1);
        self.write("<a href=\"#");
        self.write_footnote_id(reference_id);
        self.write("\">↩</a>\n</div>\n");
    }

    fn render_legacy_footnote_definition_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        footnote_def: &FootnoteDefinition<'_>,
        hooks: &mut H,
    ) {
        let target_id = self.legacy_footnote_target_id(footnote_def.identifier);
        self.write("<div id=\"");
        self.write_footnote_id(target_id);
        self.write("\" class=\"footnote\"");
        self.write_source_span_attr(footnote_def.span);
        self.write(">\n");
        for child in &footnote_def.children {
            self.render_node_with_hooks(child, hooks);
        }
        // Keep hooked rendering's claim order identical to the default path.
        let reference_id = self.legacy_footnote_reference_id(footnote_def.identifier, 1);
        self.write("<a href=\"#");
        self.write_footnote_id(reference_id);
        self.write("\">↩</a>\n</div>\n");
    }

    fn render_semantic_footnote_reference(&mut self, identifier: &str) {
        let index = self.footnote_index_of(identifier);
        let (occurrence, target_id, slug) = {
            let record = &mut self.footnotes_mut().records[index];
            record.ref_count += 1;
            (record.ref_count, record.target_id, record.slug.clone())
        };
        self.heading_slug_scratch.clear();
        self.heading_slug_scratch.push_str("fnref-");
        self.heading_slug_scratch.push_str(&slug);
        if occurrence > 1 {
            let _ = write!(self.heading_slug_scratch, "-{occurrence}");
        }
        let reference_id = self.heading_id_planner.claim(&self.heading_slug_scratch);
        self.footnotes_mut().records[index]
            .reference_ids
            .push(reference_id);

        self.write("<sup><a href=\"#");
        self.write_footnote_id(target_id);
        self.write("\" id=\"");
        self.write_footnote_id(reference_id);
        self.write("\">");
        self.write_display(index + 1);
        self.write("</a></sup>");
    }

    fn collect_semantic_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'_>) {
        let index = self.footnote_index_of(footnote_def.identifier);
        if self.footnotes_mut().records[index].body_html.is_some() {
            return;
        }
        let start = self.output.len();
        for child in &footnote_def.children {
            self.render_node(child);
        }
        let body = self.output.split_off(start);
        let record = &mut self.footnotes_mut().records[index];
        record.body_html = Some(body);
        record.span = Some(footnote_def.span);
    }

    fn collect_semantic_footnote_definition_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        footnote_def: &FootnoteDefinition<'_>,
        hooks: &mut H,
    ) {
        let index = self.footnote_index_of(footnote_def.identifier);
        if self.footnotes_mut().records[index].body_html.is_some() {
            return;
        }
        let start = self.output.len();
        for child in &footnote_def.children {
            self.render_node_with_hooks(child, hooks);
        }
        let body = self.output.split_off(start);
        let record = &mut self.footnotes_mut().records[index];
        record.body_html = Some(body);
        record.span = Some(footnote_def.span);
    }

    fn write_semantic_backlinks(&mut self, reference_ids: &[PlannedId], display: usize) {
        let ref_count = reference_ids.len();
        if ref_count == 0 {
            return;
        }
        for (idx, &reference_id) in reference_ids.iter().enumerate() {
            let occurrence = idx + 1;
            self.write("<a href=\"#");
            self.write_footnote_id(reference_id);
            self.write("\" aria-label=\"Back to reference ");
            self.write_display(display);
            if occurrence > 1 {
                self.write(", occurrence ");
                self.write_display(occurrence);
            }
            self.write("\">↩</a>");
            if occurrence < ref_count {
                self.write(" ");
            }
        }
        self.write("\n");
    }

    fn legacy_footnote_target_id(&mut self, identifier: &str) -> PlannedId {
        if let Some(&id) = self
            .footnotes
            .as_deref()
            .and_then(|state| state.legacy_targets.get(identifier))
        {
            return id;
        }
        self.heading_slug_scratch.clear();
        self.heading_slug_scratch.push_str("fn-");
        self.heading_slug_scratch.push_str(identifier);
        let id = self.heading_id_planner.claim(&self.heading_slug_scratch);
        self.footnotes_mut()
            .legacy_targets
            .insert(CompactString::from(identifier), id);
        id
    }

    fn legacy_footnote_reference_id(&mut self, identifier: &str, occurrence: usize) -> PlannedId {
        if occurrence == 1
            && let Some(&id) = self
                .footnotes
                .as_deref()
                .and_then(|state| state.legacy_first_references.get(identifier))
        {
            return id;
        }
        self.heading_slug_scratch.clear();
        self.heading_slug_scratch.push_str("fnref-");
        self.heading_slug_scratch.push_str(identifier);
        if occurrence > 1 {
            let _ = write!(self.heading_slug_scratch, "-{occurrence}");
        }
        let id = self.heading_id_planner.claim(&self.heading_slug_scratch);
        if occurrence == 1 {
            self.footnotes_mut()
                .legacy_first_references
                .insert(CompactString::from(identifier), id);
        }
        id
    }

    fn footnote_index_of(&mut self, identifier: &str) -> usize {
        if let Some(&index) = self
            .footnotes
            .as_deref()
            .and_then(|state| state.index.get(identifier))
        {
            return index as usize;
        }
        let index = self
            .footnotes
            .as_deref()
            .map_or(0, |state| state.records.len());
        let slug = self.assign_footnote_slug(identifier, index);
        self.heading_slug_scratch.clear();
        self.heading_slug_scratch.push_str("fn-");
        self.heading_slug_scratch.push_str(&slug);
        let target_id = self.heading_id_planner.claim(&self.heading_slug_scratch);
        let state = self.footnotes_mut();
        state
            .index
            .insert(CompactString::from(identifier), index as u32);
        state.records.push(FootnoteRecord {
            slug,
            target_id,
            reference_ids: Vec::new(),
            ref_count: 0,
            body_html: None,
            span: None,
        });
        index
    }

    fn assign_footnote_slug(&mut self, identifier: &str, index: usize) -> CompactString {
        let Self {
            heading_slug_scratch,
            footnotes,
            ..
        } = self;
        heading_slug_scratch.clear();
        reserve_heading_scratch(heading_slug_scratch);
        slugify_heading_into(identifier, heading_slug_scratch);
        if heading_slug_scratch == "section" && !identifier.chars().any(char::is_alphanumeric) {
            heading_slug_scratch.clear();
            let _ = write!(heading_slug_scratch, "footnote-{}", index + 1);
        }
        let slug_counts = &mut footnotes.get_or_insert_with(Default::default).slug_counts;
        claim_footnote_slug(heading_slug_scratch, slug_counts);
        CompactString::from(heading_slug_scratch.as_str())
    }

    fn write_footnote_id(&mut self, id: PlannedId) {
        write_escaped_into(&mut self.output, self.heading_id_planner.id(id));
    }
}
