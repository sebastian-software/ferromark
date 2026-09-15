//! Inline notes lower to the existing reference/definition AST at the root.
//! No numbering or definition side effects occur during speculative link parsing.

use ferromark_allocator::Vec;
use ferromark_ast::{Document, FootnoteDefinition, FootnoteReference, Node, Paragraph, Span};

use ferromark_ast::Visit;
use rustc_hash::FxHashSet;

use super::Parser;

#[derive(Default)]
struct ExplicitLabels<'a>(FxHashSet<&'a str>);

impl<'a> Visit<'a> for ExplicitLabels<'a> {
    fn visit_footnote_definition(&mut self, definition: &FootnoteDefinition<'a>) {
        if definition.label.is_some() {
            self.0.insert(definition.identifier);
        }
        ferromark_ast::walk_footnote_definition(self, definition);
    }
}
use crate::error::ParseResult;

impl<'a> Parser<'a> {
    pub(super) fn parse_inline_footnote(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let start = *pos;
        let body_start = start + 2;
        let has_closer = self.has_closer_from(content, body_start, b']');
        if has_closer {
            let (close, _) = Self::scan_balanced(content, body_start);
            if close < content.len() && close > body_start {
                let body = &content[body_start..close];
                // An inline note cannot cross a paragraph boundary. Block parsing
                // usually enforces this already; this also covers inline callers.
                if !body.lines().any(|line| line.trim().is_empty()) {
                    let mut parser =
                        self.sub_parser_with_lazy_lines(body, rustc_hash::FxHashSet::default());
                    parser.options.inline_footnotes = false;
                    let inline = parser.parse_inline_block(body, offset + body_start)?;
                    let span = Span::new((offset + start) as u32, (offset + close + 1) as u32);
                    let mut blocks = self.allocator.new_vec_with_capacity(1);
                    blocks.push(Node::Paragraph(self.allocator.boxed(Paragraph {
                        children: inline,
                        span: Span::new((offset + body_start) as u32, (offset + close) as u32),
                    })));
                    // Only internal inline placeholders have no label. They are
                    // lifted after all speculative parses and span remapping.
                    children.push(Node::FootnoteDefinition(self.allocator.boxed(
                        FootnoteDefinition {
                            identifier: "",
                            label: None,
                            children: blocks,
                            span,
                        },
                    )));
                    if let Some(seen) = self.inline_note_seen {
                        seen.set(true);
                    }
                    *pos = close + 1;
                    return Ok(());
                }
            }
        }
        // With no closing bracket anywhere, the following `[` cannot open
        // a link either. Consume both literal bytes in one step instead of
        // re-entering the link parser and repeating the same cache lookup.
        if !has_closer {
            Self::push_text(children, "^[", offset + start, offset + body_start);
            *pos = body_start;
            return Ok(());
        }
        Self::push_text(children, "^", offset + start, offset + start + 1);
        *pos += 1;
        Ok(())
    }

    pub(super) fn lower_inline_footnotes(&self, document: &mut Document<'a>) {
        // The reference prepass is only a lookup aid: it does not collect
        // every nested definition that can survive block parsing. Use the
        // final AST to avoid identifier collisions in quotes and lists too.
        let mut explicit = ExplicitLabels::default();
        explicit.visit_document(document);
        let mut definitions = self.allocator.new_vec();
        let mut next_id = 1;
        for node in &mut document.children {
            self.lift_inline_note(node, &mut definitions, &mut next_id, &explicit.0);
        }
        document.children.extend(definitions);
    }

    fn lift_inline_note(
        &self,
        node: &mut Node<'a>,
        definitions: &mut Vec<'a, Node<'a>>,
        next_id: &mut usize,
        explicit: &FxHashSet<&str>,
    ) {
        if let Node::FootnoteDefinition(definition) = node
            && definition.label.is_none()
        {
            let identifier = loop {
                let candidate = compact_str::format_compact!("{next_id}");
                *next_id += 1;
                if !explicit.contains(candidate.as_str()) {
                    break self.allocator.alloc_str(&candidate) as &'a str;
                }
            };
            definition.identifier = identifier;
            let reference = Node::FootnoteReference(self.allocator.boxed(FootnoteReference {
                identifier,
                label: None,
                span: definition.span,
            }));
            definitions.push(core::mem::replace(node, reference));
            return;
        }
        match node {
            Node::Paragraph(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::Heading(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::BlockQuote(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::ListItem(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::DefinitionList(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::DefinitionListTerm(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::DefinitionListDefinition(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::Emphasis(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::Strong(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::Link(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::Highlight(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::Delete(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::Superscript(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::Subscript(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::FootnoteDefinition(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::MdxJsxFlowElement(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::MdxJsxTextElement(n) => {
                for child in &mut n.children {
                    self.lift_inline_note(child, definitions, next_id, explicit);
                }
            }
            Node::List(n) => {
                for item in &mut n.children {
                    for child in &mut item.children {
                        self.lift_inline_note(child, definitions, next_id, explicit);
                    }
                }
            }
            Node::Table(n) => {
                if let Some(attributes) = &mut n.attributes {
                    for child in &mut attributes.caption {
                        self.lift_inline_note(child, definitions, next_id, explicit);
                    }
                }
                for row in &mut n.children {
                    for cell in &mut row.children {
                        for child in &mut cell.children {
                            self.lift_inline_note(child, definitions, next_id, explicit);
                        }
                    }
                }
            }
            Node::Text(_)
            | Node::ThematicBreak(_)
            | Node::CodeBlock(_)
            | Node::MathBlock(_)
            | Node::Html(_)
            | Node::InlineCode(_)
            | Node::InlineMath(_)
            | Node::Break(_)
            | Node::Image(_)
            | Node::FootnoteReference(_)
            | Node::Definition(_)
            | Node::MdxjsEsm(_)
            | Node::MdxFlowExpression(_)
            | Node::MdxTextExpression(_) => {}
        }
    }
}
