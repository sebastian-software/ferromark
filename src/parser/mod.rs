//! High-performance Markdown parser for Ferromark.
//!
//! This crate provides a fast, arena-allocated Markdown parser following
//! the CommonMark specification with GFM extensions.
//!
//! # Features
//!
//! - Arena-based allocation for zero-copy parsing
//! - CommonMark compliant with GFM extensions
//! - Optional MDX, math, footnotes, and other Markdown syntax extensions
//!
//! # Example
//!
//! ```
//! use ferromark::allocator::Allocator;
//! use ferromark::parser::Parser;
//!
//! let allocator = Allocator::new();
//! let source = "# Hello World\n\nThis is a paragraph.";
//! let parser = Parser::new(&allocator, source);
//! let document = parser.parse();
//! ```

#![deny(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented
    )
)]

mod error;
pub use error::{ParseError, ParseErrorKind, ParseResult};

use crate::allocator::Allocator;
use crate::ast::{Document, Span};

mod block;
mod block_quote;
mod byte_class;
mod cursor;
mod definition_list;
mod delimiters;
mod fenced_code;
mod footnote;
mod front_matter;
mod html;
mod indented_code;
mod inline;
mod inline_footnote;
mod inline_html;
mod inline_link;
mod lazy_paragraph;
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
mod short_scan;
mod source_normalization;
mod spans;
mod table;
mod table_attributes;
mod table_cell_source;

#[cfg(test)]
mod tests;

pub use options::ParserOptions;

/// Internal parse phase, inherited by container sub-parsers. Collection uses
/// the same block grammar without building ordinary inline content or invoking
/// another document prepass. It is not a user-visible syntax option.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParsePhase {
    Document,
    Definitions,
}

/// Memo key for a JSX closer lookup: the identity of an inline slice
/// (pointer, length), the position an opening tag ends at and the tag name
/// a closer has to match. Borrowing the name keeps the lookup
/// allocation-free; names come from the source slice and so live as long as
/// the parser.
type JsxCloserKey<'a> = (usize, usize, usize, Option<&'a str>);

/// Memo key for an inline-math closer probe: the identity of a content
/// slice (pointer, length) and the width of the opener a closer has to
/// match.
type MathCloserKey = (usize, usize, u8);

/// A range of one content slice that the last closing-tag walk for one tag
/// name read without finding a closer: an opening tag in it that
/// `mdx_jsx_closer_presence` does not hold is one nothing closes.
///
/// A run of thousands of unclosed openers is one of these instead of one
/// record each, which keeps the memo from costing more than the walk it
/// replaces.
#[derive(Clone, Copy)]
struct JsxCloserGap<'a> {
    slice: (usize, usize),
    name: Option<&'a str>,
    from: usize,
    until: usize,
}

/// Memo tables of the opt-in scans; see [`Parser::extension_memos`].
#[derive(Default)]
struct ExtensionMemos<'a> {
    /// Memoized presence of a matching closing JSX tag for one opening tag.
    ///
    /// An opening tag can only become a node when a closer matches it, and
    /// the walk that answers that reads the rest of the slice. `<A>` repeated
    /// nests one level per tag, so every opener but the innermost is unclosed
    /// and every one of them used to pay its own walk. One walk decides every
    /// opener it passes (see `scan::record_matching_closes`), which is what
    /// turns the run into a single pass.
    ///
    /// The key names the opener, not just the slice: the answer depends on
    /// where the scan starts, so an answer recorded for one opener must never
    /// be handed to another.
    mdx_jsx_closer_presence:
        std::cell::RefCell<rustc_hash::FxHashMap<JsxCloserKey<'a>, Option<(usize, usize)>>>,

    /// The range of unclosed openers the last such walk left behind.
    mdx_jsx_closer_gap: std::cell::Cell<Option<JsxCloserGap<'a>>>,

    /// Memoized matching `}` for a `{`, keyed by the address of the brace
    /// and of the content end, exactly as `bracket_matches` is.
    ///
    /// `skip_braces` reports that nothing closed only after walking to the
    /// end of the content, so a run of `{` paid one walk each, and a single
    /// `}` behind the run defeats the cheap `has_closer_from` guard in front
    /// of it. One walk decides every brace it passes.
    brace_matches: std::cell::RefCell<rustc_hash::FxHashMap<(usize, usize), Option<usize>>>,

    /// The end of a content slice and a range of it the last brace walk
    /// read byte by byte without finding a closer: a `{` in that range
    /// that `brace_matches` does not hold is a `{` nothing closes.
    ///
    /// A run of thousands of unclosed braces is one range here instead of
    /// one record each, which is what keeps the memo from costing more
    /// than the walk it replaces.
    brace_gap: std::cell::Cell<Option<(usize, usize, usize)>>,

    /// Memoized inline-math closer probe for a content slice and an opener
    /// width. See `Parser::inline_math_close`.
    ///
    /// A `$` opener scans every later `$` to the end of the content before
    /// it can report that nothing closes it, and — unlike the script spans —
    /// keeps going past a candidate that cannot close. A run of them
    /// therefore cost one walk each.
    math_closers: std::cell::RefCell<rustc_hash::FxHashMap<MathCloserKey, math::MathClose>>,

    /// Ranges of a content slice that a scan for an inline-math closer read
    /// without finding one, keyed like `math_closers`.
    ///
    /// The candidate memo above settles a run whose suffix holds no closing
    /// `$` at all. This settles the run whose only candidate sits inside a
    /// code span: the walk that stepped over it read everything else, so
    /// every opener in what it read is answered from the record.
    math_closer_gaps: std::cell::RefCell<rustc_hash::FxHashMap<MathCloserKey, math::MathGaps>>,

    /// Memo for the next `*` or `_` in a content slice, which is what a `$`
    /// before a digit has to find between itself and its closer before it
    /// opens anything. Without it a run of such openers searched the same
    /// span once each.
    math_emphasis: std::cell::Cell<Option<math::MathEmphasis>>,

    /// The first `$$` terminator at or after a scan start in this parser's
    /// source, with the source length standing for "nothing closes".
    ///
    /// Every line opening with `$$` asks for it, through the block dispatch
    /// and again through the block-start probe, and the scan walks to the
    /// end of the source to report that there is none.
    math_block_close: std::cell::Cell<Option<(usize, usize)>>,

    /// Memoized position of the last `]]` in a content slice, keyed like
    /// `link_probe_cache`: the wiki-link scan's counterpart to
    /// `has_closer_from`.
    /// A `[[` with no `]]` after it walks to the end of the content to find
    /// that out, so a run of them cost one walk each; one search answers for
    /// every opener in the slice.
    wiki_closer: std::cell::RefCell<rustc_hash::FxHashMap<(usize, usize), Option<usize>>>,
}

/// A memo table allocated in the arena on first use, so a parser that never
/// consults it carries one pointer, not an empty table. The inline memos
/// answer the quadratic shapes the release review bounded, which ordinary
/// documents rarely present; held inline they had grown the parser from 272
/// to 736 bytes, and every block quote and list item builds a sub-parser.
type LazyMap<'a, K, V> = std::cell::OnceCell<crate::allocator::Box<'a, MemoTable<K, V>>>;

/// A memo table behind a [`LazyMap`].
type MemoTable<K, V> = std::cell::RefCell<rustc_hash::FxHashMap<K, V>>;

/// Markdown parser.
pub struct Parser<'a> {
    /// Arena allocator.
    allocator: &'a Allocator,

    /// Source text.
    source: &'a str,

    /// Root-only map from normalized Markdown offsets to the original source.
    source_map: Option<&'a source_normalization::NormalizedSourceMap<'a>>,

    /// Raw document metadata, already in original source coordinates.
    front_matter: Option<crate::allocator::Box<'a, crate::ast::FrontMatter<'a>>>,

    /// Parser options.
    options: ParserOptions,

    /// Current position in the source.
    position: usize,

    /// Current nesting depth.
    nesting_depth: usize,

    /// Inline contexts currently open above this one.
    ///
    /// Link text, image alt text, wiki-link labels, script spans and inline
    /// JSX phrasing all re-enter [`Self::parse_inline`] on the same parser,
    /// so this counts what `nesting_depth` cannot: inline parsing never
    /// builds a sub-parser. Inline parsing runs behind `&self`, hence the
    /// cell. See `inline::Parser::enter_inline` for the bound itself.
    inline_depth: std::cell::Cell<usize>,

    /// Depth of the deepest inline subtree finished inside the inline
    /// context currently being scanned.
    ///
    /// `inline_depth` counts the contexts *above* a pairing; this counts
    /// what is already *below* it, so that delimiter-run pairing can tell
    /// how much of the budget an emphasis node would spend. Every
    /// `parse_inline` resets it on entry and folds its own result back into
    /// the enclosing level on exit (`inline::InlineDepthGuard`), so a level
    /// only ever reads the subtrees nested directly inside it.
    ///
    /// Every parser owns its counter outright. Only a sub-parser built from
    /// inside an inline context — an inline note (`^[...]`), and nothing
    /// else — holds content that lands inside the node the parent is about
    /// to wrap, and that one site hands its result back explicitly
    /// (`Parser::inline_note_sub_parser` and
    /// `inline::Parser::fold_nested_inline_depth`). Sharing one arena cell
    /// by pointer instead charged every parse an arena allocation and an
    /// indirection on each access for a case almost no document has.
    nested_inline_depth: std::cell::Cell<usize>,

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

    /// The document parse or its temporary block-only definition pass.
    phase: ParsePhase,

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

    /// Memo for the next byte in a content slice that could start an inline
    /// construct other than a bracket (see `next_bracket_text_stop`).
    ///
    /// Bracket text is parsed where it stands only while it holds nothing
    /// but text and brackets, and a nested run asks that question once per
    /// level over the same slice. One forward window answers all of them.
    bracket_text_stop: std::cell::Cell<delimiters::ForwardMemo>,

    /// A `[start, end)` window of this parser's source that holds no
    /// definition-list item start. See `Parser::can_start_definition_item_at`.
    definition_item_gap: std::cell::Cell<Option<(usize, usize)>>,

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
    /// Most nested brackets no longer reach the probe — they are parsed
    /// where they stand (`Parser::parse_bracket_text`) — but the ones it
    /// still answers for are re-scanned by the same fallback.
    ///
    /// Every slice lives in the source or the arena, both of which outlive
    /// the parser, so an address plus a length names one byte range for as
    /// long as the cache exists.
    link_probe_cache: LazyMap<'a, (usize, usize), bool>,

    /// Memoized bracket matches: for the address of a scan start and of the
    /// content end, where that scan's `]` is and whether an opener sits
    /// inside it.
    ///
    /// `scan_balanced` walks from one opener to its `]`, so a run of nested
    /// brackets walked the same bytes once per level. One walk already
    /// decides every opener it passes (see `scan_balanced_matched`), and
    /// keeping those answers is what turns the run into a single pass.
    /// Openers with no `]` after them are recorded too: they are the
    /// unbalanced shape that used to walk to the end of the content once per
    /// opener.
    ///
    /// Same lifetime argument as `link_probe_cache`: every slice lives in the
    /// source or the arena, so an address pair names one byte range for as
    /// long as the cache exists.
    bracket_matches: LazyMap<'a, (usize, usize), (usize, bool)>,

    /// One forward window per closing byte — `]`, `>` and `}` — over the
    /// bytes this parser reads: a range that holds none of it, and the one
    /// just behind that range. See `Parser::has_closer_from`.
    ///
    /// An opener can only open something when its closer follows it, and
    /// the balanced scans answer that by walking to the end of the content.
    /// A run of openers with no closer therefore paid one full walk each:
    /// 64 KiB of `[ ` took 1.0 s and 32 KiB of `{ ` took 0.26 s, both
    /// growing x16 for every x4 of input. The window moves forward with the
    /// parse and answers every opener in the run from bytes the first one
    /// already read, so the run costs one scan in total — and it answers
    /// from three words of parser state instead of a hash of the slice.
    closer_windows: [std::cell::Cell<delimiters::CloserWindow>; delimiters::CLOSER_SLOTS],

    /// The memo tables of the opt-in scans — MDX, math and wiki links —
    /// allocated in the arena the first time one of them is consulted, so a
    /// parse that never enables them carries one pointer for all nine.
    extension_memos: std::cell::OnceCell<crate::allocator::Box<'a, ExtensionMemos<'a>>>,

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
    /// A memo table, allocated in the arena the first time it is consulted.
    fn lazy_map<'s, K, V>(&'s self, cell: &'s LazyMap<'a, K, V>) -> &'s MemoTable<K, V> {
        cell.get_or_init(|| self.allocator.boxed(std::cell::RefCell::default()))
    }

    fn link_probe_cache(&self) -> &MemoTable<(usize, usize), bool> {
        self.lazy_map(&self.link_probe_cache)
    }

    fn bracket_matches(&self) -> &MemoTable<(usize, usize), (usize, bool)> {
        self.lazy_map(&self.bracket_matches)
    }

    /// The opt-in scans' memo tables, allocated in the arena on first use.
    fn extension_memos(&self) -> &ExtensionMemos<'a> {
        self.extension_memos
            .get_or_init(|| self.allocator.boxed(ExtensionMemos::default()))
    }

    /// Creates a new parser with default options.
    #[must_use]
    pub fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self::with_options(allocator, source, ParserOptions::default())
    }

    /// Creates a new parser with the specified options.
    #[must_use]
    pub fn with_options(allocator: &'a Allocator, source: &'a str, options: ParserOptions) -> Self {
        Self::with_phase(allocator, source, options, ParsePhase::Document)
    }

    fn with_phase(
        allocator: &'a Allocator,
        source: &'a str,
        options: ParserOptions,
        phase: ParsePhase,
    ) -> Self {
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
            inline_note_seen: (phase == ParsePhase::Document
                && options.inline_footnotes
                && !source.is_empty())
            .then(|| &*allocator.alloc(std::cell::Cell::new(false))),
            options,
            position: 0,
            nesting_depth: 0,
            inline_depth: std::cell::Cell::new(0),
            nested_inline_depth: std::cell::Cell::new(0),
            definitions: None,
            phase,
            footnote_labels: None,
            lazy_lines: None,
            comment_lines: None,
            definition_marker: std::cell::Cell::new(None),
            bracket_text_stop: std::cell::Cell::default(),
            definition_item_gap: std::cell::Cell::new(None),
            link_probe_cache: std::cell::OnceCell::new(),
            bracket_matches: std::cell::OnceCell::new(),
            closer_windows: std::array::from_fn(|_| std::cell::Cell::default()),
            extension_memos: std::cell::OnceCell::new(),
            definition_region: None,
            comment_definition_region: None,
        };
        // Discover document-wide definitions once, before inline resolution
        // (see `prepass.rs`). Collection itself must not recurse.
        if phase == ParsePhase::Document {
            let (definitions, footnote_labels) = parser.build_prepass();
            parser.definitions = definitions;
            parser.footnote_labels = footnote_labels;
        }
        parser
    }

    /// Creates a parser for re-parsing a stripped sub-source (block quote,
    /// list item, footnote body, or JSX child content) that shares this
    /// parser's reference definitions instead of re-collecting them.
    /// Sub-parser that also knows which of its lines were added by lazy
    /// continuation (offsets into `source`).
    pub(in crate::parser) fn sub_parser_with_lazy_lines(
        &self,
        source: &'a str,
        lazy_lines: rustc_hash::FxHashSet<u32>,
    ) -> Parser<'a> {
        // Block constructs are entered with no inline context open, so
        // nothing this sub-parser nests can end up inside a node the caller
        // is still building, and its `nested_inline_depth` is its own. An
        // inline note is the one construct that re-enters the parser from
        // inline content; it goes through `inline_note_sub_parser`, which
        // says how the count gets back. Anything else that starts doing so
        // has to do the same, and this is what says so.
        debug_assert_eq!(
            self.inline_depth.get(),
            0,
            "a sub-parser built from inside an inline context must fold its \
             nested inline depth back (see Parser::inline_note_sub_parser)"
        );
        self.nested_sub_parser(source, lazy_lines)
    }

    /// Sub-parser for an inline note's body, which is the one sub-source
    /// entered from inside an inline context.
    ///
    /// The note's content becomes children of a node the enclosing inline
    /// level is still building, so the depth this parser finishes with
    /// counts toward the same budget. The caller reads it back with
    /// `inline::Parser::fold_nested_inline_depth` on every exit, including
    /// the error one, which is what the previously shared counter cell did
    /// when the guard in the sub-parse dropped.
    pub(in crate::parser) fn inline_note_sub_parser(&self, source: &'a str) -> Parser<'a> {
        self.nested_sub_parser(source, rustc_hash::FxHashSet::default())
    }

    /// Builds either kind of sub-parser.
    ///
    /// Every sub-source is one block level deeper than its parent, so the
    /// depth is raised here rather than at each call site: this is the only
    /// way a construct re-enters the parser on a sub-source, and counting it
    /// in one place is what makes [`ParserOptions::max_nesting_depth`] apply
    /// to all of them.
    fn nested_sub_parser(
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
            // Container sub-sources are entered from block parsing, where no
            // inline context is open, so this normally copies a zero. An
            // inline note (`^[...]`) is the exception: it builds a sub-parser
            // from inside `parse_inline`, and carrying the count is what
            // keeps a chain of them bounded.
            inline_depth: std::cell::Cell::new(self.inline_depth.get()),
            // Copied, so that an inline note's body starts from what the
            // enclosing level has already finished, exactly as the shared
            // cell handed it over. Its own result travels back the other way
            // (see `inline_note_sub_parser`); a block sub-source has no
            // enclosing inline level to report to.
            nested_inline_depth: std::cell::Cell::new(self.nested_inline_depth.get()),
            definitions: self.definitions.clone(),
            phase: self.phase,
            footnote_labels: self.footnote_labels.clone(),
            inline_note_seen: self.inline_note_seen,
            // Most sub-sources are entered without any lazy continuation
            // line, and every block quote and list item builds one of these.
            lazy_lines: (!lazy_lines.is_empty()).then(|| std::rc::Rc::new(lazy_lines)),
            comment_lines: None,
            definition_marker: std::cell::Cell::new(None),
            bracket_text_stop: std::cell::Cell::default(),
            definition_item_gap: std::cell::Cell::new(None),
            link_probe_cache: std::cell::OnceCell::new(),
            bracket_matches: std::cell::OnceCell::new(),
            closer_windows: std::array::from_fn(|_| std::cell::Cell::default()),
            extension_memos: std::cell::OnceCell::new(),
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

/// Parses Markdown source into an AST.
///
/// This is a convenience function that creates a parser with default options.
pub fn parse<'a>(
    allocator: &'a crate::allocator::Allocator,
    source: &'a str,
) -> ParseResult<crate::ast::Document<'a>> {
    Parser::new(allocator, source).parse()
}
