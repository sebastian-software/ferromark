use crate::ast::{Delete, Emphasis, Highlight, Link, Strong, Subscript, Superscript};

use super::HtmlRenderHooks;
use crate::renderer::html::renderer::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::renderer::html::renderer) fn render_emphasis_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        emphasis: &Emphasis<'_>,
        hooks: &mut H,
    ) {
        self.write("<em>");
        self.render_inline_children_with_hooks(&emphasis.children, hooks);
        self.write("</em>");
    }

    pub(in crate::renderer::html::renderer) fn render_strong_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        strong: &Strong<'_>,
        hooks: &mut H,
    ) {
        self.write("<strong>");
        self.render_inline_children_with_hooks(&strong.children, hooks);
        self.write("</strong>");
    }

    pub(in crate::renderer::html::renderer) fn render_link_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        link: &Link<'_>,
        hooks: &mut H,
    ) {
        self.write_link_open(link);
        let prev_in_link = self.in_link;
        self.in_link = true;
        self.render_inline_children_with_hooks(&link.children, hooks);
        self.in_link = prev_in_link;
        self.write("</a>");
    }

    /// Visits highlighted text and its children.
    pub(in crate::renderer::html::renderer) fn render_highlight_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        highlight: &Highlight<'_>,
        hooks: &mut H,
    ) {
        self.write("<mark>");
        self.render_inline_children_with_hooks(&highlight.children, hooks);
        self.write("</mark>");
    }

    pub(in crate::renderer::html::renderer) fn render_delete_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        delete: &Delete<'_>,
        hooks: &mut H,
    ) {
        self.write("<del>");
        self.render_inline_children_with_hooks(&delete.children, hooks);
        self.write("</del>");
    }

    pub(in crate::renderer::html::renderer) fn render_superscript_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        superscript: &Superscript<'_>,
        hooks: &mut H,
    ) {
        self.write("<sup>");
        self.render_inline_children_with_hooks(&superscript.children, hooks);
        self.write("</sup>");
    }

    pub(in crate::renderer::html::renderer) fn render_subscript_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        subscript: &Subscript<'_>,
        hooks: &mut H,
    ) {
        self.write("<sub>");
        self.render_inline_children_with_hooks(&subscript.children, hooks);
        self.write("</sub>");
    }
}
