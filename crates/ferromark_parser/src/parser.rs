//! Markdown parser implementation.

use ferromark_allocator::Allocator;
use ferromark_ast::{Document, Span};

use crate::error::ParseResult;

mod block;
mod block_quote;
mod byte_class;
mod cursor;
mod definition_list;
mod fenced_code;
mod footnote;
mod front_matter;
mod html;
mod indented_code;
mod inline;
mod inline_footnote;
mod inline_helpers;
mod inline_html;
mod inline_link;
mod leaf;
mod line_comments;
mod line_scan;
mod list;
mod list_item;
mod math;
mod mdx_esm;
mod mdx_jsx;
mod options;
mod prepass;
mod reference;
mod source_normalization;
mod spans;
mod table;
mod table_attributes;
mod table_cell_source;

#[cfg(test)]
mod tests;

pub use options::ParserOptions;

/// Markdown parser.
pub struct Parser<'a> {
    /// Arena allocator.
    allocator: &'a Allocator,

    /// Source text.
    source: &'a str,

    /// Root-only map from normalized Markdown offsets to the original source.
    source_map: Option<&'a source_normalization::NormalizedSourceMap<'a>>,

    /// Raw document metadata, already in original source coordinates.
    front_matter: Option<ferromark_allocator::Box<'a, ferromark_ast::FrontMatter<'a>>>,

    /// Parser options.
    options: ParserOptions,

    /// Current position in the source.
    position: usize,

    /// Current nesting depth.
    nesting_depth: usize,

    /// Link reference definitions collected by the root parser's
    /// pre-pass, shared with sub-parsers (block quote and list item
    /// contents) so references resolve document-wide.
    ///
    /// `None` and an empty map mean the same thing to every reader; the
    /// distinction exists so that documents without definitions — the
    /// overwhelming majority, and the ones where per-call cost is most
    /// visible — never pay for the `Rc` allocation at all. The same applies
    /// to the two collections below.
    definitions: Option<std::rc::Rc<reference::ReferenceMap<'a>>>,

    /// Footnote labels defined anywhere in the document, collected by the
    /// same kind of pre-pass as `definitions` so an inline `[^x]` can tell
    /// whether a definition exists before reaching it.
    footnote_labels: Option<std::rc::Rc<footnote::FootnoteLabels>>,

    /// Shared by container sub-parsers. Allocated in the existing arena only
    /// when inline notes are enabled; avoids a source scan or AST walk when
    /// no inline note was parsed. Speculative parses may set it harmlessly.
    inline_note_seen: Option<&'a std::cell::Cell<bool>>,

    /// Byte offsets (in `source`) of lines that entered this sub-source
    /// via lazy continuation. Such lines are paragraph text by
    /// construction and must not be reinterpreted as setext underlines
    /// during the re-parse.
    lazy_lines: Option<std::rc::Rc<rustc_hash::FxHashSet<u32>>>,

    /// Eligible physical comment lines carried into a stripped sub-source.
    /// `None` at the root means inspect physical line prefixes directly.
    comment_lines: Option<std::rc::Rc<rustc_hash::FxHashSet<u32>>>,

    /// Next possible definition-list body marker in this parser's source.
    /// A cached exhausted suffix avoids rescanning it for every paragraph.
    definition_marker: std::cell::Cell<Option<(usize, usize)>>,

    /// Memoized "this bracket text already contains a link" verdicts,
    /// keyed by the address and length of the bracketed slice.
    ///
    /// CommonMark forbids a link inside a link, so `parse_link` has to
    /// parse the bracket text before it can decide whether the outer
    /// bracket is a link at all. When it is not, the bracket stays literal
    /// and the caller re-scans the same bytes, parsing that inner text a
    /// second time. Without memoization every nesting level doubles the
    /// work, so `[[[[a](u)](u)](u)]...` costs 2^depth — a 200-byte
    /// document already runs for minutes.
    ///
    /// Every slice lives in the source or the arena, both of which outlive
    /// the parser, so an address plus a length names one byte range for as
    /// long as the cache exists.
    link_probe_cache: std::cell::RefCell<rustc_hash::FxHashMap<(usize, usize), bool>>,

    /// Memoized position of the final `]` or `}` in a content slice, keyed
    /// the same way as `link_probe_cache` plus the byte being looked for.
    ///
    /// An opener can only open something when its closer follows it, and
    /// the balanced scans answer that by walking to the end of the content.
    /// A run of openers with no closer therefore paid one full walk each:
    /// 64 KiB of `[ ` took 1.0 s and 32 KiB of `{ ` took 0.26 s, both
    /// growing x16 for every x4 of input. The position of the last closer
    /// settles it for every opener in the slice at once, so the run costs
    /// one scan in total.
    last_closer: std::cell::RefCell<rustc_hash::FxHashMap<(usize, usize, u8), Option<usize>>>,

    /// The last `[scanned_from, blank_line)` window found while bounding a
    /// link reference definition, so a run of them costs one scan in total.
    ///
    /// A definition may not contain a blank line, so `try_parse_definition_node`
    /// cuts its candidate region at the next one. Scanning for that from each
    /// definition made a document that is nothing but definitions quadratic:
    /// 16,000 of them (197 KB) took 567 ms against 0.3 ms for the same bytes
    /// of prose. Every definition in one run shares the same boundary, and
    /// block parsing walks forward, so the previous answer stays valid for
    /// any start inside the window.
    definition_region: Option<(usize, usize)>,

    /// Reuse a comment-stripped definition region across consecutive definitions.
    comment_definition_region: Option<std::rc::Rc<line_comments::CommentDefinitionRegion<'a>>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser with default options.
    #[must_use]
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self::with_options(allocator, source, ParserOptions::default())
    }

    /// Creates a new parser with the specified options.
    #[must_use]
    pub fn with_options(allocator: &'a Allocator, source: &'a str, options: ParserOptions) -> Self {
        let front_matter = options
            .front_matter
            .then(|| front_matter::extract(source))
            .flatten();
        let body_start = front_matter
            .as_ref()
            .map_or(0, |metadata| metadata.span.end as usize);
        let (source, source_map) = source_normalization::normalize(allocator, source, body_start);
        let mut parser = Self {
            allocator,
            source,
            source_map,
            front_matter: front_matter.map(|metadata| allocator.boxed(metadata)),
            inline_note_seen: (options.inline_footnotes && !source.is_empty())
                .then(|| &*allocator.alloc(std::cell::Cell::new(false))),
            options,
            position: 0,
            nesting_depth: 0,
            definitions: None,
            footnote_labels: None,
            lazy_lines: None,
            comment_lines: None,
            definition_marker: std::cell::Cell::new(None),
            link_probe_cache: std::cell::RefCell::default(),
            last_closer: std::cell::RefCell::default(),
            definition_region: None,
            comment_definition_region: None,
        };
        // A single fused pre-pass collects both the reference definitions
        // and the footnote labels (see `prepass.rs`).
        let (definitions, footnote_labels) = parser.build_prepass();
        parser.definitions = definitions;
        parser.footnote_labels = footnote_labels;
        parser
    }

    /// Creates a parser for re-parsing a stripped sub-source (block quote,
    /// list item, footnote body, or JSX child content) that shares this
    /// parser's reference definitions instead of re-collecting them.
    /// Sub-parser that also knows which of its lines were added by lazy
    /// continuation (offsets into `source`).
    ///
    /// Every sub-source is one block level deeper than its parent, so the
    /// depth is raised here rather than at each call site: this is the only
    /// way a block construct re-enters the parser, and counting it in one
    /// place is what makes [`ParserOptions::max_nesting_depth`] apply to
    /// all of them.
    pub(crate) fn sub_parser_with_lazy_lines(
        &self,
        source: &'a str,
        lazy_lines: rustc_hash::FxHashSet<u32>,
    ) -> Parser<'a> {
        Self {
            allocator: self.allocator,
            source,
            source_map: None,
            front_matter: None,
            // Line comments are recognized on physical source lines, before
            // container prefixes are stripped, never on generated sub-sources.
            // Front matter belongs only to the original document start.
            options: ParserOptions {
                line_comments: false,
                front_matter: false,
                ..self.options.clone()
            },
            position: 0,
            nesting_depth: self.nesting_depth + 1,
            definitions: self.definitions.clone(),
            footnote_labels: self.footnote_labels.clone(),
            inline_note_seen: self.inline_note_seen,
            // Most sub-sources are entered without any lazy continuation
            // line, and every block quote and list item builds one of these.
            lazy_lines: (!lazy_lines.is_empty()).then(|| std::rc::Rc::new(lazy_lines)),
            comment_lines: None,
            definition_marker: std::cell::Cell::new(None),
            link_probe_cache: std::cell::RefCell::default(),
            last_closer: std::cell::RefCell::default(),
            definition_region: None,
            comment_definition_region: None,
        }
    }

    /// Parses the source into a document AST.
    pub fn parse(mut self) -> ParseResult<Document<'a>> {
        use spans::SpanMap;

        let mut result = self.parse_document();
        if let Some(map) = self.source_map {
            match &mut result {
                Ok(document) => {
                    document.span = map.map_document_span(document.span);
                    for node in &mut document.children {
                        Self::remap_node_spans(node, map);
                    }
                }
                Err(error) => {
                    let span = error.span_mut();
                    *span = map.map_span(*span);
                }
            }
        }
        if self.nesting_depth == 0
            && self.inline_note_seen.is_some_and(std::cell::Cell::get)
            && let Ok(document) = &mut result
        {
            self.lower_inline_footnotes(document);
        }
        result
    }

    fn parse_document(&mut self) -> ParseResult<Document<'a>> {
        let mut children = self
            .allocator
            .new_vec_with_capacity(Self::document_children_capacity(self.source.len()));

        while !self.is_at_end() {
            if let Some(node) = self.parse_block()? {
                children.push(node);
            }
        }

        let span = Span::new(0, self.source.len() as u32);
        Ok(Document {
            front_matter: self.front_matter.take(),
            children,
            span,
        })
    }

    /// Slots to reserve for the document's top-level block list.
    ///
    /// The published sample (and concatenations of it) lands a block every
    /// ~40 bytes. Inline children already reserve from a similar density
    /// heuristic; the root list used to grow from zero, which recopied the
    /// whole array through the doubling ladder on every large parse. One
    /// vector per document, so over-reserve is cheap compared to that copy.
    fn document_children_capacity(source_len: usize) -> usize {
        const BYTES_PER_BLOCK: usize = 40;
        (source_len / BYTES_PER_BLOCK).max(4)
    }
}
