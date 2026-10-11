//! Static heading labels shared by JSX output and text analysis.

use crate::ast::Node;

pub fn heading_text(nodes: &[Node<'_>]) -> String {
    let mut text = String::new();
    for node in nodes {
        append_heading_text(node, &mut text);
    }
    text
}

fn append_heading_text(node: &Node<'_>, text: &mut String) {
    match node {
        Node::Text(value) => text.push_str(value.value),
        Node::InlineCode(value) => text.push_str(value.value),
        Node::Emphasis(value) => append_heading_children(&value.children, text),
        Node::Strong(value) => append_heading_children(&value.children, text),
        Node::Highlight(value) => append_heading_children(&value.children, text),
        Node::Delete(value) => append_heading_children(&value.children, text),
        Node::Insertion(value) => append_heading_children(&value.children, text),
        Node::Superscript(value) => append_heading_children(&value.children, text),
        Node::Subscript(value) => append_heading_children(&value.children, text),
        Node::Link(value) => append_heading_children(&value.children, text),
        Node::Span(value) => append_heading_children(&value.children, text),
        Node::MdxJsxFlowElement(value) => append_heading_children(&value.children, text),
        Node::MdxJsxTextElement(value) => append_heading_children(&value.children, text),
        _ => {}
    }
}

fn append_heading_children(nodes: &[Node<'_>], text: &mut String) {
    for node in nodes {
        append_heading_text(node, text);
    }
}
