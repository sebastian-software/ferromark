//! Resolve document-wide definitions using the existing block grammar.
//!
//! A temporary arena holds only block structure; inline parsing is skipped.
//! Copy the collected definition values into the document arena, then discard
//! the temporary tree. The ordinary no-definition path never enters this pass.
use compact_str::CompactString;
use ferromark_allocator::Allocator;
use ferromark_ast::{Definition, FootnoteDefinition, Visit, walk_footnote_definition};

use super::{Parser, ReferenceDef, ReferenceMap};
use crate::ParserOptions;
use crate::parser::footnote::{FootnoteLabels, normalize_footnote_label};

struct Collector<'a> {
    allocator: &'a Allocator,
    definitions: ReferenceMap<'a>,
    labels: FootnoteLabels,
}

impl<'a, 'tree> Visit<'tree> for Collector<'a> {
    fn visit_footnote_definition(&mut self, definition: &FootnoteDefinition<'tree>) {
        self.labels
            .insert(normalize_footnote_label(definition.identifier));
        walk_footnote_definition(self, definition);
    }

    fn visit_definition(&mut self, definition: &Definition<'tree>) {
        self.definitions
            .entry(CompactString::from(definition.identifier))
            .or_insert_with(|| ReferenceDef {
                url: self.allocator.alloc_str(definition.url),
                title: definition
                    .title
                    .map(|title| self.allocator.alloc_str(title)),
            });
    }
}

impl<'a> Parser<'a> {
    pub(in crate::parser) fn collect_definitions(&self) -> (ReferenceMap<'a>, FootnoteLabels) {
        let temporary = Allocator::new();
        // Choose the collection phase at construction. Syntax options retain
        // their real meaning throughout; no temporarily disabled options or
        // post-construction state repair is needed to suppress recursion.
        let mut parser = Parser::with_phase(
            &temporary,
            self.source,
            ParserOptions {
                front_matter: false,
                ..self.options.clone()
            },
            super::super::ParsePhase::Definitions,
        );
        let mut collector = Collector {
            allocator: self.allocator,
            definitions: ReferenceMap::default(),
            labels: FootnoteLabels::default(),
        };
        if let Ok(document) = parser.parse_document() {
            collector.visit_document(&document);
        }
        // If the block grammar rejects the document (for example depth), the
        // real parse reports that error; no successful output uses this map.
        (collector.definitions, collector.labels)
    }
}
