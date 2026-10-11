//! Read-only inventories of Markdown references and literal MDX attributes.
//!
//! Collection follows the parsed tree. It never parses raw HTML, executes MDX
//! expressions, resolves spreads, classifies URLs, or checks destinations.

use rustc_hash::FxHashMap;

use crate::ast::{
    Document, Image, Link, MdxJsxAttributeEntry, MdxJsxAttributeValue, MdxJsxFlowElement,
    MdxJsxTextElement, Node, Span, Visit, walk_link, walk_mdx_jsx_flow_element,
    walk_mdx_jsx_text_element,
};

/// Syntax that produced a reference occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceKind {
    /// Inline, full, collapsed, or shortcut Markdown link.
    Link,
    /// Inline, full, collapsed, or shortcut Markdown image.
    Image,
    /// CommonMark angle autolink or enabled GFM bare autolink.
    Autolink,
    /// Quoted, literal MDX `href` or `src` attribute.
    MdxAttribute,
}

/// Relationship between an occurrence and the original parsed source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceProvenance {
    /// The reference is unchanged from the parser tree.
    Source,
    /// A pass created or changed the reference's destination or display data.
    Transformed,
}

/// One reference occurrence. Repeated destinations remain separate entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentReference {
    /// Syntax origin; MDX attributes remain distinct from Markdown links.
    pub kind: ReferenceKind,
    /// Parser-resolved Markdown URL, or entity-decoded MDX literal.
    pub destination: String,
    /// Available Markdown label or image alternative text. MDX has no label.
    pub text: Option<String>,
    /// Available Markdown title.
    pub title: Option<String>,
    /// MDX element name, including a custom component's authored name.
    pub element: Option<String>,
    /// MDX attribute name (`href` or `src`).
    pub attribute: Option<String>,
    /// Original half-open UTF-8 byte range, present only for source references.
    /// Markdown ranges cover the occurrence's complete syntax; MDX ranges
    /// cover the named attribute, including its quoted value.
    pub span: Option<Span>,
    /// Provenance of the returned destination and display data.
    pub provenance: ReferenceProvenance,
    /// Original occurrence range when a retained reference was changed.
    /// This is a related source location, not a span of the transformed value.
    /// Generated references have no original range.
    pub original_span: Option<Span>,
}

impl Document<'_> {
    /// Collects references from an unmodified parser tree in document order.
    ///
    /// `source` must be the original input used to parse this document. The
    /// source is used only to distinguish Markdown links from autolinks;
    /// definitions, code, raw HTML and dynamic MDX attributes are excluded.
    /// This method does not render, mutate, deduplicate, or cache the tree.
    ///
    /// For a mutable transform pipeline, capture a [`ReferenceSnapshot`]
    /// before running passes and use its [`ReferenceSnapshot::collect`] method
    /// afterward. A bare AST cannot establish a changed node's provenance.
    #[must_use]
    pub fn collect_references(&self, source: &str) -> Vec<DocumentReference> {
        let mut references = Vec::new();
        visit_references(self, source, |_, reference| {
            references.push((reference.span.start, reference.owned()));
        });
        references.sort_by_key(|(start, _)| *start);
        references
            .into_iter()
            .map(|(_, reference)| reference)
            .collect()
    }
}

/// Compact, opt-in source bookkeeping for references across arena transforms.
///
/// Capture this from an unmodified parser tree, then collect from that same
/// document and arena. The snapshot borrows destinations and titles, retaining
/// collected labels for exact comparison without extraction result arrays.
/// It must not be reused
/// with a different document or after resetting its allocator. Replacing a
/// reference node conservatively marks it transformed, even if its text is
/// identical. Retained nodes keep their original syntax kind.
#[derive(Debug)]
pub struct ReferenceSnapshot<'a> {
    originals: FxHashMap<usize, BorrowedReference<'a>>,
}

impl<'a> ReferenceSnapshot<'a> {
    /// Captures reference identities before caller-controlled AST passes.
    #[must_use]
    pub fn capture(document: &Document<'a>, source: &str) -> Self {
        let mut originals = FxHashMap::default();
        visit_references(document, source, |identity, reference| {
            originals.insert(identity, reference);
        });
        Self { originals }
    }

    /// Collects the current tree while preserving honest source provenance.
    ///
    /// Changed or generated nodes have `span: None`. Retained changed nodes
    /// carry `original_span`; generated nodes have no claimed original range.
    #[must_use]
    pub fn collect(&self, document: &Document<'_>, source: &str) -> Vec<DocumentReference> {
        let mut references = Vec::new();
        visit_references(document, source, |identity, reference| {
            let mut entry = reference.owned();
            match self.originals.get(&identity) {
                Some(original) if original.unchanged(&reference) => {
                    entry.kind = original.kind;
                }
                original => {
                    entry.provenance = ReferenceProvenance::Transformed;
                    entry.span = None;
                    entry.original_span = original.map(|original| original.span);
                    if let Some(original) = original {
                        entry.kind = original.kind;
                    } else if entry.kind == ReferenceKind::Autolink {
                        // Source text cannot identify the syntax of a generated link.
                        entry.kind = ReferenceKind::Link;
                    }
                }
            }
            references.push((reference.span.start, entry));
        });
        references.sort_by_key(|(start, _)| *start);
        references
            .into_iter()
            .map(|(_, reference)| reference)
            .collect()
    }
}

#[derive(Debug)]
struct BorrowedReference<'a> {
    kind: ReferenceKind,
    destination: &'a str,
    text: Option<String>,
    title: Option<&'a str>,
    element: Option<&'a str>,
    attribute: Option<&'a str>,
    span: Span,
}

impl BorrowedReference<'_> {
    fn unchanged(&self, current: &BorrowedReference<'_>) -> bool {
        self.destination == current.destination
            && self.text == current.text
            && self.title == current.title
            && self.element == current.element
            && self.attribute == current.attribute
            && self.span == current.span
    }

    fn owned(&self) -> DocumentReference {
        DocumentReference {
            kind: self.kind,
            destination: if self.kind == ReferenceKind::MdxAttribute {
                crate::renderer::decode_html_entities(self.destination)
            } else {
                self.destination.to_owned()
            },
            text: self.text.clone(),
            title: self.title.map(str::to_owned),
            element: self.element.map(str::to_owned),
            attribute: self.attribute.map(str::to_owned),
            span: Some(self.span),
            provenance: ReferenceProvenance::Source,
            original_span: None,
        }
    }
}

fn visit_references<'a>(
    document: &Document<'a>,
    source: &str,
    emit: impl FnMut(usize, BorrowedReference<'a>),
) {
    let mut collector = ReferenceVisitor {
        source,
        emit,
        lifetime: std::marker::PhantomData,
    };
    collector.visit_document(document);
}

struct ReferenceVisitor<'source, 'a, F> {
    source: &'source str,
    emit: F,
    lifetime: std::marker::PhantomData<&'a ()>,
}

impl<'a, F: FnMut(usize, BorrowedReference<'a>)> ReferenceVisitor<'_, 'a, F> {
    fn attributes(&mut self, element: Option<&'a str>, attributes: &[MdxJsxAttributeEntry<'a>]) {
        for entry in attributes {
            let MdxJsxAttributeEntry::Attribute(attribute) = entry else {
                continue;
            };
            if !matches!(attribute.name, "href" | "src") {
                continue;
            }
            let Some(MdxJsxAttributeValue::Literal(destination)) = &attribute.value else {
                continue;
            };
            (self.emit)(
                std::ptr::from_ref(attribute).addr(),
                BorrowedReference {
                    kind: ReferenceKind::MdxAttribute,
                    destination,
                    text: None,
                    title: None,
                    element,
                    attribute: Some(attribute.name),
                    span: attribute.span,
                },
            );
        }
    }
}

impl<'a, F: FnMut(usize, BorrowedReference<'a>)> Visit<'a> for ReferenceVisitor<'_, 'a, F> {
    fn visit_link(&mut self, link: &Link<'a>) {
        let raw = self
            .source
            .get(link.span.start as usize..link.span.end as usize);
        let kind = if raw.is_some_and(|raw| raw.starts_with('[')) {
            ReferenceKind::Link
        } else {
            ReferenceKind::Autolink
        };
        (self.emit)(
            std::ptr::from_ref(link).addr(),
            BorrowedReference {
                kind,
                destination: link.url,
                text: Some(label(&link.children)),
                title: link.title,
                element: None,
                attribute: None,
                span: link.span,
            },
        );
        walk_link(self, link);
    }

    fn visit_image(&mut self, image: &Image<'a>) {
        (self.emit)(
            std::ptr::from_ref(image).addr(),
            BorrowedReference {
                kind: ReferenceKind::Image,
                destination: image.url,
                text: Some(image.alt.to_owned()),
                title: image.title,
                element: None,
                attribute: None,
                span: image.span,
            },
        );
    }

    fn visit_mdx_jsx_flow_element(&mut self, element: &MdxJsxFlowElement<'a>) {
        self.attributes(element.name, &element.attributes);
        walk_mdx_jsx_flow_element(self, element);
    }

    fn visit_mdx_jsx_text_element(&mut self, element: &MdxJsxTextElement<'a>) {
        self.attributes(element.name, &element.attributes);
        walk_mdx_jsx_text_element(self, element);
    }
}

fn label(children: &[Node<'_>]) -> String {
    #[derive(Default)]
    struct Label(String);
    impl Visit<'_> for Label {
        fn visit_text(&mut self, node: &crate::ast::Text<'_>) {
            self.0.push_str(node.value);
        }
        fn visit_inline_code(&mut self, node: &crate::ast::InlineCode<'_>) {
            self.0.push_str(node.value);
        }
        fn visit_inline_math(&mut self, node: &crate::ast::InlineMath<'_>) {
            self.0.push_str(node.value);
        }
        fn visit_image(&mut self, node: &Image<'_>) {
            self.0.push_str(node.alt);
        }
        fn visit_break(&mut self, _: &crate::ast::Break) {
            self.0.push('\n');
        }
    }
    let mut label = Label::default();
    for child in children {
        label.visit_node(child);
    }
    label.0
}
