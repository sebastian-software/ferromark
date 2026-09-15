use crate::ast::Document;

use super::{HtmlRenderHooks, HtmlRenderer};
use crate::renderer::html::toc::{
    DocumentRenderScan, collect_inline_toc_entries, scan_document_for_render,
};

impl HtmlRenderer {
    /// Renders a committed streaming fragment while preserving cross-fragment state.
    ///
    /// This is intentionally separate from [`Self::render`], so normal one-shot
    /// rendering keeps its exact setup cost and behavior. Incremental callers use
    /// this to preserve heading ID de-duplication across committed fragments.
    #[must_use]
    pub fn render_incremental_fragment(&mut self, document: &Document<'_>) -> String {
        self.render_fragment(document)
    }

    /// Renders a committed streaming fragment through opt-in per-node hooks.
    ///
    /// This preserves the same cross-fragment heading and footnote state as
    /// [`Self::render_incremental_fragment`].
    #[must_use]
    pub fn render_incremental_fragment_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        document: &Document<'_>,
        hooks: &mut H,
    ) -> String {
        self.render_fragment_with_hooks(document, hooks)
    }

    /// Renders an unstable streaming fragment without mutating committed state.
    ///
    /// The returned HTML is meant to be replaceable by the next streaming update.
    #[must_use]
    pub fn render_provisional_fragment(&mut self, document: &Document<'_>) -> String {
        let document_scan = self.scan_for_fragment(document);
        let heading_id_counts =
            (document_scan.heading_count != 0).then(|| self.heading_id_counts.clone());
        let footnote_ref_counts = self.footnote_ref_counts.clone();
        let footnote_index = self.footnote_index.clone();
        let footnote_records = self.footnote_records.clone();
        let footnote_slug_counts = self.footnote_slug_counts.clone();
        let html = self.render_fragment_with_scan(document, document_scan);
        if let Some(heading_id_counts) = heading_id_counts {
            self.heading_id_counts = heading_id_counts;
        }
        self.footnote_ref_counts = footnote_ref_counts;
        self.footnote_index = footnote_index;
        self.footnote_records = footnote_records;
        self.footnote_slug_counts = footnote_slug_counts;
        html
    }

    /// Renders an unstable streaming fragment through hooks without mutating committed state.
    #[must_use]
    pub fn render_provisional_fragment_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        document: &Document<'_>,
        hooks: &mut H,
    ) -> String {
        let document_scan = self.scan_for_fragment(document);
        let heading_id_counts =
            (document_scan.heading_count != 0).then(|| self.heading_id_counts.clone());
        let footnote_ref_counts = self.footnote_ref_counts.clone();
        let footnote_index = self.footnote_index.clone();
        let footnote_records = self.footnote_records.clone();
        let footnote_slug_counts = self.footnote_slug_counts.clone();
        let html = self.render_fragment_with_scan_and_hooks(document, document_scan, hooks);
        if let Some(heading_id_counts) = heading_id_counts {
            self.heading_id_counts = heading_id_counts;
        }
        self.footnote_ref_counts = footnote_ref_counts;
        self.footnote_index = footnote_index;
        self.footnote_records = footnote_records;
        self.footnote_slug_counts = footnote_slug_counts;
        html
    }

    /// Clears renderer state that spans incremental fragments.
    pub fn reset_incremental_state(&mut self) {
        self.output.clear();
        self.heading_id_counts.clear();
        self.clear_footnote_state();
        self.toc_entries.clear();
        self.document_has_toc_marker = false;
        self.heading_text_scratch.clear();
        self.heading_slug_scratch.clear();
        self.in_link = false;
        // `autolink_index` is deliberately untouched: it is derived from the
        // renderer's immutable options, not from the fragments rendered so
        // far, and every fragment path used to rebuild the identical value.
    }

    /// Scans a fragment for the facts its setup consumes.
    ///
    /// Unlike the one-shot render path this always walks: the provisional
    /// entry points use the exact heading count to decide whether committed
    /// heading-ID state has to be snapshotted, and an approximation there
    /// would change what a provisional render leaves behind. Only the
    /// per-paragraph marker predicate is skipped, and only when the renderer
    /// would ignore a marker anyway.
    fn scan_for_fragment(&self, document: &Document<'_>) -> DocumentRenderScan {
        scan_document_for_render(
            document,
            self.options.inline_toc && self.options.heading_ids,
        )
    }

    fn render_fragment(&mut self, document: &Document<'_>) -> String {
        let document_scan = self.scan_for_fragment(document);
        self.render_fragment_with_scan(document, document_scan)
    }

    fn render_fragment_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        document: &Document<'_>,
        hooks: &mut H,
    ) -> String {
        let document_scan = self.scan_for_fragment(document);
        self.render_fragment_with_scan_and_hooks(document, document_scan, hooks)
    }

    fn render_fragment_with_scan(
        &mut self,
        document: &Document<'_>,
        document_scan: DocumentRenderScan,
    ) -> String {
        self.output.clear();
        self.toc_entries.clear();
        self.document_has_toc_marker = document_scan.has_toc_marker;
        if self.document_has_toc_marker {
            collect_inline_toc_entries(document, self.options.toc_max_depth, &mut self.toc_entries);
        }
        self.heading_id_counts.reserve(document_scan.heading_count);
        self.in_link = false;
        let estimated_len = (document.span.len() as usize).saturating_mul(2);
        if self.output.capacity() < estimated_len {
            self.output.reserve(estimated_len - self.output.capacity());
        }
        self.render_document(document);
        self.finish_semantic_footnotes();
        std::mem::take(&mut self.output)
    }

    fn render_fragment_with_scan_and_hooks<H: HtmlRenderHooks>(
        &mut self,
        document: &Document<'_>,
        document_scan: DocumentRenderScan,
        hooks: &mut H,
    ) -> String {
        self.output.clear();
        self.toc_entries.clear();
        self.document_has_toc_marker = document_scan.has_toc_marker;
        if self.document_has_toc_marker {
            collect_inline_toc_entries(document, self.options.toc_max_depth, &mut self.toc_entries);
        }
        self.heading_id_counts.reserve(document_scan.heading_count);
        self.in_link = false;
        let estimated_len = (document.span.len() as usize).saturating_mul(2);
        if self.output.capacity() < estimated_len {
            self.output.reserve(estimated_len - self.output.capacity());
        }
        self.render_document_with_hooks(document, hooks);
        self.finish_semantic_footnotes();
        std::mem::take(&mut self.output)
    }
}
