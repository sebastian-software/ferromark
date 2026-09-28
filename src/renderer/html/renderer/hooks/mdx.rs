use crate::ast::{MdxJsxAttributeEntry, MdxJsxFlowElement, MdxJsxTextElement, Node};

use super::HtmlRenderHooks;
use crate::renderer::html::escape::write_escaped_into;
use crate::renderer::html::mdx_payload::{collect_island_payload, stringify_xss_safe};
use crate::renderer::html::renderer::HtmlRenderer;

impl HtmlRenderer {
    pub(in crate::renderer::html::renderer) fn render_mdx_jsx_flow_element_with_hooks<
        H: HtmlRenderHooks,
    >(
        &mut self,
        node: &MdxJsxFlowElement<'_>,
        hooks: &mut H,
    ) {
        self.render_mdx_jsx_with_hooks(
            node.name,
            &node.attributes,
            &node.children,
            "div",
            true,
            hooks,
        );
    }

    pub(in crate::renderer::html::renderer) fn render_mdx_jsx_text_element_with_hooks<
        H: HtmlRenderHooks,
    >(
        &mut self,
        node: &MdxJsxTextElement<'_>,
        hooks: &mut H,
    ) {
        self.render_mdx_jsx_with_hooks(
            node.name,
            &node.attributes,
            &node.children,
            "span",
            false,
            hooks,
        );
    }

    fn render_mdx_jsx_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        name: Option<&str>,
        attributes: &[MdxJsxAttributeEntry<'_>],
        children: &[Node<'_>],
        tag: &str,
        block: bool,
        hooks: &mut H,
    ) {
        let Some(name) = name else {
            self.render_mdx_children_with_hooks(children, block, hooks);
            return;
        };

        let previous_child_html = self.in_mdx_island_children;
        let previous_abbreviation_depth = self
            .abbreviation_state
            .as_ref()
            .map(|state| state.mdx_depth)
            .unwrap_or_default();
        if name.eq_ignore_ascii_case("abbr")
            && let Some(state) = self.abbreviation_state.as_mut()
        {
            state.mdx_depth = state.mdx_depth.saturating_add(1);
        }
        self.in_mdx_island_children = false;
        self.output.push('<');
        self.output.push_str(tag);
        self.output
            .push_str(" class=\"ox-island\" data-ox-island=\"");
        write_escaped_into(&mut self.output, name);
        self.output.push('"');

        let payload = collect_island_payload(attributes);
        let json = (!payload.is_empty()).then(|| stringify_xss_safe(&payload.into_json_value()));
        if let Some(json) = json.as_deref() {
            self.output.push_str(" data-ox-props=\"");
            write_escaped_into(&mut self.output, json);
            self.output.push('"');
        }

        self.output.push('>');
        if let Some(json) = json.as_deref() {
            self.output.push_str("<script type=\"application/json\">");
            self.output.push_str(json);
            self.output.push_str("</script>");
        }
        self.in_mdx_island_children = true;
        self.render_mdx_children_with_hooks(children, block, hooks);
        self.output.push_str("</");
        self.output.push_str(tag);
        self.output.push('>');
        self.in_mdx_island_children = previous_child_html;
        if let Some(state) = self.abbreviation_state.as_mut() {
            state.mdx_depth = previous_abbreviation_depth;
        }
        if block {
            self.output.push('\n');
        }
    }

    fn render_mdx_children_with_hooks<H: HtmlRenderHooks>(
        &mut self,
        children: &[Node<'_>],
        block: bool,
        hooks: &mut H,
    ) {
        if block {
            for child in children {
                self.render_node_with_hooks(child, hooks);
            }
        } else {
            self.render_inline_children_with_hooks(children, hooks);
        }
    }
}
