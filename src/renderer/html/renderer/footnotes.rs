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

use super::super::escape::write_escaped_into;
use super::super::heading::slugify_heading_into;
use super::{HtmlRenderHooks, HtmlRenderer, PlannedId, reserve_heading_scratch};

/// First `-N` tried when a slug repeats, matching the ids the scan produced.
const FIRST_SUFFIX: usize = 2;

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
        if !self.options.semantic_footnotes
            || !self
                .footnote_records
                .iter()
                .any(|record| record.body_html.is_some())
        {
            return;
        }
        self.write("<section class=\"footnotes\" aria-label=\"Footnotes\">\n<ol>\n");
        for index in 0..self.footnote_records.len() {
            let Some(body) = self.footnote_records[index].body_html.take() else {
                continue;
            };
            let display = index + 1;
            let target_id = self.footnote_records[index].target_id;
            self.write("<li id=\"");
            self.write_footnote_id(target_id);
            self.write("\"");
            if let Some(span) = self.footnote_records[index].span {
                self.write_source_span_attr(span);
            }
            self.write(">\n");
            self.write(&body);
            self.write_semantic_backlinks(index, display);
            self.write("</li>\n");
        }
        self.write("</ol>\n</section>\n");
    }

    pub(in crate::renderer::html::renderer) fn clear_footnote_state(&mut self) {
        // The legacy marker path uses this one whether or not the semantic
        // option is on.
        self.footnote_ref_counts.clear();
        self.legacy_footnote_targets.clear();
        self.legacy_footnote_first_references.clear();
        if !self.options.semantic_footnotes {
            // The other three are written only from `footnote_index_of` and
            // `assign_footnote_slug`, both reachable only through the semantic
            // reference and definition renderers, and `options` cannot change
            // after the renderer is constructed. With the option off they stay
            // empty for the renderer's whole life, so there is nothing to
            // clear.
            debug_assert!(
                self.footnote_index.is_empty()
                    && self.footnote_records.is_empty()
                    && self.footnote_slug_counts.is_empty(),
                "semantic footnote state written while the option is off"
            );
            return;
        }
        self.footnote_index.clear();
        self.footnote_records.clear();
        self.footnote_slug_counts.clear();
    }

    fn render_legacy_footnote_reference(&mut self, identifier: &str) {
        // A footnote may be referenced repeatedly, so each occurrence
        // needs its own id: the first keeps `fnref-<id>` (which the
        // definition's back-link targets) and later ones get a `-N`
        // suffix. Without this the document carries duplicate ids, which
        // is invalid HTML and breaks in-page anchors.
        let occurrence = {
            let count = self
                .footnote_ref_counts
                .entry(identifier.to_owned())
                .or_insert(0);
            *count += 1;
            *count
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
            let record = &mut self.footnote_records[index];
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
        self.footnote_records[index]
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
        if self.footnote_records[index].body_html.is_some() {
            return;
        }
        let start = self.output.len();
        for child in &footnote_def.children {
            self.render_node(child);
        }
        self.footnote_records[index].body_html = Some(self.output.split_off(start));
        self.footnote_records[index].span = Some(footnote_def.span);
    }

    fn collect_semantic_footnote_definition_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        footnote_def: &FootnoteDefinition<'_>,
        hooks: &mut H,
    ) {
        let index = self.footnote_index_of(footnote_def.identifier);
        if self.footnote_records[index].body_html.is_some() {
            return;
        }
        let start = self.output.len();
        for child in &footnote_def.children {
            self.render_node_with_hooks(child, hooks);
        }
        self.footnote_records[index].body_html = Some(self.output.split_off(start));
        self.footnote_records[index].span = Some(footnote_def.span);
    }

    fn write_semantic_backlinks(&mut self, index: usize, display: usize) {
        let ref_count = self.footnote_records[index].reference_ids.len();
        if ref_count == 0 {
            return;
        }
        for occurrence in 1..=ref_count {
            let reference_id = self.footnote_records[index].reference_ids[occurrence - 1];
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
        let key = CompactString::from(identifier);
        if let Some(&id) = self.legacy_footnote_targets.get(&key) {
            return id;
        }
        self.heading_slug_scratch.clear();
        self.heading_slug_scratch.push_str("fn-");
        self.heading_slug_scratch.push_str(identifier);
        let id = self.heading_id_planner.claim(&self.heading_slug_scratch);
        self.legacy_footnote_targets.insert(key, id);
        id
    }

    fn legacy_footnote_reference_id(&mut self, identifier: &str, occurrence: usize) -> PlannedId {
        if occurrence == 1
            && let Some(&id) = self.legacy_footnote_first_references.get(identifier)
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
            self.legacy_footnote_first_references
                .insert(CompactString::from(identifier), id);
        }
        id
    }

    fn footnote_index_of(&mut self, identifier: &str) -> usize {
        if let Some(&index) = self.footnote_index.get(identifier) {
            return index as usize;
        }
        let index = self.footnote_records.len();
        let slug = self.assign_footnote_slug(identifier, index);
        self.heading_slug_scratch.clear();
        self.heading_slug_scratch.push_str("fn-");
        self.heading_slug_scratch.push_str(&slug);
        let target_id = self.heading_id_planner.claim(&self.heading_slug_scratch);
        self.footnote_index
            .insert(CompactString::from(identifier), index as u32);
        self.footnote_records.push(FootnoteRecord {
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
        self.heading_slug_scratch.clear();
        reserve_heading_scratch(&mut self.heading_slug_scratch);
        slugify_heading_into(identifier, &mut self.heading_slug_scratch);
        if self.heading_slug_scratch == "section" && !identifier.chars().any(char::is_alphanumeric)
        {
            self.heading_slug_scratch.clear();
            let _ = write!(self.heading_slug_scratch, "footnote-{}", index + 1);
        }
        self.uniquify_footnote_slug();
        CompactString::from(self.heading_slug_scratch.as_str())
    }

    fn write_footnote_id(&mut self, id: PlannedId) {
        write_escaped_into(&mut self.output, self.heading_id_planner.id(id));
    }

    /// Turns the slug in `heading_slug_scratch` into one no footnote holds,
    /// appending `-2`, `-3`, ... as the old scan did.
    ///
    /// The map answers "is this slug taken" in one lookup, and remembers
    /// per base slug which suffix the last collision reached — so a run of
    /// identical slugs walks forward instead of restarting at `-2` and
    /// re-testing every earlier one.
    fn uniquify_footnote_slug(&mut self) {
        let base_len = self.heading_slug_scratch.len();
        let Some(&start) = self
            .footnote_slug_counts
            .get(self.heading_slug_scratch.as_str())
        else {
            self.claim_footnote_slug();
            return;
        };
        // Only allocated when a slug actually repeats, which is rare.
        let base = CompactString::from(&self.heading_slug_scratch[..base_len]);
        let mut suffix = start;
        loop {
            self.heading_slug_scratch.truncate(base_len);
            let _ = write!(self.heading_slug_scratch, "-{suffix}");
            suffix += 1;
            if !self
                .footnote_slug_counts
                .contains_key(self.heading_slug_scratch.as_str())
            {
                self.claim_footnote_slug();
                self.footnote_slug_counts.insert(base, suffix);
                return;
            }
        }
    }

    /// Records the scratch slug as taken, starting its own suffix run at 2.
    fn claim_footnote_slug(&mut self) {
        self.footnote_slug_counts.insert(
            CompactString::from(self.heading_slug_scratch.as_str()),
            FIRST_SUFFIX,
        );
    }
}
