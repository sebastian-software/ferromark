use crate::ast::{ListItem, Node, Span};

use super::super::Parser;
use super::SpanMap;

impl<'a> Parser<'a> {
    fn remap_span(span: &mut Span, source_map: &impl SpanMap) {
        *span = source_map.map_span(*span);
    }

    fn remap_inline_span(span: &mut Span, source_map: &impl SpanMap) {
        *span = source_map.map_inline_span(*span);
    }

    pub(in crate::parser) fn remap_node_spans(node: &mut Node<'a>, source_map: &impl SpanMap) {
        match node {
            Node::Paragraph(node) => {
                Self::remap_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::Heading(node) => {
                Self::remap_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::ThematicBreak(node) => Self::remap_span(&mut node.span, source_map),
            Node::BlockQuote(node) => {
                Self::remap_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::List(node) => {
                Self::remap_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_list_item_spans(child, source_map);
                }
            }
            Node::ListItem(node) => Self::remap_list_item_spans(node, source_map),
            Node::CodeBlock(node) => Self::remap_span(&mut node.span, source_map),
            Node::MathBlock(node) => Self::remap_span(&mut node.span, source_map),
            Node::Html(node) => Self::remap_span(&mut node.span, source_map),
            Node::Table(node) => {
                Self::remap_span(&mut node.span, source_map);
                if let Some(attributes) = &mut node.attributes {
                    for child in &mut attributes.caption {
                        Self::remap_node_spans(child, source_map);
                    }
                }
                for row in &mut node.children {
                    Self::remap_span(&mut row.span, source_map);
                    for cell in &mut row.children {
                        Self::remap_span(&mut cell.span, source_map);
                        for child in &mut cell.children {
                            Self::remap_node_spans(child, source_map);
                        }
                    }
                }
            }
            Node::DefinitionList(node) => {
                Self::remap_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::DefinitionListTerm(node) => {
                Self::remap_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::DefinitionListDefinition(node) => {
                Self::remap_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::Text(node) => Self::remap_inline_span(&mut node.span, source_map),
            Node::Emphasis(node) => {
                Self::remap_inline_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::Strong(node) => {
                Self::remap_inline_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::InlineCode(node) => Self::remap_inline_span(&mut node.span, source_map),
            Node::InlineMath(node) => Self::remap_inline_span(&mut node.span, source_map),
            Node::Break(node) => Self::remap_inline_span(&mut node.span, source_map),
            Node::Link(node) => {
                Self::remap_inline_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::Image(node) => Self::remap_inline_span(&mut node.span, source_map),
            Node::Highlight(node) => {
                Self::remap_inline_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::Delete(node) => {
                Self::remap_inline_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::Superscript(node) => {
                Self::remap_inline_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::Subscript(node) => {
                Self::remap_inline_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::FootnoteReference(node) => Self::remap_inline_span(&mut node.span, source_map),
            Node::Definition(node) => Self::remap_span(&mut node.span, source_map),
            Node::FootnoteDefinition(node) => {
                Self::remap_span(&mut node.span, source_map);
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::MdxJsxFlowElement(node) => {
                Self::remap_span(&mut node.span, source_map);
                for attribute in &mut node.attributes {
                    Self::remap_mdx_attribute_entry(attribute, source_map);
                }
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::MdxJsxTextElement(node) => {
                Self::remap_inline_span(&mut node.span, source_map);
                for attribute in &mut node.attributes {
                    Self::remap_mdx_attribute_entry(attribute, source_map);
                }
                for child in &mut node.children {
                    Self::remap_node_spans(child, source_map);
                }
            }
            Node::MdxjsEsm(node) => Self::remap_span(&mut node.span, source_map),
            Node::MdxFlowExpression(node) => Self::remap_span(&mut node.span, source_map),
            Node::MdxTextExpression(node) => Self::remap_inline_span(&mut node.span, source_map),
        }
    }

    fn remap_mdx_attribute_entry(
        entry: &mut crate::ast::MdxJsxAttributeEntry<'a>,
        source_map: &impl SpanMap,
    ) {
        match entry {
            crate::ast::MdxJsxAttributeEntry::Attribute(attribute) => {
                Self::remap_span(&mut attribute.span, source_map);
                if let Some(crate::ast::MdxJsxAttributeValue::Expression(expr)) =
                    &mut attribute.value
                {
                    Self::remap_span(&mut expr.span, source_map);
                }
            }
            crate::ast::MdxJsxAttributeEntry::Expression(expr) => {
                Self::remap_span(&mut expr.span, source_map);
            }
        }
    }

    fn remap_list_item_spans(list_item: &mut ListItem<'a>, source_map: &impl SpanMap) {
        Self::remap_span(&mut list_item.span, source_map);
        for child in &mut list_item.children {
            Self::remap_node_spans(child, source_map);
        }
    }
}
