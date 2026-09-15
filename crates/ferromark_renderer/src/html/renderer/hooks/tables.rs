use ferromark_ast::{
    AlignKind, DefinitionList, DefinitionListDefinition, DefinitionListTerm, Node, Table,
    TableCell, TableRow,
};

use super::HtmlRenderHooks;
use crate::html::renderer::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::html::renderer) fn render_table_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        table: &Table<'_>,
        hooks: &mut H,
    ) {
        self.write_table_opening(table);
        if let Some(attributes) = &table.attributes
            && !attributes.caption.is_empty()
        {
            self.write("<caption>");
            for child in &attributes.caption {
                self.render_inline_node_with_hooks(child, hooks);
            }
            self.write("</caption>\n");
        }
        self.write_table_colgroup(table);
        for (i, row) in table.children.iter().enumerate() {
            if i == 0 {
                self.write("<thead>\n");
            } else if i == 1 {
                self.write("<tbody>\n");
            }
            self.render_table_row_with_hooks(row, i == 0, &table.align, hooks);
            if i == 0 {
                self.write("</thead>\n");
            }
        }
        if table.children.len() > 1 {
            self.write("</tbody>\n");
        }
        self.write("</table>\n");
    }

    pub(in crate::html::renderer) fn render_definition_list_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        list: &DefinitionList<'_>,
        hooks: &mut H,
    ) {
        self.write("<dl class=\"ox-definition-list\"");
        self.write_source_span_attr(list.span);
        self.write(">\n");
        for child in &list.children {
            self.render_node_with_hooks(child, hooks);
        }
        self.write("</dl>\n");
    }

    pub(in crate::html::renderer) fn render_definition_list_term_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        term: &DefinitionListTerm<'_>,
        hooks: &mut H,
    ) {
        self.write("<dt");
        self.write_source_span_attr(term.span);
        self.write(">");
        for child in &term.children {
            self.render_inline_node_with_hooks(child, hooks);
        }
        self.write("</dt>\n");
    }

    pub(in crate::html::renderer) fn render_definition_list_definition_with_hooks<
        H: HtmlRenderHooks,
    >(
        &mut self,
        definition: &DefinitionListDefinition<'_>,
        hooks: &mut H,
    ) {
        self.write("<dd");
        self.write_source_span_attr(definition.span);
        if definition.children.len() == 1
            && let Some(Node::Paragraph(paragraph)) = definition.children.first()
        {
            self.write(">");
            for child in &paragraph.children {
                self.render_inline_node_with_hooks(child, hooks);
            }
            self.write("</dd>\n");
            return;
        }
        self.write(">\n");
        for child in &definition.children {
            self.render_node_with_hooks(child, hooks);
        }
        self.write("</dd>\n");
    }

    fn render_table_row_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        row: &TableRow<'_>,
        is_header: bool,
        align: &ferromark_allocator::Vec<'_, AlignKind>,
        hooks: &mut H,
    ) {
        self.write("<tr");
        self.write_source_span_attr(row.span);
        self.write(">\n");
        let tag = if is_header { "th" } else { "td" };
        let mut column_index = 0usize;
        for cell in &row.children {
            self.write_table_cell_open(tag, cell, align.get(column_index).copied());
            self.render_table_cell_with_hooks(cell, hooks);
            self.write("</");
            self.write(tag);
            self.write(">\n");
            column_index = column_index.saturating_add(Self::normalized_table_colspan(cell));
        }
        self.write("</tr>\n");
    }

    fn render_table_cell_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        cell: &TableCell<'_>,
        hooks: &mut H,
    ) {
        for child in &cell.children {
            self.render_inline_node_with_hooks(child, hooks);
        }
    }
}
