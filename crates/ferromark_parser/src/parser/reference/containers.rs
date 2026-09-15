//! Resolve container definitions using the existing block grammar.
//!
//! A temporary arena holds only block structure; inline parsing is skipped.
//! Copy the collected definition values into the document arena, then discard
//! the temporary tree. The ordinary no-definition path never enters this pass.
use compact_str::CompactString;
use ferromark_allocator::Allocator;
use ferromark_ast::{Definition, Visit};

use super::{Parser, ReferenceDef, ReferenceMap};
use crate::ParserOptions;

struct Collector<'a> {
    allocator: &'a Allocator,
    definitions: ReferenceMap<'a>,
}

impl<'a, 'tree> Visit<'tree> for Collector<'a> {
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
    pub(in crate::parser) fn collect_container_references(&self) -> ReferenceMap<'a> {
        let temporary = Allocator::new();
        // Disable collectors during construction to avoid recursively invoking
        // this pass. The source is already normalized and front matter removed.
        let mut parser = Parser::with_options(
            &temporary,
            self.source,
            ParserOptions {
                allow_link_refs: false,
                footnotes: false,
                inline_footnotes: false,
                front_matter: false,
                ..self.options.clone()
            },
        );
        parser.options = ParserOptions {
            front_matter: false,
            ..self.options.clone()
        };
        parser.collecting_references = true;
        let mut collector = Collector {
            allocator: self.allocator,
            definitions: ReferenceMap::default(),
        };
        if let Ok(document) = parser.parse_document() {
            collector.visit_document(&document);
        }
        // If the block grammar rejects the document (for example depth), the
        // real parse reports that error; no successful output uses this map.
        collector.definitions
    }
}
