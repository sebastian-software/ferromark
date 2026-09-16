//! Visitor trait glue for `HtmlRenderer`.
//!
//! Each trait method delegates to a focused helper module. The AST-facing behavior
//! remains concentrated in one impl block, while the larger HTML-writing logic stays
//! split by block and inline responsibilities.

use crate::ast::{
    BlockQuote, Break, CodeBlock, Definition, DefinitionList, DefinitionListDefinition,
    DefinitionListTerm, Delete, Document, Emphasis, FootnoteDefinition, FootnoteReference, Heading,
    Html, Image, InlineCode, InlineMath, Link, List, ListItem, MathBlock, MdxFlowExpression,
    MdxJsxFlowElement, MdxJsxTextElement, MdxTextExpression, MdxjsEsm, Node, Paragraph, Strong,
    Subscript, Superscript, Table, Text, ThematicBreak, Visit,
};

use super::HtmlRenderer;

impl<'a> Visit<'a> for HtmlRenderer {
    fn visit_document(&mut self, document: &Document<'a>) {
        self.render_document(document);
    }

    fn visit_node(&mut self, node: &Node<'a>) {
        self.render_node(node);
    }

    fn visit_paragraph(&mut self, paragraph: &Paragraph<'a>) {
        self.render_paragraph(paragraph);
    }

    fn visit_heading(&mut self, heading: &Heading<'a>) {
        self.render_heading(heading);
    }

    fn visit_thematic_break(&mut self, thematic_break: &ThematicBreak) {
        self.render_thematic_break(thematic_break);
    }

    fn visit_block_quote(&mut self, block_quote: &BlockQuote<'a>) {
        self.render_block_quote(block_quote);
    }

    fn visit_list(&mut self, list: &List<'a>) {
        self.render_list(list);
    }

    fn visit_list_item(&mut self, list_item: &ListItem<'a>) {
        self.render_list_item(list_item);
    }

    fn visit_code_block(&mut self, code_block: &CodeBlock<'a>) {
        self.render_code_block(code_block);
    }

    fn visit_math_block(&mut self, math_block: &MathBlock<'a>) {
        self.render_math_block(math_block);
    }

    fn visit_html(&mut self, html: &Html<'a>) {
        self.render_html(html);
    }

    fn visit_table(&mut self, table: &Table<'a>) {
        self.render_table(table);
    }

    fn visit_definition_list(&mut self, definition_list: &DefinitionList<'a>) {
        self.render_definition_list(definition_list);
    }

    fn visit_definition_list_term(&mut self, definition_list_term: &DefinitionListTerm<'a>) {
        self.render_definition_list_term(definition_list_term);
    }

    fn visit_definition_list_definition(
        &mut self,
        definition_list_definition: &DefinitionListDefinition<'a>,
    ) {
        self.render_definition_list_definition(definition_list_definition);
    }

    fn visit_text(&mut self, text: &Text<'a>) {
        self.render_text(text);
    }

    fn visit_emphasis(&mut self, emphasis: &Emphasis<'a>) {
        self.render_emphasis(emphasis);
    }

    fn visit_strong(&mut self, strong: &Strong<'a>) {
        self.render_strong(strong);
    }

    fn visit_inline_code(&mut self, inline_code: &InlineCode<'a>) {
        self.render_inline_code(inline_code);
    }

    fn visit_inline_math(&mut self, inline_math: &InlineMath<'a>) {
        self.render_inline_math(inline_math);
    }

    fn visit_break(&mut self, break_node: &Break) {
        self.render_break(break_node);
    }

    fn visit_link(&mut self, link: &Link<'a>) {
        self.render_link(link);
    }

    fn visit_image(&mut self, image: &Image<'a>) {
        self.render_image(image);
    }

    fn visit_highlight(&mut self, highlight: &crate::ast::Highlight<'a>) {
        self.render_highlight(highlight);
    }

    fn visit_delete(&mut self, delete: &Delete<'a>) {
        self.render_delete(delete);
    }

    fn visit_superscript(&mut self, superscript: &Superscript<'a>) {
        self.render_superscript(superscript);
    }

    fn visit_subscript(&mut self, subscript: &Subscript<'a>) {
        self.render_subscript(subscript);
    }

    fn visit_footnote_reference(&mut self, footnote_ref: &FootnoteReference<'a>) {
        self.render_footnote_reference(footnote_ref);
    }

    fn visit_definition(&mut self, definition: &Definition<'a>) {
        let _ = definition;
        // Link definitions are lookup metadata for parsers and are not rendered directly.
    }

    fn visit_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'a>) {
        self.render_footnote_definition(footnote_def);
    }

    fn visit_mdx_jsx_flow_element(&mut self, node: &MdxJsxFlowElement<'a>) {
        self.render_mdx_jsx_flow_element(node);
    }

    fn visit_mdx_jsx_text_element(&mut self, node: &MdxJsxTextElement<'a>) {
        self.render_mdx_jsx_text_element(node);
    }

    fn visit_mdxjs_esm(&mut self, _node: &MdxjsEsm<'a>) {}

    fn visit_mdx_flow_expression(&mut self, _node: &MdxFlowExpression<'a>) {}

    fn visit_mdx_text_expression(&mut self, _node: &MdxTextExpression<'a>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::Allocator;
    use crate::ast::{walk_document, walk_node};
    use crate::parser::{Parser, ParserOptions};
    use crate::renderer::HtmlRendererOptions;

    /// The public typed visitor and the renderer's direct dispatch must agree.
    /// Walking every nested node catches a missing typed override even when the
    /// optimized document renderer never calls that override itself.
    struct CheckDispatch<'d, 'a> {
        document: &'d Document<'a>,
        visited: std::collections::HashSet<std::mem::Discriminant<Node<'a>>>,
    }

    impl<'a> Visit<'a> for CheckDispatch<'_, 'a> {
        fn visit_node(&mut self, node: &Node<'a>) {
            self.visited.insert(std::mem::discriminant(node));
            let mut direct = HtmlRenderer::with_options(HtmlRendererOptions::commonmark());
            let mut typed = HtmlRenderer::with_options(HtmlRendererOptions::commonmark());
            direct.prepare_render(self.document);
            typed.prepare_render(self.document);
            direct.visit_node(node);
            walk_node(&mut typed, node);
            assert_eq!(
                typed.output, direct.output,
                "typed visitor lost rendering for {node:?}"
            );
            walk_node(self, node);
        }

        fn visit_list_item(&mut self, item: &ListItem<'a>) {
            // Lists store items directly rather than as Node::ListItem.
            let mut direct = HtmlRenderer::with_options(HtmlRendererOptions::commonmark());
            let mut typed = HtmlRenderer::with_options(HtmlRendererOptions::commonmark());
            direct.prepare_render(self.document);
            typed.prepare_render(self.document);
            direct.render_list_item(item);
            typed.visit_list_item(item);
            assert_eq!(typed.output, direct.output);
            crate::ast::walk_list_item(self, item);
        }
    }

    #[test]
    fn typed_visitor_preserves_every_node_kind() {
        let source = "# Heading\n\n---\n\n> Quote\n\n- Item\n\n```txt\ncode\n```\n\n$$\nx\n$$\n\n<div>raw</div>\n\n| A | B |\n| --- | --- |\n| x | y |\n\nTerm\n: Meaning\n\n*em* **strong** `code` $x$ [link](/url) ![alt](/img) ==mark== ~~old~~ x^2^ H~2~O[^note]  \nbreak\n\n[ref]: /ref\n\n[^note]: Footnote\n\n<Component />\n\nText <Badge /> {inline}\n\n{flow}\n\nexport const x = 1;\n";
        let allocator = Allocator::new();
        let options = ParserOptions {
            mdx: true,
            math: true,
            definition_lists: true,
            highlight: true,
            superscript: true,
            subscript: true,
            ..ParserOptions::gfm()
        };
        let document = Parser::with_options(&allocator, source, options)
            .parse()
            .unwrap();
        let mut check = CheckDispatch {
            document: &document,
            visited: std::collections::HashSet::new(),
        };
        walk_document(&mut check, &document);
        assert_eq!(
            check.visited.len(),
            32,
            "fixture must exercise every parsed Node variant (list items are checked separately)"
        );

        let mut visitor = HtmlRenderer::with_options(HtmlRendererOptions::commonmark());
        visitor.prepare_render(&document);
        visitor.visit_document(&document);
        visitor.finish_render();
        assert_eq!(
            visitor.output,
            HtmlRenderer::with_options(HtmlRendererOptions::commonmark()).render(&document)
        );
        assert!(visitor.output.contains("<mark>mark</mark>"));
    }
}
