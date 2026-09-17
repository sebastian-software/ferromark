//! Resolve document-wide definitions using the existing block grammar.
//!
//! A temporary arena holds only block structure; inline parsing is skipped.
//! Copy the collected definition values into the document arena, then discard
//! the temporary tree. The ordinary no-definition path never enters this pass.
//!
//! The pass runs over the segments the pre-pass planner hands it rather than
//! over the whole document (see `super::super::prepass::segments` and
//! `docs/decisions/2026-09-16-segmented-definition-pass.md`). Each segment
//! starts and ends at a line where the real parser is at the document root
//! with everything closed, so a segment parse sees the same blocks the full
//! parse sees there. Segments arrive in document order and share one
//! collector, which is what keeps first-definition precedence intact.
use crate::allocator::Allocator;
use crate::ast::{Definition, FootnoteDefinition, Visit, walk_footnote_definition};
use compact_str::CompactString;

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
    /// Collects the definitions of `segments`, given in ascending order.
    ///
    /// Front matter belongs to the original document start only, so it is off
    /// for every segment, including the one covering the whole body.
    pub(in crate::parser) fn collect_definitions(
        &self,
        segments: &[(usize, usize)],
    ) -> (ReferenceMap<'a>, FootnoteLabels) {
        let temporary = Allocator::new();
        // Choose the collection phase at construction. Syntax options retain
        // their real meaning throughout; no temporarily disabled options or
        // post-construction state repair is needed to suppress recursion.
        let options = ParserOptions {
            front_matter: false,
            ..self.options.clone()
        };
        let mut collector = Collector {
            allocator: self.allocator,
            definitions: ReferenceMap::default(),
            labels: FootnoteLabels::default(),
        };
        for &(start, end) in segments {
            let mut parser = Parser::with_phase(
                &temporary,
                &self.source[start..end],
                options.clone(),
                super::super::ParsePhase::Definitions,
            );
            // If the block grammar rejects a segment (for example depth), the
            // real parse reports that error; no successful output uses this
            // map, so stop with whatever earlier segments contributed.
            let Ok(document) = parser.parse_document() else {
                break;
            };
            collector.visit_document(&document);
        }
        (collector.definitions, collector.labels)
    }
}
