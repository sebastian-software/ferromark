//! Stateful HTML renderer and public render entry point.
//!
//! The renderer struct lives here, while specialized child modules implement output
//! helpers, block and inline visitors, URL rewriting, callouts, and code block details.
//! This keeps the public constructor/render path visible without forcing unrelated
//! rendering rules into one large file.

mod blocks;
mod callout;
mod code_block;
mod definition_list;
mod footnotes;
mod hooks;
mod incremental;
mod inlines;
mod links;
mod mdx;
mod table_columns;
mod visit;
mod write;

use crate::ast::{Document, Node};
use compact_str::CompactString;
use rustc_hash::FxHashMap;

use super::autolink::FirstByteIndex;
use super::heading::{HeadingIdPlanner, PlannedId};
use super::options::{HtmlRendererOptions, RendererOptions};

pub use hooks::{
    CodeHighlightInput, HighlightedCodeBlock, HtmlRenderContext, HtmlRenderControl,
    HtmlRenderHooks, NoHtmlRenderHooks,
};

/// Stateful HTML renderer for Markdown AST documents.
///
/// A renderer instance owns reusable buffers for heading IDs and autolink scanning.
/// Reusing the same instance across renders avoids a
/// set of hot-path allocations while keeping the public API as simple as
/// [`HtmlRenderer::render`].
pub struct HtmlRenderer {
    options: RendererOptions,
    output: String,
    /// Shared by heading and footnote IDs, and with Node metadata for matching
    /// heading IDs. It is retained across incremental fragments.
    heading_id_planner: HeadingIdPlanner,
    /// How many times each footnote identifier has been referenced so
    /// far in this render, so repeated references can be given unique
    /// `fnref-` ids. Cleared per `render()` like the heading id map.
    footnote_ref_counts: FxHashMap<String, usize>,
    /// Legacy source identifier → claimed target ID. Kept across incremental
    /// fragments so a later reference points to the same target.
    legacy_footnote_targets: FxHashMap<CompactString, PlannedId>,
    /// Reserved first reference ID for each legacy footnote. Definitions use
    /// it for their backlink even when the reference appears later.
    legacy_footnote_first_references: FxHashMap<CompactString, PlannedId>,
    /// Source identifier → list index for semantic footnotes. One insert
    /// per unique footnote; later markers look up this map.
    footnote_index: FxHashMap<CompactString, u32>,
    /// Document-order footnote list used when `semantic_footnotes` is on.
    footnote_records: Vec<footnotes::FootnoteRecord>,
    /// Slugs already handed to a footnote, mapped to the next `-N` suffix
    /// to try for that slug as a base. Presence means "taken", so one
    /// lookup answers both questions the uniquifier asks. This replaces a
    /// scan of `footnote_records` per footnote, which made a document of
    /// many footnotes quadratic. Cleared per render like the heading map.
    footnote_slug_counts: FxHashMap<CompactString, usize>,
    /// Reusable scratch buffer for the raw concatenated heading text in
    /// `prepare_heading_id`. A long-lived buffer avoids paying for a fresh
    /// `String` allocation per heading — `slugify_heading` previously
    /// allocated one `text` String per call. Empty until the first heading
    /// (see [`reserve_heading_scratch`]).
    heading_text_scratch: String,
    /// Reusable scratch buffer for semantic footnote slugs. Heading slugs are
    /// written straight into the heading ID planner instead.
    heading_slug_scratch: String,
    /// Unique heading ID for the heading currently being written, including
    /// any `-N` suffix and configured prefix, as held by `heading_id_planner`.
    /// Permalinks reuse this exact value instead of slugifying again.
    heading_id: PlannedId,
    /// Whether `heading_id` came from an explicit `{#id}` heading attribute
    /// rather than from the slugifier.
    ///
    /// A generated slug is built only from lowercase alphanumerics, `-`
    /// separators, the `section` fallback, and an optional `-N` suffix (see
    /// `slugify_heading_into`), so it can never contain a byte that attribute
    /// escaping would replace. Recording the provenance lets the `id` and the
    /// permalink `href` push such an id into the output verbatim, while
    /// author-supplied ids keep the escaping pass. Written by
    /// `prepare_heading_id` before either consumer reads it.
    heading_id_is_explicit: bool,
    /// 1-based code block index inside the current render. Code-line link
    /// metadata uses this only when a block has no filename/title to derive a
    /// human-readable fragment prefix from.
    code_block_index: usize,
    /// Suppresses URL auto-linking while we're already inside an `<a>` so
    /// the builtin can't nest anchors. Tracked manually rather than via
    /// the AST because `visit_text` can be reached through many parents
    /// (paragraphs, headings, emphasis, …) and only the link case needs
    /// to mask it out.
    in_link: bool,
    /// Named MDX island children run the GFM tagfilter on raw HTML so a
    /// `<script>` in component children cannot execute. Cleared while the
    /// island writes its own non-executing JSON payload.
    in_mdx_island_children: bool,
    /// First-byte skip index for the autolink scanner. It depends only on
    /// `options.autolink_patterns` and `options.autolink_urls`, neither of
    /// which can change after construction (`options` is private and never
    /// reassigned), so it is built once per renderer and reused for every text
    /// node of every render instead of being rebuilt per node — or, as before,
    /// per render, which charged two 256-entry tables plus needle and gate
    /// selection to documents far too short to amortize them. `None` when
    /// autolinking is disabled or there are no patterns.
    ///
    /// Callout bodies suppress autolinking by taking this field for the
    /// duration of the body and putting it back afterwards, so the per-render
    /// `is_some()` gate keeps its exact meaning.
    autolink_index: Option<FirstByteIndex>,
}

/// Working capacity a heading scratch buffer is given on first use.
///
/// A typical heading text, slug, and id all sit well under this, so one
/// reservation covers the whole render.
const HEADING_SCRATCH_CAPACITY: usize = 64;

/// Smallest output buffer a document with any content is given.
///
/// Enough for a short paragraph, heading, or list item — the shapes whose
/// markup overhead is not proportional to their source — to be written in one
/// reservation. See [`HtmlRenderer::reserve_output_for`].
pub(super) const MIN_OUTPUT_CAPACITY: usize = 64;

/// Gives a just-cleared scratch buffer its working capacity on first use.
///
/// [`HtmlRenderer`] leaves the heading scratch buffers empty at construction,
/// so building a renderer for a document without headings performs no scratch
/// allocation at all. The first heading pays exactly the one reservation the
/// constructor used to make, and a reused renderer keeps that capacity for
/// later renders because these buffers are only ever cleared, never shrunk.
#[inline]
fn reserve_heading_scratch(buffer: &mut String) {
    if buffer.capacity() == 0 {
        buffer.reserve(HEADING_SCRATCH_CAPACITY);
    }
}

impl HtmlRenderer {
    /// Creates a new HTML renderer with default options.
    ///
    /// Equivalent to `with_options(HtmlRendererOptions::new())`: the documented
    /// defaults borrow static data, so this constructor has one source of truth
    /// and neither path allocates for the configuration itself.
    #[must_use]
    pub fn new() -> Self {
        Self::with_options(HtmlRendererOptions::new())
    }

    /// Creates a new HTML renderer with the specified options.
    ///
    /// The options are moved in, not cloned. Default and static values stay
    /// borrowed all the way through, so a renderer built per document from a
    /// default options value performs no configuration allocation.
    #[must_use]
    pub fn with_options(options: HtmlRendererOptions) -> Self {
        Self::with_renderer_options(options.into())
    }

    /// Returns this renderer with a fixed heading level offset.
    ///
    /// Positive values move headings toward `h6`; negative values move them
    /// toward `h1`. The rendered level clamps to the valid `h1`–`h6` range.
    /// The parsed AST and heading IDs are unchanged.
    #[must_use]
    pub fn with_heading_level_offset(mut self, offset: i32) -> Self {
        self.options.heading_level_offset = offset;
        self
    }

    /// Returns this renderer configured to prefix heading IDs and their
    /// generated permalink fragments.
    ///
    /// The prefix is included in duplicate-ID planning, so generated IDs stay
    /// unique when a prefixed heading collides with another emitted ID. It
    /// affects heading IDs only; authored fragment links and footnote
    /// identifiers are unchanged.
    /// Prefixes may contain ASCII letters, digits, `_`, and `-`. Use an empty
    /// prefix to keep the default behavior.
    pub fn try_with_heading_id_prefix(
        mut self,
        prefix: impl Into<String>,
    ) -> Result<Self, super::heading::InvalidHeadingIdPrefix> {
        let prefix = prefix.into();
        Self::validate_heading_id_prefix(&prefix)?;
        self.options.set_heading_id_prefix(prefix);
        Ok(self)
    }

    /// Validates a heading ID prefix without constructing a renderer.
    pub fn validate_heading_id_prefix(
        prefix: &str,
    ) -> Result<(), super::heading::InvalidHeadingIdPrefix> {
        if prefix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            Ok(())
        } else {
            Err(super::heading::InvalidHeadingIdPrefix)
        }
    }

    fn with_renderer_options(options: RendererOptions) -> Self {
        // The index is a pure function of the options, which are immutable for
        // the life of the renderer, so it is built here rather than at every
        // `render` entry.
        let autolink_patterns = options.autolink_patterns();
        let autolink_index = if options.autolink_urls && !autolink_patterns.is_empty() {
            Some(FirstByteIndex::from_patterns(autolink_patterns))
        } else {
            None
        };
        Self {
            options,
            output: String::new(),
            heading_id_planner: HeadingIdPlanner::new(),
            footnote_ref_counts: FxHashMap::default(),
            legacy_footnote_targets: FxHashMap::default(),
            legacy_footnote_first_references: FxHashMap::default(),
            footnote_index: FxHashMap::default(),
            footnote_records: Vec::new(),
            footnote_slug_counts: FxHashMap::default(),
            // The heading scratch buffers and the planner's ID storage start
            // empty. Constructing them pre-sized cost allocations per renderer
            // even for a document that has no heading at all — the common
            // shape for short inputs and for pipelines that build one renderer
            // per document. The first heading (or footnote slug) reserves the
            // working capacity instead, and reuse keeps it warm.
            heading_text_scratch: String::new(),
            heading_slug_scratch: String::new(),
            heading_id: PlannedId::default(),
            heading_id_is_explicit: false,
            code_block_index: 0,
            in_link: false,
            in_mdx_island_children: false,
            autolink_index,
        }
    }

    /// Renders a document to HTML string.
    #[must_use]
    pub fn render(&mut self, document: &Document<'_>) -> String {
        self.render_into_output(document);
        std::mem::take(&mut self.output)
    }

    /// Renders a document and returns a borrow of the renderer's own output
    /// buffer instead of handing the buffer away.
    ///
    /// [`Self::render`] moves the buffer out, so a reused renderer starts the
    /// next document from zero capacity and pays for the growth again. Callers
    /// that copy the HTML somewhere else anyway (the NAPI boundary copies it
    /// into a JavaScript string) should prefer this: the buffer stays warm and
    /// steady-state rendering stops allocating for output entirely. The borrow
    /// lasts until the next render call.
    #[must_use]
    pub fn render_borrowed(&mut self, document: &Document<'_>) -> &str {
        self.render_into_output(document);
        &self.output
    }

    fn render_into_output(&mut self, document: &Document<'_>) {
        self.prepare_render(document);
        self.render_document(document);
        self.finish_render();
    }

    pub(in crate::renderer::html::renderer) fn prepare_render(&mut self, document: &Document<'_>) {
        self.output.clear();
        self.in_mdx_island_children = false;
        self.code_block_index = 0;
        self.heading_id_planner.clear();
        self.clear_footnote_state();
        // The autolink first-byte index is built once per renderer (see the
        // field) because it depends only on the immutable options.
        self.reserve_output_for(document);
    }

    /// Sizes the output buffer for the document about to be rendered.
    ///
    /// HTML output is typically 2×–3× the markdown source (every `**bold**`
    /// becomes `<strong>...</strong>` etc.) so the original 1.5× estimate kept
    /// undersizing the buffer and forcing power-of-two reallocs on large
    /// documents. 2× hits the realistic mean for the bundled corpora
    /// (rust-book / vite / vue / typescript-handbook all land between 1.8×
    /// and 2.6×).
    ///
    /// A ratio alone is wrong at the short end, though, because HTML overhead
    /// is per block rather than proportional: the shortest paragraph the
    /// renderer emits already carries `<p>` and `</p>\n`, and a heading with
    /// an id carries its own text a second time inside the attribute. Doubling
    /// a comment-sized source therefore under-sizes the buffer and makes the
    /// output grow itself two or three times on the way out, which is the
    /// whole render for such a document. [`MIN_OUTPUT_CAPACITY`] covers those
    /// shapes in one reservation without changing what long documents get.
    pub(in crate::renderer::html::renderer) fn reserve_output_for(
        &mut self,
        document: &Document<'_>,
    ) {
        let source_len = document.span.len() as usize;
        if source_len == 0 {
            // An empty document renders to nothing. Leave the buffer alone so
            // an empty source still yields a string that never allocated.
            return;
        }
        let estimated_len = source_len.saturating_mul(2).max(MIN_OUTPUT_CAPACITY);
        if self.output.capacity() < estimated_len {
            self.output.reserve(estimated_len - self.output.capacity());
        }
    }

    pub(in crate::renderer::html::renderer) fn finish_render(&mut self) {
        self.finish_semantic_footnotes();
    }

    pub(in crate::renderer::html::renderer) fn render_document(&mut self, document: &Document<'_>) {
        for child in &document.children {
            self.render_node(child);
        }
    }

    #[inline]
    pub(in crate::renderer::html::renderer) fn render_node(&mut self, node: &Node<'_>) {
        match node {
            Node::Paragraph(node) => self.render_paragraph(node),
            Node::Heading(node) => self.render_heading(node),
            Node::ThematicBreak(node) => self.render_thematic_break(node),
            Node::BlockQuote(node) => self.render_block_quote(node),
            Node::List(node) => self.render_list(node),
            Node::ListItem(node) => self.render_list_item(node),
            Node::CodeBlock(node) => self.render_code_block(node),
            Node::MathBlock(node) => self.render_math_block(node),
            Node::Html(node) => self.render_html(node),
            Node::Table(node) => self.render_table(node),
            Node::DefinitionList(node) => self.render_definition_list(node),
            Node::DefinitionListTerm(node) => self.render_definition_list_term(node),
            Node::DefinitionListDefinition(node) => self.render_definition_list_definition(node),
            Node::Text(node) => self.render_text(node),
            Node::Emphasis(node) => self.render_emphasis(node),
            Node::Strong(node) => self.render_strong(node),
            Node::InlineCode(node) => self.render_inline_code(node),
            Node::InlineMath(node) => self.render_inline_math(node),
            Node::Break(node) => self.render_break(node),
            Node::Link(node) => self.render_link(node),
            Node::Image(node) => self.render_image(node),
            Node::Highlight(node) => self.render_highlight(node),
            Node::Delete(node) => self.render_delete(node),
            Node::Superscript(node) => self.render_superscript(node),
            Node::Subscript(node) => self.render_subscript(node),
            Node::FootnoteReference(node) => self.render_footnote_reference(node),
            Node::Definition(_) => {}
            Node::FootnoteDefinition(node) => self.render_footnote_definition(node),
            Node::MdxJsxFlowElement(node) => self.render_mdx_jsx_flow_element(node),
            Node::MdxJsxTextElement(node) => self.render_mdx_jsx_text_element(node),
            Node::MdxjsEsm(_) | Node::MdxFlowExpression(_) | Node::MdxTextExpression(_) => {}
        }
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}
