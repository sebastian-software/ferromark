use crate::ast::Document;

use super::{HtmlRenderHooks, HtmlRenderer};

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
        // Checkpoint `heading_id_planner` even when `heading_ids` is false:
        // footnotes and explicit element IDs (`{#id}`) share the same planner.
        // Rollback reverts only the provisional delta in O(Δ) without cloning
        // committed IDs, and `footnotes` stays `None` when no footnotes exist.
        let heading_checkpoint = self.heading_id_planner.checkpoint();
        let footnotes = self.footnotes.clone();
        let code_block_index = self.code_block_index;
        let html = self.render_fragment(document);
        self.heading_id_planner.rollback(heading_checkpoint);
        self.footnotes = footnotes;
        self.code_block_index = code_block_index;
        html
    }

    /// Renders an unstable streaming fragment through hooks without mutating committed state.
    #[must_use]
    pub fn render_provisional_fragment_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        document: &Document<'_>,
        hooks: &mut H,
    ) -> String {
        let heading_checkpoint = self.heading_id_planner.checkpoint();
        let footnotes = self.footnotes.clone();
        let code_block_index = self.code_block_index;
        let html = self.render_fragment_with_hooks(document, hooks);
        self.heading_id_planner.rollback(heading_checkpoint);
        self.footnotes = footnotes;
        self.code_block_index = code_block_index;
        html
    }

    /// Clears renderer state that spans incremental fragments.
    pub fn reset_incremental_state(&mut self) {
        self.output.clear();
        self.heading_id_planner.clear();
        self.clear_footnote_state();
        self.heading_text_scratch.clear();
        self.heading_slug_scratch.clear();
        self.code_block_index = 0;
        self.in_link = false;
        self.in_mdx_island_children = false;
        if let Some(state) = self.abbreviation_state.as_mut() {
            state.raw_html_depth = 0;
            state.mdx_depth = 0;
            state.inline_depth = 0;
        }
        // `autolink_index` is deliberately untouched: it is derived from the
        // renderer's immutable options, not from the fragments rendered so
        // far, and every fragment path used to rebuild the identical value.
    }

    fn render_fragment(&mut self, document: &Document<'_>) -> String {
        self.output.clear();
        self.in_link = false;
        self.in_mdx_island_children = false;
        if let Some(state) = self.abbreviation_state.as_mut() {
            state.raw_html_depth = 0;
            state.mdx_depth = 0;
            state.inline_depth = 0;
        }
        self.reserve_output_for(document);
        self.render_document(document);
        self.finish_semantic_footnotes();
        std::mem::take(&mut self.output)
    }

    fn render_fragment_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        document: &Document<'_>,
        hooks: &mut H,
    ) -> String {
        self.node_hooks_enabled = true;
        self.output.clear();
        self.in_link = false;
        self.in_mdx_island_children = false;
        if let Some(state) = self.abbreviation_state.as_mut() {
            state.raw_html_depth = 0;
            state.mdx_depth = 0;
            state.inline_depth = 0;
        }
        self.reserve_output_for(document);
        self.render_document_with_hooks(document, hooks);
        self.finish_semantic_footnotes();
        std::mem::take(&mut self.output)
    }
}

#[cfg(test)]
mod tests {
    use crate::allocator::Allocator;
    use crate::parser::Parser;
    use crate::renderer::html::HtmlRenderer;

    #[test]
    fn heading_id_planner_continues_across_committed_fragments() {
        let first_allocator = Allocator::new();
        let first = Parser::new(&first_allocator, "# a\n\n# a").parse().unwrap();
        let next_allocator = Allocator::new();
        let next = Parser::new(&next_allocator, "# a-1").parse().unwrap();
        let mut renderer = HtmlRenderer::new();

        let first_html = renderer.render_incremental_fragment(&first);
        assert!(first_html.contains("id=\"a\""), "{first_html}");
        assert!(first_html.contains("id=\"a-1\""), "{first_html}");

        let provisional = renderer.render_provisional_fragment(&next);
        assert!(provisional.contains("id=\"a-1-1\""), "{provisional}");
        let committed = renderer.render_incremental_fragment(&next);
        assert_eq!(committed, provisional);

        renderer.reset_incremental_state();
        assert!(
            renderer
                .render_incremental_fragment(&next)
                .contains("id=\"a-1\"")
        );
    }

    #[test]
    fn heading_id_prefix_is_kept_across_committed_and_provisional_fragments() {
        let first_allocator = Allocator::new();
        let first = Parser::new(&first_allocator, "# a").parse().unwrap();
        let next_allocator = Allocator::new();
        let next = Parser::new(&next_allocator, "# a").parse().unwrap();
        let mut renderer = HtmlRenderer::new()
            .try_with_heading_id_prefix("docs-")
            .unwrap();

        assert!(
            renderer
                .render_incremental_fragment(&first)
                .contains("id=\"docs-a\"")
        );
        let provisional = renderer.render_provisional_fragment(&next);
        assert!(provisional.contains("id=\"docs-a-1\""), "{provisional}");
        let committed = renderer.render_incremental_fragment(&next);
        assert_eq!(committed, provisional);

        renderer.reset_incremental_state();
        assert!(
            renderer
                .render_incremental_fragment(&next)
                .contains("id=\"docs-a\"")
        );
    }

    #[test]
    fn provisional_rollback_restores_duplicate_suffix_counters_and_footnotes() {
        let a1 = Allocator::new();
        let committed_doc = Parser::new(&a1, "# a\n\n# a-1").parse().unwrap();
        let a2 = Allocator::new();
        let provisional_doc = Parser::new(&a2, "# a\n\n# a\n\nText[^1]\n\n[^1]: Note")
            .parse()
            .unwrap();
        let a3 = Allocator::new();
        let next_doc = Parser::new(&a3, "# a").parse().unwrap();

        let mut renderer = HtmlRenderer::new();
        let first_html = renderer.render_incremental_fragment(&committed_doc);
        assert!(first_html.contains("id=\"a\""), "{first_html}");
        assert!(first_html.contains("id=\"a-1\""), "{first_html}");

        let provisional_html = renderer.render_provisional_fragment(&provisional_doc);
        assert!(
            provisional_html.contains("id=\"a-2\""),
            "{provisional_html}"
        );
        assert!(
            provisional_html.contains("id=\"a-3\""),
            "{provisional_html}"
        );

        // After rolling back the provisional fragment, the next committed `# a`
        // must still claim `a-2` (not `a-4`), and repeated provisional renders
        // must produce identical output.
        let provisional_again = renderer.render_provisional_fragment(&provisional_doc);
        assert_eq!(provisional_again, provisional_html);

        let next_html = renderer.render_incremental_fragment(&next_doc);
        assert!(next_html.contains("id=\"a-2\""), "{next_html}");
    }

    #[test]
    fn provisional_rollback_restores_empty_explicit_heading_id_counter() {
        use crate::ast::{Document, ElementAttributes, Heading, Node, Span};

        let allocator = Allocator::new();
        let make_doc = || {
            let mut children = allocator.new_vec();
            children.push(Node::Heading(allocator.boxed(Heading {
                depth: 1,
                attributes: Some(allocator.boxed(ElementAttributes {
                    id: Some(""),
                    classes: allocator.new_vec(),
                    values: allocator.new_vec(),
                })),
                children: allocator.new_vec(),
                span: Span::new(0, 0),
            })));
            Document {
                front_matter: None,
                children,
                span: Span::new(0, 0),
            }
        };
        let doc = make_doc();
        let mut renderer = HtmlRenderer::new();

        let first = renderer.render_incremental_fragment(&doc);
        assert!(first.contains("id=\"\""), "{first}");

        let provisional = renderer.render_provisional_fragment(&doc);
        assert!(provisional.contains("id=\"-1\""), "{provisional}");

        let second = renderer.render_incremental_fragment(&doc);
        assert!(second.contains("id=\"-1\""), "{second}");
    }

    #[test]
    fn provisional_noncanonical_suffix_does_not_rewind_base_counter() {
        use crate::ast::{Document, ElementAttributes, Heading, Node, Span};

        let allocator = Allocator::new();
        let make_doc = |id: &'static str| {
            let mut children = allocator.new_vec();
            children.push(Node::Heading(allocator.boxed(Heading {
                depth: 1,
                attributes: Some(allocator.boxed(ElementAttributes {
                    id: Some(id),
                    classes: allocator.new_vec(),
                    values: allocator.new_vec(),
                })),
                children: allocator.new_vec(),
                span: Span::new(0, 0),
            })));
            Document {
                front_matter: None,
                children,
                span: Span::new(0, 0),
            }
        };
        let empty_doc = make_doc("");
        let leading_zero = make_doc("-01");
        let plus_sign = make_doc("-+1");

        let mut renderer = HtmlRenderer::new();
        for _ in 0..4 {
            let _ = renderer.render_incremental_fragment(&empty_doc);
        }
        let before = renderer.heading_id_planner.clone();
        let _ = renderer.render_provisional_fragment(&leading_zero);
        let _ = renderer.render_provisional_fragment(&plus_sign);
        // Rolling back `-01` or `-+1` must leave `""`'s internal `next_suffix`
        // untouched so the next `""` claim does not re-probe `-1`, `-2`, `-3`.
        assert_eq!(
            format!("{:?}", renderer.heading_id_planner),
            format!("{before:?}")
        );
        let next = renderer.render_incremental_fragment(&empty_doc);
        assert!(next.contains("id=\"-4\""), "{next}");
    }
}
