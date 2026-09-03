#![warn(missing_docs)]

//! Markdown to HTML with a secure rendering default.
//!
//! ferromark streams Markdown into HTML without building an AST. Its default
//! [`RenderPolicy::Untrusted`] escapes raw HTML and restricts unsafe URL
//! schemes. Use [`Options`] to select syntax extensions and rendering policy.
//!
//! # Quick start
//!
//! ```
//! let html = ferromark::to_html("# Hello\n\n**World**");
//!
//! assert_eq!(html, "<h1 id=\"hello\">Hello</h1>\n<p><strong>World</strong></p>\n");
//! ```
//!
//! # Optional MDX support
//!
//! Enable the `mdx` Cargo feature to segment and render MDX documents. The
//! feature is opt-in and does not alter the default Markdown renderer.
//!
//! # Platform optimizations
//!
//! The inline-specials scanner uses NEON on AArch64 builds with NEON enabled
//! and baseline SSE2 on x86-64. When neither path is available, it uses the
//! scalar scanner. See the repository benchmark documentation for current,
//! reproducible measurements.

pub mod block;
pub mod cursor;
pub mod escape;
pub mod footnote;
pub mod inline;
pub mod limits;
pub mod link_ref;
#[cfg(feature = "mdx")]
pub mod mdx;
#[cfg(feature = "profiling")]
#[doc(hidden)]
pub mod profiling;
pub mod range;
pub mod render;

use smallvec::SmallVec;

// Re-export primary types
pub use block::{Alignment, BlockEvent, BlockParser, CalloutType, CodeBlockKind, fixup_list_tight};
pub use footnote::FootnoteStore;
pub use inline::{InlineEvent, InlineParser};
pub use limits::{ResourceLimit, ResourceLimitReport};
pub use link_ref::{LinkRefDef, LinkRefStore};
pub use range::{InputSizeError, MAX_INPUT_BYTES, Range, validate_input_size};
pub use render::HtmlWriter;

/// A complete fenced code block passed to a custom renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct FencedCodeBlock<'a> {
    /// Decoded first word of the CommonMark info string, when present.
    pub language: Option<&'a str>,
    /// Decoded info-string text after the language word, when present —
    /// the "meta" text tooling conventions use for line highlighting,
    /// titles, and similar (e.g. `{1-3} title="…"`).
    pub meta: Option<&'a str>,
    /// Raw code content before HTML escaping.
    pub code: &'a str,
}

/// HTML that a [`FencedCodeRenderer`] has explicitly marked as trusted.
///
/// ferromark writes this value verbatim, including under
/// [`RenderPolicy::Untrusted`]. Constructing it asserts that every untrusted
/// value embedded in the markup has already been escaped. This type does not
/// sanitize HTML and a renderer that violates the contract can introduce XSS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedHtml(String);

impl TrustedHtml {
    /// Mark an owned HTML fragment as safe to write verbatim.
    #[must_use]
    pub fn from_trusted(html: impl Into<String>) -> Self {
        Self(html.into())
    }

    /// Borrow the trusted HTML fragment.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the wrapper and return its HTML fragment.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Opt-in renderer for complete fenced code blocks.
///
/// Returning `None` asks ferromark to emit its normal escaped
/// `<pre><code>...</code></pre>` output. Indented code blocks never invoke this
/// interface.
pub trait FencedCodeRenderer {
    /// Render one fenced code block, or return `None` to use the safe fallback.
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml>;
}

/// Trust boundary applied while rendering links, images, and raw HTML.
///
/// Future releases may add policies. Downstream matches must include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum RenderPolicy {
    /// Escape all raw HTML and allow only browser-safe URL schemes.
    #[default]
    Untrusted,
    /// Preserve raw HTML and arbitrary URL schemes for trusted Markdown and MDX.
    Trusted,
}

/// Parsing/rendering options.
///
/// This struct is non-exhaustive so new options can be added without a breaking
/// release. Outside ferromark, Rust rejects both complete and struct-update
/// literals for non-exhaustive structs. Start with a preset and mutate the
/// public fields instead:
///
/// ```
/// use ferromark::Options;
///
/// let mut options = Options::default();
/// options.heading_ids = false;
/// ```
///
/// [`options!`] is a compact equivalent when several fields need changing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Options {
    /// Select the output trust boundary. Defaults to [`RenderPolicy::Untrusted`].
    pub render_policy: RenderPolicy,
    /// Parse raw inline and block HTML. Untrusted rendering still escapes it.
    pub allow_html: bool,
    /// Resolve link reference definitions and reference-style links.
    pub allow_link_refs: bool,
    /// Enable GFM table extension.
    pub tables: bool,
    /// Enable MultiMarkdown/iA-style column spans in pipe tables.
    ///
    /// Requires [`Self::tables`] to be enabled.
    pub merged_table_cells: bool,
    /// Interpret table delimiter dash counts as relative column-width hints.
    ///
    /// This opt-in extension emits numeric `<col>` widths and does not accept
    /// arbitrary CSS or HTML attributes.
    pub table_column_widths: bool,
    /// Enable GFM strikethrough extension (`~~text~~`).
    pub strikethrough: bool,
    /// Enable highlight/mark extension (`==text==`).
    pub highlight: bool,
    /// Enable superscript extension (`^text^`).
    pub superscript: bool,
    /// Enable subscript extension (`~text~`).
    pub subscript: bool,
    /// Enable GFM task list extension (`[ ]` / `[x]`).
    pub task_lists: bool,
    /// Enable GFM autolink literals extension (bare URLs, www, emails).
    pub autolink_literals: bool,
    /// Enable the GFM disallowed raw HTML extension in trusted mode.
    ///
    /// This is not an HTML sanitizer. [`RenderPolicy::Untrusted`] escapes all
    /// raw HTML regardless of this setting.
    pub disallowed_raw_html: bool,
    /// Enable footnotes extension (`[^label]` references and `[^label]:` definitions).
    pub footnotes: bool,
    /// Enable Pandoc-style inline footnotes (`^[single paragraph note]`).
    pub inline_footnotes: bool,
    /// Enable front matter detection (`---`/`+++` delimited metadata at document start).
    pub front_matter: bool,
    /// Generate GitHub-compatible heading IDs (`<h1 id="slug">`).
    pub heading_ids: bool,
    /// Enable math spans (`$inline$` and `$$display$$`).
    pub math: bool,
    /// Enable GitHub-style callouts/admonitions (`> [!NOTE]`, `> [!WARNING]`, etc.).
    pub callouts: bool,
    /// Enable PHP Markdown Extra-style definition lists.
    pub definition_lists: bool,
    /// Enable source-only line comments beginning with `//`.
    pub line_comments: bool,
    /// Enable CommonMark indented code blocks (four or more leading spaces).
    ///
    /// Disable this for dialects that reserve indentation for other block
    /// semantics and require fenced code blocks instead.
    pub indented_code_blocks: bool,
    /// Prefix internal absolute link destinations with this base path.
    ///
    /// When set, `<a>` destinations that start with `/` (but not `//`, and
    /// not already with the base) are prefixed — for sites deployed under a
    /// subpath, e.g. GitHub Pages. Trailing slashes on the base are ignored
    /// and a bare `"/"` is a no-op. Image sources and autolinks are not
    /// rewritten.
    pub link_base_path: Option<Box<str>>,
}

impl Options {
    /// Return the smallest supported Markdown syntax surface.
    ///
    /// Ordinary paragraphs, headings, emphasis, code, links, images, lists,
    /// blockquotes, and breaks remain available. Optional extensions,
    /// reference links, and raw HTML parsing are disabled.
    #[must_use]
    pub const fn minimal() -> Self {
        Self {
            render_policy: RenderPolicy::Untrusted,
            allow_html: false,
            allow_link_refs: false,
            tables: false,
            merged_table_cells: false,
            table_column_widths: false,
            strikethrough: false,
            highlight: false,
            superscript: false,
            subscript: false,
            task_lists: false,
            autolink_literals: false,
            disallowed_raw_html: false,
            footnotes: false,
            inline_footnotes: false,
            front_matter: false,
            heading_ids: false,
            math: false,
            callouts: false,
            definition_lists: false,
            line_comments: false,
            indented_code_blocks: true,
            link_base_path: None,
        }
    }

    /// Return the CommonMark syntax configuration.
    ///
    /// This enables raw HTML parsing and reference links but keeps
    /// [`RenderPolicy::Untrusted`]. Select [`RenderPolicy::Trusted`] separately
    /// when raw HTML passthrough is appropriate for trusted input.
    #[must_use]
    pub const fn commonmark() -> Self {
        Self {
            render_policy: RenderPolicy::Untrusted,
            allow_html: true,
            allow_link_refs: true,
            tables: false,
            merged_table_cells: false,
            table_column_widths: false,
            strikethrough: false,
            highlight: false,
            superscript: false,
            subscript: false,
            task_lists: false,
            autolink_literals: false,
            disallowed_raw_html: false,
            footnotes: false,
            inline_footnotes: false,
            front_matter: false,
            heading_ids: false,
            math: false,
            callouts: false,
            definition_lists: false,
            line_comments: false,
            indented_code_blocks: true,
            link_base_path: None,
        }
    }

    /// Return the GitHub Flavored Markdown syntax configuration.
    ///
    /// This extends [`Options::commonmark`] with tables, strikethrough, task
    /// lists, autolink literals, and the disallowed raw HTML extension. Output
    /// remains under [`RenderPolicy::Untrusted`] unless explicitly overridden.
    #[must_use]
    pub const fn gfm() -> Self {
        Self {
            render_policy: RenderPolicy::Untrusted,
            allow_html: true,
            allow_link_refs: true,
            tables: true,
            merged_table_cells: false,
            table_column_widths: false,
            strikethrough: true,
            highlight: false,
            superscript: false,
            subscript: false,
            task_lists: true,
            autolink_literals: true,
            disallowed_raw_html: true,
            footnotes: false,
            inline_footnotes: false,
            front_matter: false,
            heading_ids: false,
            math: false,
            callouts: false,
            definition_lists: false,
            line_comments: false,
            indented_code_blocks: true,
            link_base_path: None,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            render_policy: RenderPolicy::Untrusted,
            allow_html: true,
            allow_link_refs: true,
            tables: true,
            merged_table_cells: false,
            table_column_widths: false,
            strikethrough: true,
            highlight: false,
            superscript: false,
            subscript: false,
            task_lists: true,
            autolink_literals: false,
            disallowed_raw_html: true,
            footnotes: false,
            inline_footnotes: false,
            front_matter: false,
            heading_ids: true,
            math: false,
            callouts: true,
            definition_lists: false,
            line_comments: false,
            indented_code_blocks: true,
            link_base_path: None,
        }
    }
}

/// Create [`Options`] by applying public-field updates to a preset.
///
/// This is useful outside ferromark because [`Options`] is non-exhaustive and
/// therefore cannot be constructed with a struct literal. It is equivalent to
/// creating the preset, mutating each named public field, and returning it.
///
/// ```
/// use ferromark::Options;
///
/// let options = ferromark::options!(Options::gfm();
///     front_matter: true,
///     allow_html: false,
/// );
/// assert!(options.front_matter);
/// assert!(!options.allow_html);
/// ```
#[macro_export]
macro_rules! options {
    ($base:expr; $($field:ident $( : $value:expr )?),* $(,)?) => {{
        let mut __ferromark_options = $base;
        $(
            $crate::options!(@set __ferromark_options, $field $(, $value)?);
        )*
        __ferromark_options
    }};
    (@set $options:ident, $field:ident, $value:expr) => {
        $options.$field = $value
    };
    (@set $options:ident, $field:ident) => {
        $options.$field = $field
    };
}

/// Result of parsing Markdown with front matter extraction.
pub struct ParseResult<'a> {
    /// Rendered HTML output.
    pub html: String,
    /// Raw front matter content (between delimiters), if detected.
    pub front_matter: Option<&'a str>,
    /// Document headings in source order, for table-of-contents rendering.
    pub headings: Vec<Heading>,
    /// Resource-limit fallbacks used while parsing the document.
    pub resource_limits: ResourceLimitReport,
}

/// One document heading collected during rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// Heading level, 1–6.
    pub level: u8,
    /// The generated slug, present when [`Options::heading_ids`] is enabled.
    pub id: Option<String>,
    /// Plain heading text with inline markup and HTML tags removed.
    pub text: String,
}

/// Extract front matter from the start of a document.
///
/// Returns `Some((content, rest_offset))` where `content` is the raw text between
/// delimiters and `rest_offset` is the byte offset where the remaining markdown begins.
/// Returns `None` if no valid front matter is found.
fn extract_front_matter(input: &str) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    if bytes.len() < 3 {
        return None;
    }

    // Determine delimiter character: must be exactly 3 of `-` or `+` at byte 0
    let delim_char = match bytes[0] {
        b'-' | b'+' => bytes[0],
        _ => return None,
    };

    // Verify exactly 3 delimiter chars (not 4+)
    if bytes.len() < 3 || bytes[1] != delim_char || bytes[2] != delim_char {
        return None;
    }

    // After the 3 delimiter chars, only whitespace allowed before newline
    let mut pos = 3;
    if pos < bytes.len() && bytes[pos] == delim_char {
        // 4+ delimiter chars — not front matter
        return None;
    }
    while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }

    // Must hit newline (or end of input for degenerate case, but that means no closing)
    if pos >= bytes.len() {
        return None;
    }
    if bytes[pos] == b'\r' {
        pos += 1;
    }
    if pos >= bytes.len() || bytes[pos] != b'\n' {
        return None;
    }
    pos += 1;

    let content_start = pos;

    // Search for closing delimiter
    loop {
        if pos >= bytes.len() {
            // No closing delimiter found
            return None;
        }

        // Check if current line is a closing delimiter
        let line_start = pos;
        if pos + 2 < bytes.len()
            && bytes[pos] == delim_char
            && bytes[pos + 1] == delim_char
            && bytes[pos + 2] == delim_char
        {
            let mut p = pos + 3;
            // Must not have 4+ delimiter chars
            if p < bytes.len() && bytes[p] == delim_char {
                // Not a closing delimiter, skip this line
            } else {
                // Optional trailing whitespace
                while p < bytes.len() && (bytes[p] == b' ' || bytes[p] == b'\t') {
                    p += 1;
                }
                // Must be at newline or EOF
                let at_end = p >= bytes.len()
                    || bytes[p] == b'\n'
                    || (bytes[p] == b'\r' && p + 1 < bytes.len() && bytes[p + 1] == b'\n');

                if at_end {
                    let content = &input[content_start..line_start];
                    // Advance past the closing delimiter line
                    let mut rest = p;
                    if rest < bytes.len() {
                        if bytes[rest] == b'\r' {
                            rest += 1;
                        }
                        if rest < bytes.len() && bytes[rest] == b'\n' {
                            rest += 1;
                        }
                    }
                    return Some((content, rest));
                }
            }
        }

        // Skip to next line
        while pos < bytes.len() && bytes[pos] != b'\n' {
            pos += 1;
        }
        if pos < bytes.len() {
            pos += 1; // skip \n
        }

        // Safety: if we haven't advanced past line_start, force progress
        if pos <= line_start {
            break;
        }
    }

    None
}

/// Parse Markdown and return HTML, front matter, headings, and resource-limit
/// fallbacks.
///
/// Uses default options with `front_matter: true`.
///
/// # Example
/// ```
/// let result = ferromark::parse("---\ntitle: Hello\n---\n# Content");
/// assert_eq!(result.front_matter, Some("title: Hello\n"));
/// assert!(result.html.contains("Content</h1>"));
/// assert_eq!(result.headings[0].id.as_deref(), Some("content"));
/// assert_eq!(result.headings[0].text, "Content");
/// assert!(result.resource_limits.is_empty());
/// ```
///
/// # Panics
///
/// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use [`try_parse`] to
/// handle the limit as an error.
pub fn parse(input: &str) -> ParseResult<'_> {
    try_parse(input).unwrap_or_else(|error| panic!("{error}"))
}

/// Parse Markdown with default options without panicking for oversized input.
pub fn try_parse(input: &str) -> Result<ParseResult<'_>, InputSizeError> {
    let options = Options {
        front_matter: true,
        ..Options::default()
    };
    try_parse_with_options(input, &options)
}

/// Parse Markdown with options and return HTML, front matter, headings, and
/// resource-limit fallbacks.
///
/// Front matter is only extracted when `options.front_matter` is `true`.
/// Heading IDs are only present when `options.heading_ids` is enabled.
///
/// # Panics
///
/// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use
/// [`try_parse_with_options`] to handle the limit as an error.
pub fn parse_with_options<'a>(input: &'a str, options: &Options) -> ParseResult<'a> {
    try_parse_with_options(input, options).unwrap_or_else(|error| panic!("{error}"))
}

/// Parse Markdown with options without panicking for oversized input.
pub fn try_parse_with_options<'a>(
    input: &'a str,
    options: &Options,
) -> Result<ParseResult<'a>, InputSizeError> {
    validate_input_size(input.len())?;
    Ok(parse_impl(input, options, None, None))
}

#[cfg(feature = "mdx")]
pub(crate) fn try_parse_with_options_and_link_refs<'a>(
    input: &'a str,
    options: &Options,
    link_refs: &LinkRefStore,
) -> Result<ParseResult<'a>, InputSizeError> {
    validate_input_size(input.len())?;
    Ok(parse_impl(input, options, None, Some(link_refs)))
}

/// Parse Markdown with options and an opt-in fenced-code renderer, returning
/// HTML, front matter, headings, and resource-limit fallbacks.
///
/// See [`FencedCodeRenderer`] for the escaping contract the renderer must
/// uphold.
///
/// # Panics
///
/// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use
/// [`try_parse_with_renderer`] to handle the limit as an error.
pub fn parse_with_renderer<'a>(
    input: &'a str,
    options: &Options,
    renderer: &mut dyn FencedCodeRenderer,
) -> ParseResult<'a> {
    try_parse_with_renderer(input, options, renderer).unwrap_or_else(|error| panic!("{error}"))
}

/// Parse Markdown with options and a fenced-code renderer without panicking
/// for oversized input.
pub fn try_parse_with_renderer<'a>(
    input: &'a str,
    options: &Options,
    renderer: &mut dyn FencedCodeRenderer,
) -> Result<ParseResult<'a>, InputSizeError> {
    validate_input_size(input.len())?;
    Ok(parse_impl(input, options, Some(renderer), None))
}

fn parse_impl<'a>(
    input: &'a str,
    options: &Options,
    renderer: Option<&mut dyn FencedCodeRenderer>,
    shared_link_refs: Option<&LinkRefStore>,
) -> ParseResult<'a> {
    let (front_matter, markdown) = if options.front_matter {
        match extract_front_matter(input) {
            Some((fm, offset)) => (Some(fm), &input[offset..]),
            None => (None, input),
        }
    } else {
        (None, input)
    };

    let mut headings = Vec::new();
    let mut resource_limits = ResourceLimitReport::default();
    let mut writer = HtmlWriter::with_capacity_for(markdown.len());
    render_to_writer_impl(
        markdown.as_bytes(),
        &mut writer,
        options,
        renderer,
        Some(&mut headings),
        Some(&mut resource_limits),
        shared_link_refs,
    );
    let html = writer
        .into_string()
        .expect("rendering from a UTF-8 Markdown string must produce UTF-8 HTML");
    ParseResult {
        html,
        front_matter,
        headings,
        resource_limits,
    }
}

/// Reusable Markdown-to-HTML rendering session.
///
/// A session keeps parser and renderer scratch buffers between documents. Use
/// it when rendering many documents with the same [`Options`], especially
/// small inputs where per-call allocations would otherwise dominate. A
/// `Renderer` is not shared implicitly; create one per worker or protect it
/// with application-level synchronization.
///
/// # Example
/// ```
/// use ferromark::Renderer;
///
/// let mut renderer = Renderer::new();
/// let mut html = Vec::new();
/// renderer.render_into("# First", &mut html);
/// assert_eq!(html, b"<h1 id=\"first\">First</h1>\n");
///
/// renderer.render_into("**Second**", &mut html);
/// assert_eq!(html, b"<p><strong>Second</strong></p>\n");
/// ```
pub struct Renderer {
    options: Options,
    block_events: Vec<BlockEvent>,
    inline_parser: InlineParser,
    inline_events: Vec<InlineEvent>,
    render_state: RenderState,
}

impl Renderer {
    /// Create a reusable renderer with [`Options::default`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_options(Options::default())
    }

    /// Create a reusable renderer with fixed options.
    #[must_use]
    pub fn with_options(options: Options) -> Self {
        Self {
            render_state: RenderState::new(&options),
            options,
            block_events: Vec::with_capacity(64),
            inline_parser: InlineParser::new(),
            inline_events: Vec::with_capacity(64),
        }
    }

    /// Return the options used by every document rendered by this session.
    #[must_use]
    pub const fn options(&self) -> &Options {
        &self.options
    }

    /// Render one Markdown document to an owned HTML string.
    ///
    /// # Panics
    ///
    /// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use
    /// [`Self::try_render`] to handle the limit as an error.
    #[must_use]
    pub fn render(&mut self, input: &str) -> String {
        self.try_render(input)
            .unwrap_or_else(|error| panic!("{error}"))
    }

    /// Render one Markdown document without panicking for oversized input.
    pub fn try_render(&mut self, input: &str) -> Result<String, InputSizeError> {
        validate_input_size(input.len())?;
        let markdown = markdown_without_front_matter(input, &self.options);
        let mut writer = HtmlWriter::with_capacity_for(markdown.len());
        self.render_to_writer(markdown.as_bytes(), &mut writer);
        Ok(writer
            .into_string()
            .expect("rendering from a UTF-8 Markdown string must produce UTF-8 HTML"))
    }

    /// Render one Markdown document into a caller-owned reusable buffer.
    ///
    /// The output buffer and this session's parser scratch allocations are
    /// retained for the next call.
    ///
    /// # Panics
    ///
    /// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use
    /// [`Self::try_render_into`] to handle the limit as an error.
    pub fn render_into(&mut self, input: &str, out: &mut Vec<u8>) {
        self.try_render_into(input, out)
            .unwrap_or_else(|error| panic!("{error}"));
    }

    /// Render into a caller-owned buffer without panicking for oversized
    /// input.
    pub fn try_render_into(
        &mut self,
        input: &str,
        out: &mut Vec<u8>,
    ) -> Result<(), InputSizeError> {
        validate_input_size(input.len())?;
        let markdown = markdown_without_front_matter(input, &self.options);
        out.clear();
        out.reserve(markdown.len() + markdown.len() / 4);
        let mut writer = HtmlWriter::with_capacity(0);
        std::mem::swap(writer.buffer_mut(), out);
        self.render_to_writer(markdown.as_bytes(), &mut writer);
        std::mem::swap(writer.buffer_mut(), out);
        Ok(())
    }

    fn render_to_writer(&mut self, input: &[u8], writer: &mut HtmlWriter) {
        render_to_writer_with_state::<DisabledFencedCodeRenderer>(
            input,
            writer,
            &self.options,
            None,
            None,
            None,
            &mut self.block_events,
            &mut self.inline_parser,
            &mut self.inline_events,
            &mut self.render_state,
            None,
        );
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Markdown to HTML.
///
/// This is the primary API for simple use cases.
///
/// # Example
/// ```
/// let html = ferromark::to_html("# Hello\n\nWorld");
/// assert!(html.contains("Hello</h1>"));
/// assert!(html.contains("<p>World</p>"));
/// ```
///
/// # Panics
///
/// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use [`try_to_html`] to
/// handle the limit as an error.
pub fn to_html(input: &str) -> String {
    try_to_html(input).unwrap_or_else(|error| panic!("{error}"))
}

/// Convert Markdown to HTML without panicking for oversized input.
pub fn try_to_html(input: &str) -> Result<String, InputSizeError> {
    try_to_html_with_options(input, &Options::default())
}

/// Convert Markdown to HTML, writing into a provided buffer.
///
/// This reuses only the output buffer. Use [`Renderer`] to retain parser and
/// renderer scratch allocations across documents as well.
///
/// # Example
///
/// ```
/// let mut output = Vec::new();
/// ferromark::to_html_into("**Hello**", &mut output);
///
/// assert_eq!(output, b"<p><strong>Hello</strong></p>\n");
/// ```
///
/// # Panics
///
/// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use
/// [`try_to_html_into`] to handle the limit as an error.
pub fn to_html_into(input: &str, out: &mut Vec<u8>) {
    try_to_html_into(input, out).unwrap_or_else(|error| panic!("{error}"));
}

/// Convert Markdown to HTML into a provided buffer without panicking for
/// oversized input.
pub fn try_to_html_into(input: &str, out: &mut Vec<u8>) -> Result<(), InputSizeError> {
    try_to_html_into_with_options(input, out, &Options::default())
}

/// Convert Markdown to HTML with options.
///
/// When `options.front_matter` is `true`, any front matter at the start of the
/// document is silently stripped before parsing.
///
/// # Example
///
/// ```
/// use ferromark::{Options, to_html_with_options};
///
/// let mut options = Options::gfm();
/// options.heading_ids = true;
///
/// assert_eq!(
///     to_html_with_options("# Hello", &options),
///     "<h1 id=\"hello\">Hello</h1>\n"
/// );
/// ```
///
/// # Panics
///
/// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use
/// [`try_to_html_with_options`] to handle the limit as an error.
pub fn to_html_with_options(input: &str, options: &Options) -> String {
    try_to_html_with_options(input, options).unwrap_or_else(|error| panic!("{error}"))
}

/// Convert Markdown to HTML with options without panicking for oversized input.
pub fn try_to_html_with_options(input: &str, options: &Options) -> Result<String, InputSizeError> {
    validate_input_size(input.len())?;
    let markdown = markdown_without_front_matter(input, options);
    let mut writer = HtmlWriter::with_capacity_for(markdown.len());
    render_to_writer(markdown.as_bytes(), &mut writer, options);
    Ok(writer
        .into_string()
        .expect("rendering from a UTF-8 Markdown string must produce UTF-8 HTML"))
}

fn markdown_without_front_matter<'a>(input: &'a str, options: &Options) -> &'a str {
    if options.front_matter {
        match extract_front_matter(input) {
            Some((_, offset)) => &input[offset..],
            None => input,
        }
    } else {
        input
    }
}

/// Convert Markdown to HTML with an opt-in fenced-code renderer.
///
/// The renderer sees only fenced code blocks. Returning `None` preserves the
/// normal escaped code-block output. Installing a renderer is an explicit HTML
/// trust boundary; see [`TrustedHtml`].
///
/// # Panics
///
/// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use
/// [`try_to_html_with_renderer`] to handle the limit as an error.
pub fn to_html_with_renderer(
    input: &str,
    options: &Options,
    renderer: &mut dyn FencedCodeRenderer,
) -> String {
    try_to_html_with_renderer(input, options, renderer).unwrap_or_else(|error| panic!("{error}"))
}

/// Convert Markdown to HTML with a fenced-code renderer without panicking for
/// oversized input.
pub fn try_to_html_with_renderer(
    input: &str,
    options: &Options,
    renderer: &mut dyn FencedCodeRenderer,
) -> Result<String, InputSizeError> {
    validate_input_size(input.len())?;
    let markdown = markdown_without_front_matter(input, options);
    let mut writer = HtmlWriter::with_capacity_for(markdown.len());
    render_to_writer_with_renderer(markdown.as_bytes(), &mut writer, options, Some(renderer));
    Ok(writer
        .into_string()
        .expect("rendering from a UTF-8 Markdown string must produce UTF-8 HTML"))
}

/// Convert Markdown to HTML into a provided buffer with options.
///
/// When `options.front_matter` is `true`, any front matter at the start of the
/// document is silently stripped before parsing.
///
/// # Panics
///
/// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use
/// [`try_to_html_into_with_options`] to handle the limit as an error.
pub fn to_html_into_with_options(input: &str, out: &mut Vec<u8>, options: &Options) {
    try_to_html_into_with_options(input, out, options).unwrap_or_else(|error| panic!("{error}"));
}

/// Convert Markdown into a provided buffer with options without panicking for
/// oversized input.
pub fn try_to_html_into_with_options(
    input: &str,
    out: &mut Vec<u8>,
    options: &Options,
) -> Result<(), InputSizeError> {
    validate_input_size(input.len())?;
    let markdown = markdown_without_front_matter(input, options);
    out.clear();
    out.reserve(markdown.len() + markdown.len() / 4);
    let mut writer = HtmlWriter::with_capacity(0);
    // Use the provided buffer directly
    std::mem::swap(writer.buffer_mut(), out);
    render_to_writer(markdown.as_bytes(), &mut writer, options);
    std::mem::swap(writer.buffer_mut(), out);
    Ok(())
}

/// Convert Markdown into a reusable buffer with an opt-in fenced-code renderer.
///
/// # Panics
///
/// Panics when the input exceeds [`MAX_INPUT_BYTES`]. Use
/// [`try_to_html_into_with_renderer`] to handle the limit as an error.
pub fn to_html_into_with_renderer(
    input: &str,
    out: &mut Vec<u8>,
    options: &Options,
    renderer: &mut dyn FencedCodeRenderer,
) {
    try_to_html_into_with_renderer(input, out, options, renderer)
        .unwrap_or_else(|error| panic!("{error}"));
}

/// Convert Markdown into a reusable buffer with a fenced-code renderer without
/// panicking for oversized input.
pub fn try_to_html_into_with_renderer(
    input: &str,
    out: &mut Vec<u8>,
    options: &Options,
    renderer: &mut dyn FencedCodeRenderer,
) -> Result<(), InputSizeError> {
    validate_input_size(input.len())?;
    let markdown = markdown_without_front_matter(input, options);
    out.clear();
    out.reserve(markdown.len() + markdown.len() / 4);
    let mut writer = HtmlWriter::with_capacity(0);
    std::mem::swap(writer.buffer_mut(), out);
    render_to_writer_with_renderer(markdown.as_bytes(), &mut writer, options, Some(renderer));
    std::mem::swap(writer.buffer_mut(), out);
    Ok(())
}

/// State for collecting paragraph content before inline parsing.
struct ParagraphState {
    /// Collected text content (joined with newlines).
    content: Vec<u8>,
    /// Whether we're currently in a paragraph.
    in_paragraph: bool,
}

impl ParagraphState {
    fn new() -> Self {
        Self {
            content: Vec::with_capacity(256),
            in_paragraph: false,
        }
    }

    fn start(&mut self) {
        self.in_paragraph = true;
        self.content.clear();
    }

    fn reset(&mut self) {
        self.content.clear();
        self.in_paragraph = false;
    }

    fn add_text(&mut self, text: &[u8]) {
        #[cfg(feature = "profiling")]
        profiling::record_paragraph_copy(text.len());
        self.content.extend_from_slice(text);
    }

    fn add_soft_break(&mut self) {
        #[cfg(feature = "profiling")]
        profiling::record_paragraph_copy(1);
        self.content.push(b'\n');
    }

    fn finish(&mut self) -> &[u8] {
        self.in_paragraph = false;
        // CommonMark: strip trailing spaces/tabs from paragraph content
        while self
            .content
            .last()
            .is_some_and(|&b| b == b' ' || b == b'\t')
        {
            self.content.pop();
        }
        &self.content
    }
}

/// State for collecting heading content before inline parsing.
struct HeadingState {
    /// Collected text content (joined with newlines).
    content: Vec<u8>,
    /// Whether we're currently in a heading.
    in_heading: bool,
    /// Current heading level (stored for deferred tag emission).
    level: u8,
}

impl HeadingState {
    fn new() -> Self {
        Self {
            content: Vec::with_capacity(64),
            in_heading: false,
            level: 0,
        }
    }

    fn start(&mut self) {
        self.in_heading = true;
        self.content.clear();
    }

    fn reset(&mut self) {
        self.content.clear();
        self.in_heading = false;
        self.level = 0;
    }

    fn add_text(&mut self, text: &[u8]) {
        self.content.extend_from_slice(text);
    }

    fn add_soft_break(&mut self) {
        self.content.push(b'\n');
    }

    fn finish(&mut self) -> &[u8] {
        self.in_heading = false;
        while self
            .content
            .last()
            .is_some_and(|&b| b == b' ' || b == b'\t')
        {
            self.content.pop();
        }
        &self.content
    }
}

/// Tracker for deduplicating heading IDs.
///
/// Uses the crate's fast non-cryptographic hasher: heading slugs are short
/// and not a hash-DoS surface, so SipHash's cost is not warranted.
///
/// Base slugs live in a single append-only arena; the dedup map is keyed by
/// the slug's hash and stores arena ranges, so recording a new slug never
/// allocates a dedicated key. Distinct slugs that collide on the 64-bit hash
/// share a map entry and are told apart by comparing arena bytes.
struct HeadingIdTracker {
    /// All base slugs seen so far, concatenated.
    arena: Vec<u8>,
    /// Maps a slug hash to the entries whose slug has that hash.
    used: std::collections::HashMap<u64, SmallVec<[SlugEntry; 1]>, rustc_hash::FxBuildHasher>,
    /// Reusable buffer holding the id returned by `make_id`.
    slug_buf: Vec<u8>,
}

/// One recorded base slug: its bytes in the arena and how often it repeated.
struct SlugEntry {
    /// Start offset of the slug in the arena.
    start: usize,
    /// Length of the slug in bytes.
    len: usize,
    /// How many times this base slug has been seen after the first.
    seen: usize,
}

impl HeadingIdTracker {
    fn new() -> Self {
        Self {
            arena: Vec::with_capacity(256),
            used: std::collections::HashMap::with_capacity_and_hasher(
                32,
                rustc_hash::FxBuildHasher,
            ),
            slug_buf: Vec::with_capacity(64),
        }
    }

    fn reset(&mut self) {
        self.arena.clear();
        self.used.clear();
        self.slug_buf.clear();
    }

    /// Build a unique heading id from raw heading content, appending `-1`,
    /// `-2`, etc. on collision. The returned slice borrows the internal
    /// buffer and is valid until the next call. Recording a slug appends to
    /// the arena instead of allocating a per-heading map key.
    fn make_id(&mut self, raw: &[u8]) -> &str {
        generate_slug_into(raw, &mut self.slug_buf);
        if self.slug_buf.is_empty() {
            self.slug_buf.extend_from_slice(b"heading");
        }
        // Dedup on raw slug bytes; UTF-8 validity is checked once on return.
        // The slug is valid UTF-8 by construction: `generate_slug_into` only
        // removes whole ASCII bytes and lowercases ASCII, which cannot split
        // multibyte sequences in UTF-8 heading text.
        let hash = {
            use std::hash::{BuildHasher, Hasher};
            let mut hasher = rustc_hash::FxBuildHasher.build_hasher();
            hasher.write(&self.slug_buf);
            hasher.finish()
        };
        let entries = self.used.entry(hash).or_default();
        let arena = &self.arena;
        let slug = self.slug_buf.as_slice();
        match entries
            .iter_mut()
            .find(|e| &arena[e.start..e.start + e.len] == slug)
        {
            Some(entry) => {
                entry.seen += 1;
                let n = entry.seen;
                self.slug_buf.push(b'-');
                push_decimal(&mut self.slug_buf, n);
            }
            None => {
                entries.push(SlugEntry {
                    start: self.arena.len(),
                    len: self.slug_buf.len(),
                    seen: 0,
                });
                self.arena.extend_from_slice(&self.slug_buf);
            }
        }
        std::str::from_utf8(&self.slug_buf).unwrap_or("heading")
    }
}

/// Append the decimal representation of `n` to `buf`.
fn push_decimal(buf: &mut Vec<u8>, mut n: usize) {
    let mut digits = [0u8; 20];
    let mut i = digits.len();
    loop {
        i -= 1;
        digits[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    buf.extend_from_slice(&digits[i..]);
}

/// Generate a GitHub-compatible slug from raw heading text.
///
/// Steps:
/// 1. Strip inline markup delimiters (`*`, `_`, `~`, `` ` ``, `[`, `]`, `!`, `#`)
/// 2. Lowercase
/// 3. Replace whitespace runs with `-`
/// 4. Remove chars that are not alphanumeric, `-`, `_`, or space
/// 5. Strip leading/trailing `-`
fn generate_slug_into(raw: &[u8], slug: &mut Vec<u8>) {
    /// Skip the byte without touching whitespace state (markup delimiters).
    const SLUG_MARKUP: u8 = 0;
    /// Fold a whitespace run into a single `-`.
    const SLUG_SPACE: u8 = 1;
    /// Drop the byte but end any pending whitespace run (other punctuation).
    const SLUG_DROP: u8 = 2;
    /// Any other value is the (lowercased) byte to append.
    static SLUG_LUT: [u8; 256] = {
        let mut table = [SLUG_DROP; 256];
        let mut i = 0usize;
        while i < 256 {
            let b = i as u8;
            table[i] = match b {
                // Strip inline markup delimiters (keep _ since it's valid in slugs)
                b'*' | b'~' | b'`' | b'[' | b']' | b'!' | b'#' => SLUG_MARKUP,
                b' ' | b'\t' | b'\n' | b'\r' => SLUG_SPACE,
                // Lowercase ASCII
                b'A'..=b'Z' => b + 32,
                // Keep alphanumeric, hyphen, underscore, and multibyte UTF-8
                b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => b,
                0x80..=0xFF => b,
                _ => SLUG_DROP,
            };
            i += 1;
        }
        table
    };

    slug.clear();
    let mut prev_was_space = false;

    for &b in raw {
        match SLUG_LUT[b as usize] {
            SLUG_MARKUP => {}
            SLUG_SPACE => {
                if !prev_was_space && !slug.is_empty() {
                    slug.push(b'-');
                    prev_was_space = true;
                }
            }
            SLUG_DROP => prev_was_space = false,
            ch => {
                prev_was_space = false;
                slug.push(ch);
            }
        }
    }

    // Strip trailing hyphen
    while slug.last() == Some(&b'-') {
        slug.pop();
    }
    // Strip leading hyphen
    let leading = slug.iter().take_while(|&&b| b == b'-').count();
    if leading > 0 {
        slug.drain(..leading);
    }
}

/// State for collecting table cell content before inline parsing.
struct CellState {
    /// Collected text content (only used when escapes force a copy).
    content: Vec<u8>,
    /// Borrowed input range for the common single-text, no-escape cell.
    pending: Option<Range>,
    /// Whether we're currently in a cell.
    in_cell: bool,
}

impl CellState {
    fn new() -> Self {
        Self {
            content: Vec::with_capacity(64),
            pending: None,
            in_cell: false,
        }
    }

    fn start(&mut self) {
        self.in_cell = true;
        self.content.clear();
        self.pending = None;
    }

    fn reset(&mut self) {
        self.content.clear();
        self.pending = None;
        self.in_cell = false;
    }

    fn add_text(&mut self, range: Range, input: &[u8]) {
        let text = range.slice(input);
        // Fast path: a cell is almost always a single text range without a
        // backslash escape, which can be rendered straight from the input.
        if self.content.is_empty()
            && self.pending.is_none()
            && memchr::memchr(b'\\', text).is_none()
        {
            self.pending = Some(range);
            return;
        }
        if let Some(prev) = self.pending.take() {
            self.content.extend_from_slice(prev.slice(input));
        }
        // In table cells, \| is a table-level escape meaning literal |
        // Replace \| with | before inline parsing. Bytes between escapes are
        // copied in bulk.
        let mut i = 0;
        while let Some(offset) = memchr::memchr(b'\\', &text[i..]) {
            let idx = i + offset;
            if idx + 1 < text.len() && text[idx + 1] == b'|' {
                self.content.extend_from_slice(&text[i..idx]);
                self.content.push(b'|');
                i = idx + 2;
            } else {
                self.content.extend_from_slice(&text[i..=idx]);
                i = idx + 1;
            }
        }
        self.content.extend_from_slice(&text[i..]);
    }

    fn finish<'a>(&'a mut self, input: &'a [u8]) -> &'a [u8] {
        self.in_cell = false;
        let mut slice: &[u8] = match self.pending.take() {
            Some(range) => range.slice(input),
            None => {
                // Trim trailing whitespace
                while self
                    .content
                    .last()
                    .is_some_and(|&b| b == b' ' || b == b'\t')
                {
                    self.content.pop();
                }
                return &self.content;
            }
        };
        while slice.last().is_some_and(|&b| b == b' ' || b == b'\t') {
            slice = &slice[..slice.len() - 1];
        }
        slice
    }
}

/// Buffered state used only while a custom renderer handles a fenced block.
struct FencedCodeState {
    info: Option<Range>,
}

impl FencedCodeState {
    fn new(info: Option<Range>) -> Self {
        Self { info }
    }
}

/// Scratch buffers and mutable state retained by a reusable renderer.
struct RenderState {
    para_state: ParagraphState,
    heading_state: HeadingState,
    cell_state: CellState,
    tight_list_stack: Vec<(bool, u32)>,
    at_tight_li_start: bool,
    need_newline_before_block: bool,
    pending_loose_li_newline: bool,
    blockquote_depth: u32,
    in_table_head: bool,
    pending_task: block::TaskState,
    footnote_numbers: FootnoteNumbers,
    heading_id_tracker: Option<HeadingIdTracker>,
    callout_stack: Vec<Option<block::CalloutType>>,
    pending_footnote_backref: Option<(String, usize)>,
    definition_description_stack: Vec<bool>,
    paragraph_tags_suppressed: bool,
    fenced_code_state: Option<FencedCodeState>,
    fenced_code_buffer: Vec<u8>,
}

impl RenderState {
    fn new(options: &Options) -> Self {
        Self {
            para_state: ParagraphState::new(),
            heading_state: HeadingState::new(),
            cell_state: CellState::new(),
            tight_list_stack: Vec::new(),
            at_tight_li_start: false,
            need_newline_before_block: false,
            pending_loose_li_newline: false,
            blockquote_depth: 0,
            in_table_head: false,
            pending_task: block::TaskState::None,
            footnote_numbers: FootnoteNumbers::new(0),
            heading_id_tracker: options.heading_ids.then(HeadingIdTracker::new),
            callout_stack: Vec::new(),
            pending_footnote_backref: None,
            definition_description_stack: Vec::new(),
            paragraph_tags_suppressed: false,
            fenced_code_state: None,
            fenced_code_buffer: Vec::new(),
        }
    }

    fn reset(&mut self, options: &Options, footnote_definition_count: usize) {
        self.para_state.reset();
        self.heading_state.reset();
        self.cell_state.reset();
        self.tight_list_stack.clear();
        self.at_tight_li_start = false;
        self.need_newline_before_block = false;
        self.pending_loose_li_newline = false;
        self.blockquote_depth = 0;
        self.in_table_head = false;
        self.pending_task = block::TaskState::None;
        self.footnote_numbers.reset(footnote_definition_count);
        match (options.heading_ids, self.heading_id_tracker.as_mut()) {
            (true, Some(tracker)) => tracker.reset(),
            (true, None) => self.heading_id_tracker = Some(HeadingIdTracker::new()),
            (false, _) => self.heading_id_tracker = None,
        }
        self.callout_stack.clear();
        self.pending_footnote_backref = None;
        self.definition_description_stack.clear();
        self.paragraph_tags_suppressed = false;
        self.fenced_code_state = None;
        self.fenced_code_buffer.clear();
    }
}

/// Mutable state and shared inputs for one HTML rendering pass.
struct RenderContext<'a, 'r, R: FencedCodeRenderer + ?Sized> {
    writer: &'a mut HtmlWriter,
    inline_parser: &'a mut InlineParser,
    inline_events: &'a mut Vec<InlineEvent>,
    para_state: &'a mut ParagraphState,
    heading_state: &'a mut HeadingState,
    cell_state: &'a mut CellState,
    tight_list_stack: &'a mut Vec<(bool, u32)>,
    at_tight_li_start: &'a mut bool,
    need_newline_before_block: &'a mut bool,
    pending_loose_li_newline: &'a mut bool,
    blockquote_depth: &'a mut u32,
    in_table_head: &'a mut bool,
    pending_task: &'a mut block::TaskState,
    link_refs: &'a LinkRefStore,
    footnote_store: Option<&'a FootnoteStore>,
    footnote_numbers: &'a mut FootnoteNumbers,
    heading_id_tracker: &'a mut Option<HeadingIdTracker>,
    callout_stack: &'a mut Vec<Option<block::CalloutType>>,
    pending_footnote_backref: &'a mut Option<(String, usize)>,
    definition_description_stack: &'a mut Vec<bool>,
    paragraph_tags_suppressed: &'a mut bool,
    options: &'a Options,
    fenced_code_renderer: Option<&'r mut R>,
    fenced_code_state: &'a mut Option<FencedCodeState>,
    fenced_code_buffer: &'a mut Vec<u8>,
    headings: Option<&'a mut Vec<Heading>>,
}

impl<'a, 'r, R: FencedCodeRenderer + ?Sized> RenderContext<'a, 'r, R> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        writer: &'a mut HtmlWriter,
        inline_parser: &'a mut InlineParser,
        inline_events: &'a mut Vec<InlineEvent>,
        state: &'a mut RenderState,
        link_refs: &'a LinkRefStore,
        footnote_store: Option<&'a FootnoteStore>,
        options: &'a Options,
        fenced_code_renderer: Option<&'r mut R>,
        headings: Option<&'a mut Vec<Heading>>,
    ) -> Self {
        state.reset(options, footnote_store.map_or(0, FootnoteStore::len));
        Self {
            writer,
            inline_parser,
            inline_events,
            para_state: &mut state.para_state,
            heading_state: &mut state.heading_state,
            cell_state: &mut state.cell_state,
            tight_list_stack: &mut state.tight_list_stack,
            at_tight_li_start: &mut state.at_tight_li_start,
            need_newline_before_block: &mut state.need_newline_before_block,
            pending_loose_li_newline: &mut state.pending_loose_li_newline,
            blockquote_depth: &mut state.blockquote_depth,
            in_table_head: &mut state.in_table_head,
            pending_task: &mut state.pending_task,
            link_refs,
            footnote_store,
            footnote_numbers: &mut state.footnote_numbers,
            heading_id_tracker: &mut state.heading_id_tracker,
            callout_stack: &mut state.callout_stack,
            pending_footnote_backref: &mut state.pending_footnote_backref,
            definition_description_stack: &mut state.definition_description_stack,
            paragraph_tags_suppressed: &mut state.paragraph_tags_suppressed,
            options,
            fenced_code_renderer,
            fenced_code_state: &mut state.fenced_code_state,
            fenced_code_buffer: &mut state.fenced_code_buffer,
            headings,
        }
    }
}

/// Render Markdown to an HtmlWriter.
fn render_to_writer(input: &[u8], writer: &mut HtmlWriter, options: &Options) {
    render_to_writer_impl::<DisabledFencedCodeRenderer>(
        input, writer, options, None, None, None, None,
    );
}

fn render_to_writer_with_renderer(
    input: &[u8],
    writer: &mut HtmlWriter,
    options: &Options,
    fenced_code_renderer: Option<&mut dyn FencedCodeRenderer>,
) {
    render_to_writer_impl(
        input,
        writer,
        options,
        fenced_code_renderer,
        None,
        None,
        None,
    );
}

struct DisabledFencedCodeRenderer;

impl FencedCodeRenderer for DisabledFencedCodeRenderer {
    fn render(&mut self, _: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        unreachable!("the default render path never installs a fenced-code renderer")
    }
}

fn render_to_writer_impl<R: FencedCodeRenderer + ?Sized>(
    input: &[u8],
    writer: &mut HtmlWriter,
    options: &Options,
    fenced_code_renderer: Option<&mut R>,
    headings: Option<&mut Vec<Heading>>,
    resource_limits: Option<&mut ResourceLimitReport>,
    shared_link_refs: Option<&LinkRefStore>,
) {
    let mut events = Vec::with_capacity((input.len() / 16).max(64));
    let mut inline_parser = InlineParser::new();
    let mut inline_events = Vec::with_capacity(64);
    let mut render_state = RenderState::new(options);
    render_to_writer_with_state(
        input,
        writer,
        options,
        fenced_code_renderer,
        headings,
        resource_limits,
        &mut events,
        &mut inline_parser,
        &mut inline_events,
        &mut render_state,
        shared_link_refs,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_to_writer_with_state<R: FencedCodeRenderer + ?Sized>(
    input: &[u8],
    writer: &mut HtmlWriter,
    options: &Options,
    fenced_code_renderer: Option<&mut R>,
    headings: Option<&mut Vec<Heading>>,
    mut resource_limits: Option<&mut ResourceLimitReport>,
    events: &mut Vec<BlockEvent>,
    inline_parser: &mut InlineParser,
    inline_events: &mut Vec<InlineEvent>,
    render_state: &mut RenderState,
    shared_link_refs: Option<&LinkRefStore>,
) {
    // Parse blocks
    events.clear();
    events.reserve((input.len() / 16).max(64));
    let mut parser = BlockParser::new_with_options(input, options.clone());
    parser.parse(events);
    if let Some(report) = resource_limits.as_deref_mut() {
        report.extend(parser.resource_limits());
    }
    #[cfg(feature = "profiling")]
    profiling::record_block_events(events, events.capacity());
    let segment_link_refs = parser.take_link_refs();
    let footnote_store = if options.footnotes {
        Some(parser.take_footnote_store())
    } else {
        None
    };

    // Fix up list tight status (ListStart gets its tight value from ListEnd)
    fixup_list_tight(events);

    let link_refs = shared_link_refs.unwrap_or(&segment_link_refs);
    let fn_store_ref = footnote_store.as_ref();
    inline_parser.begin_document();
    {
        let mut context = RenderContext::new(
            writer,
            inline_parser,
            inline_events,
            render_state,
            link_refs,
            fn_store_ref,
            options,
            fenced_code_renderer,
            headings,
        );

        // Render events to HTML
        for event in events.iter() {
            context.render_block_event(input, event);
        }

        // Render footnote section at document end
        if !context.footnote_numbers.is_empty() {
            context.render_footnote_section(input);
        }
    }
    if let Some(report) = resource_limits {
        report.extend(inline_parser.resource_limits());
    }
}

impl<R: FencedCodeRenderer + ?Sized> RenderContext<'_, '_, R> {
    /// Render a single block event using the context's explicit state boundary.
    fn render_block_event(&mut self, input: &[u8], event: &BlockEvent) {
        let writer = &mut *self.writer;
        let inline_parser = &mut *self.inline_parser;
        let inline_events = &mut *self.inline_events;
        let para_state = &mut *self.para_state;
        let heading_state = &mut *self.heading_state;
        let cell_state = &mut *self.cell_state;
        let tight_list_stack = &mut *self.tight_list_stack;
        let at_tight_li_start = &mut *self.at_tight_li_start;
        let need_newline_before_block = &mut *self.need_newline_before_block;
        let pending_loose_li_newline = &mut *self.pending_loose_li_newline;
        let blockquote_depth = &mut *self.blockquote_depth;
        let in_table_head = &mut *self.in_table_head;
        let pending_task = &mut *self.pending_task;
        let link_refs = self.link_refs;
        let footnote_store = self.footnote_store;
        let footnote_numbers = &mut *self.footnote_numbers;
        let heading_id_tracker = &mut *self.heading_id_tracker;
        let callout_stack = &mut *self.callout_stack;
        let pending_footnote_backref = &mut *self.pending_footnote_backref;
        let definition_description_stack = &mut *self.definition_description_stack;
        let paragraph_tags_suppressed = &mut *self.paragraph_tags_suppressed;
        let options = self.options;
        let fenced_code_renderer = &mut self.fenced_code_renderer;
        let fenced_code_state = &mut *self.fenced_code_state;
        let fenced_code_buffer = &mut *self.fenced_code_buffer;
        let headings = &mut self.headings;

        // Check if we're in a tight list (innermost list is tight)
        // BUT: paragraphs inside blockquotes that started AFTER the list need <p> tags
        let in_tight_list = tight_list_stack
            .last()
            .is_some_and(|(tight, bq_depth_at_start)| {
                *tight && *blockquote_depth <= *bq_depth_at_start
            });

        match event {
            BlockEvent::ParagraphStart => {
                // Write pending newline from loose list item start
                if *pending_loose_li_newline {
                    writer.newline();
                    *pending_loose_li_newline = false;
                }
                let in_tight_definition = definition_description_stack.last_mut().is_some_and(
                    |suppress_first_paragraph| {
                        let suppress = *suppress_first_paragraph;
                        *suppress_first_paragraph = false;
                        suppress
                    },
                );
                *paragraph_tags_suppressed = in_tight_list || in_tight_definition;
                if !*paragraph_tags_suppressed {
                    writer.paragraph_start();
                }
                para_state.start();
                // Paragraph content is inline, so we don't add newline
                *at_tight_li_start = false;
            }
            BlockEvent::ParagraphEnd => {
                // Parse all accumulated paragraph content at once
                let content = para_state.finish();

                // Emit pending task checkbox before paragraph content
                emit_pending_task_checkbox(pending_task, writer);

                if !content.is_empty() {
                    render_inline_content(
                        content,
                        writer,
                        inline_parser,
                        inline_events,
                        link_refs,
                        footnote_store,
                        footnote_numbers,
                        options,
                    );
                }
                if let Some((label, number)) = pending_footnote_backref.take() {
                    write_footnote_backref(writer, &label, number);
                }
                if !*paragraph_tags_suppressed {
                    writer.paragraph_end();
                } else {
                    // Mark that we need newline before next block element
                    *need_newline_before_block = true;
                }
                *paragraph_tags_suppressed = false;
            }
            BlockEvent::HeadingStart { level } => {
                if *need_newline_before_block {
                    writer.newline();
                    *need_newline_before_block = false;
                }
                if *at_tight_li_start {
                    writer.newline();
                    *at_tight_li_start = false;
                }
                // Defer heading open tag to HeadingEnd so we can generate the slug
                // from collected content before emitting the tag.
                heading_state.start();
                heading_state.level = *level;
            }
            BlockEvent::HeadingEnd { level } => {
                let content = heading_state.finish();

                // Emit heading open tag (deferred from HeadingStart)
                let mut collected_id = None;
                if let Some(tracker) = heading_id_tracker.as_mut() {
                    let id = tracker.make_id(content);
                    writer.heading_start_with_id(*level, id);
                    if headings.is_some() {
                        collected_id = Some(id.to_owned());
                    }
                } else {
                    writer.heading_start(*level);
                }

                let text_start = writer.len();
                if !content.is_empty() {
                    render_inline_content(
                        content,
                        writer,
                        inline_parser,
                        inline_events,
                        link_refs,
                        footnote_store,
                        footnote_numbers,
                        options,
                    );
                }
                if let Some(collector) = headings.as_deref_mut() {
                    collector.push(Heading {
                        level: *level,
                        id: collected_id,
                        text: heading_plain_text(&writer.as_bytes()[text_start..]),
                    });
                }
                writer.heading_end(*level);
            }
            BlockEvent::ThematicBreak(_) => {
                // If we're at the start of a tight list item, add newline before block content
                if *at_tight_li_start {
                    writer.newline();
                    *at_tight_li_start = false;
                }
                writer.thematic_break();
            }
            BlockEvent::Comment(_) => {}
            BlockEvent::HtmlBlockStart => {
                // Write pending newline from loose list item start
                if *pending_loose_li_newline {
                    writer.newline();
                    *pending_loose_li_newline = false;
                }
                // If we're at the start of a tight list item, add newline before block content
                if *at_tight_li_start {
                    writer.newline();
                    *at_tight_li_start = false;
                }
            }
            BlockEvent::HtmlBlockText(range) => {
                if options.render_policy == RenderPolicy::Untrusted {
                    writer.write_escaped_text(range.slice(input));
                } else if options.disallowed_raw_html {
                    writer.write_html_filtered(range.slice(input));
                } else {
                    writer.write_bytes(range.slice(input));
                }
            }
            BlockEvent::HtmlBlockEnd => {}
            BlockEvent::SoftBreak => {
                if para_state.in_paragraph {
                    para_state.add_soft_break();
                } else if heading_state.in_heading {
                    heading_state.add_soft_break();
                } else {
                    writer.write_str("\n");
                }
            }
            BlockEvent::Text(range) => {
                let text = range.slice(input);
                if para_state.in_paragraph {
                    // Accumulate for later parsing
                    para_state.add_text(text);
                } else if heading_state.in_heading {
                    heading_state.add_text(text);
                } else if cell_state.in_cell {
                    cell_state.add_text(*range, input);
                } else {
                    render_inline_content(
                        text,
                        writer,
                        inline_parser,
                        inline_events,
                        link_refs,
                        footnote_store,
                        footnote_numbers,
                        options,
                    );
                }
            }
            BlockEvent::Code(range) => {
                // Code block content - no inline parsing
                if fenced_code_state.is_some() {
                    fenced_code_buffer.extend_from_slice(range.slice(input));
                } else {
                    writer.write_escaped_text(range.slice(input));
                }
            }
            BlockEvent::VirtualSpaces(count) => {
                // Emit spaces for tab expansion in indented code blocks
                if fenced_code_state.is_some() {
                    fenced_code_buffer.extend(std::iter::repeat_n(b' ', *count as usize));
                } else {
                    for _ in 0..*count {
                        writer.write_byte(b' ');
                    }
                }
            }
            BlockEvent::CodeBlockStart { kind } => {
                // Write pending newline from loose list item start
                if *pending_loose_li_newline {
                    writer.newline();
                    *pending_loose_li_newline = false;
                }
                // If we're at the start of a tight list item, add newline before block content
                if *at_tight_li_start {
                    writer.newline();
                    *at_tight_li_start = false;
                }
                match kind {
                    CodeBlockKind::Fenced { info } if fenced_code_renderer.is_some() => {
                        fenced_code_buffer.clear();
                        *fenced_code_state = Some(FencedCodeState::new(*info));
                    }
                    CodeBlockKind::Fenced { info } => {
                        writer.code_block_start(info.as_ref().map(|range| range.slice(input)));
                    }
                    CodeBlockKind::Indented => writer.code_block_start(None),
                }
            }
            BlockEvent::CodeBlockEnd => {
                if let Some(state) = fenced_code_state.take() {
                    let language = state
                        .info
                        .map(|range| HtmlWriter::decode_info_word(range.slice(input)));
                    let meta = state
                        .info
                        .and_then(|range| HtmlWriter::decode_info_meta(range.slice(input)));
                    let code = std::str::from_utf8(fenced_code_buffer)
                        .expect("fenced code originates from UTF-8 Markdown input");
                    let rendered = fenced_code_renderer.as_deref_mut().and_then(|renderer| {
                        renderer.render(FencedCodeBlock {
                            language: language.as_deref().filter(|value| !value.is_empty()),
                            meta: meta.as_deref(),
                            code,
                        })
                    });

                    if let Some(html) = rendered {
                        writer.write_string(html.as_str());
                    } else {
                        writer
                            .code_block_start(state.info.as_ref().map(|range| range.slice(input)));
                        writer.write_escaped_text(fenced_code_buffer);
                        writer.code_block_end();
                    }
                    fenced_code_buffer.clear();
                } else {
                    writer.code_block_end();
                }
            }
            BlockEvent::BlockQuoteStart { callout } => {
                // Write pending newline from loose list item start
                if *pending_loose_li_newline {
                    writer.newline();
                    *pending_loose_li_newline = false;
                }
                // If we need newline (after paragraph in tight list), add it
                if *need_newline_before_block {
                    writer.newline();
                    *need_newline_before_block = false;
                }
                // If we're at the start of a tight list item, add newline before block content
                if *at_tight_li_start {
                    writer.newline();
                    *at_tight_li_start = false;
                }
                *blockquote_depth += 1;
                callout_stack.push(*callout);
                if let Some(ct) = callout {
                    writer.callout_start(*ct);
                } else {
                    writer.blockquote_start();
                }
            }
            BlockEvent::BlockQuoteEnd => {
                *blockquote_depth = blockquote_depth.saturating_sub(1);
                match callout_stack.pop() {
                    Some(Some(_)) => writer.callout_end(),
                    _ => writer.blockquote_end(),
                }
            }
            BlockEvent::ListStart { kind, tight } => {
                // Write pending newline from loose list item start
                if *pending_loose_li_newline {
                    writer.newline();
                    *pending_loose_li_newline = false;
                }
                // If we need newline (after paragraph in tight list), add it
                if *need_newline_before_block {
                    writer.newline();
                    *need_newline_before_block = false;
                }
                // If we're at the start of a tight list item, add newline before nested list
                if *at_tight_li_start {
                    writer.newline();
                    *at_tight_li_start = false;
                }
                // Push the tight status and current blockquote depth for this list
                tight_list_stack.push((*tight, *blockquote_depth));
                match kind {
                    block::ListKind::Unordered => writer.ul_start(),
                    block::ListKind::Ordered { start, .. } => {
                        writer.ol_start(if *start == 1 { None } else { Some(*start) })
                    }
                }
            }
            BlockEvent::ListEnd { kind, .. } => {
                match kind {
                    block::ListKind::Unordered => writer.ul_end(),
                    block::ListKind::Ordered { .. } => writer.ol_end(),
                }
                // Pop the tight status for this list
                tight_list_stack.pop();
            }
            BlockEvent::ListItemStart { task } => {
                writer.li_start();
                // In loose lists, defer newline until content appears (for empty items)
                if !in_tight_list {
                    *pending_loose_li_newline = true;
                } else {
                    // In tight lists, mark that we may need newline if block content follows
                    *at_tight_li_start = true;
                }
                // Store task state for rendering at the start of paragraph content
                if options.task_lists {
                    *pending_task = *task;
                }
            }
            BlockEvent::ListItemEnd => {
                *at_tight_li_start = false;
                *need_newline_before_block = false;
                *pending_loose_li_newline = false;
                *pending_task = block::TaskState::None;
                writer.li_end();
            }

            BlockEvent::DefinitionListStart => {
                if *pending_loose_li_newline {
                    writer.newline();
                    *pending_loose_li_newline = false;
                }
                if *need_newline_before_block {
                    writer.newline();
                    *need_newline_before_block = false;
                }
                if *at_tight_li_start {
                    writer.newline();
                    *at_tight_li_start = false;
                }
                writer.dl_start();
            }
            BlockEvent::DefinitionListEnd => {
                writer.dl_end();
            }
            BlockEvent::DefinitionTermStart => {
                writer.dt_start();
            }
            BlockEvent::DefinitionTermEnd => {
                writer.dt_end();
            }
            BlockEvent::DefinitionDescriptionStart { tight } => {
                writer.dd_start(*tight);
                definition_description_stack.push(*tight);
            }
            BlockEvent::DefinitionDescriptionEnd => {
                definition_description_stack.pop();
                *need_newline_before_block = false;
                writer.dd_end();
            }

            // --- Table events ---
            BlockEvent::TableStart => {
                if *pending_loose_li_newline {
                    writer.newline();
                    *pending_loose_li_newline = false;
                }
                if *need_newline_before_block {
                    writer.newline();
                    *need_newline_before_block = false;
                }
                if *at_tight_li_start {
                    writer.newline();
                    *at_tight_li_start = false;
                }
                writer.table_start();
            }
            BlockEvent::TableColumnWidthsStart => {
                writer.colgroup_start();
            }
            BlockEvent::TableColumnWidth { basis_points } => {
                writer.col_width(*basis_points);
            }
            BlockEvent::TableColumnWidthsEnd => {
                writer.colgroup_end();
            }
            BlockEvent::TableEnd => {
                writer.table_end();
            }
            BlockEvent::TableHeadStart => {
                *in_table_head = true;
                writer.thead_start();
            }
            BlockEvent::TableHeadEnd => {
                *in_table_head = false;
                writer.thead_end();
            }
            BlockEvent::TableBodyStart => {
                writer.tbody_start();
            }
            BlockEvent::TableBodyEnd => {
                writer.tbody_end();
            }
            BlockEvent::TableRowStart => {
                writer.tr_start();
            }
            BlockEvent::TableRowEnd => {
                writer.tr_end();
            }
            BlockEvent::TableCellStart { alignment, colspan } => {
                if *in_table_head {
                    writer.th_start(*alignment, *colspan);
                } else {
                    writer.td_start(*alignment, *colspan);
                }
                cell_state.start();
            }
            BlockEvent::TableCellEnd => {
                let content = cell_state.finish(input);
                if !content.is_empty() {
                    render_inline_content(
                        content,
                        writer,
                        inline_parser,
                        inline_events,
                        link_refs,
                        footnote_store,
                        footnote_numbers,
                        options,
                    );
                }
                if *in_table_head {
                    writer.th_end();
                } else {
                    writer.td_end();
                }
            }
        }
    }
}

/// Emit a pending task checkbox and reset the state.
#[inline]
fn emit_pending_task_checkbox(pending_task: &mut block::TaskState, writer: &mut HtmlWriter) {
    match *pending_task {
        block::TaskState::Unchecked => {
            writer.write_bytes(b"<input type=\"checkbox\" disabled=\"\" /> ");
        }
        block::TaskState::Checked => {
            writer.write_bytes(b"<input type=\"checkbox\" checked=\"\" disabled=\"\" /> ");
        }
        block::TaskState::None => {}
    }
    *pending_task = block::TaskState::None;
}

fn write_footnote_backref(writer: &mut HtmlWriter, label: &str, number: usize) {
    writer.write_str(" <a href=\"#user-content-fnref-");
    writer.write_string(label);
    writer.write_str("\" class=\"data-footnote-backref\" aria-label=\"Back to reference ");
    writer.write_string(&number.to_string());
    writer.write_str("\">↩</a>");
}

fn write_inline_footnote_backref(writer: &mut HtmlWriter, definition_index: usize, number: usize) {
    writer.write_str(" <a href=\"#user-content-inline-fnref-");
    writer.write_string(&(definition_index + 1).to_string());
    writer.write_str("\" class=\"data-footnote-backref\" aria-label=\"Back to reference ");
    writer.write_string(&number.to_string());
    writer.write_str("\">↩</a>");
}

/// State for tracking image rendering.
/// Since we need to render: <img src="..." alt="ALT_TEXT_HERE" title="..." />
/// But alt text comes as Text events between ImageStart and ImageEnd,
/// we need to track:
/// 1. The title to render at ImageEnd
/// 2. The nesting depth (to handle nested images like ![foo ![bar](url1)](url2))
struct ImageState {
    title_range: Option<Range>,
    title_bytes: Option<Vec<u8>>,
    /// Nesting depth: 1 = in outermost image, 2+ = in nested image
    depth: u32,
}

/// First-reference ordering plus constant-time definition-to-ordinal lookup.
struct FootnoteNumbers {
    order: Vec<FootnoteTarget>,
    /// Zero means unassigned; stored ordinals are one-based.
    reference_ordinals: Vec<usize>,
    inline_definitions: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FootnoteTarget {
    Reference(usize),
    Inline(usize),
}

impl FootnoteNumbers {
    fn new(definition_count: usize) -> Self {
        Self {
            order: Vec::new(),
            reference_ordinals: vec![0; definition_count],
            inline_definitions: Vec::new(),
        }
    }

    fn reset(&mut self, definition_count: usize) {
        self.order.clear();
        self.reference_ordinals.clear();
        self.reference_ordinals.resize(definition_count, 0);
        self.inline_definitions.clear();
    }

    fn number_reference(&mut self, definition_index: usize) -> Option<usize> {
        let ordinal = self.reference_ordinals.get_mut(definition_index)?;
        if *ordinal == 0 {
            self.order.push(FootnoteTarget::Reference(definition_index));
            *ordinal = self.order.len();
        }
        Some(*ordinal)
    }

    fn register_inline(&mut self, content: &[u8]) -> (usize, usize) {
        let definition_index = self.inline_definitions.len();
        self.inline_definitions.push(content.to_vec());
        self.order.push(FootnoteTarget::Inline(definition_index));
        (self.order.len(), definition_index)
    }

    fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}

#[allow(clippy::too_many_arguments)]
fn render_inline_content(
    text: &[u8],
    writer: &mut HtmlWriter,
    inline_parser: &mut InlineParser,
    inline_events: &mut Vec<InlineEvent>,
    link_refs: &LinkRefStore,
    footnote_store: Option<&FootnoteStore>,
    footnote_numbers: &mut FootnoteNumbers,
    options: &Options,
) {
    inline_events.clear();
    inline_events.reserve((text.len() / 8).max(8));
    let refs = options.allow_link_refs.then_some(link_refs);
    inline_parser.parse_with_options_in_document(
        text,
        refs,
        options.allow_html,
        options.strikethrough,
        options.highlight,
        options.superscript,
        options.subscript,
        options.autolink_literals,
        options.math,
        options.inline_footnotes,
        footnote_store,
        inline_events,
    );
    #[cfg(feature = "profiling")]
    profiling::record_inline_events(inline_events, inline_events.capacity());

    let mut image_state = None;
    let link_base = normalized_link_base(options);
    for event in inline_events.iter() {
        render_inline_event(
            text,
            event,
            writer,
            &mut image_state,
            link_refs,
            options.disallowed_raw_html,
            options.render_policy,
            link_base,
            footnote_store,
            footnote_numbers,
        );
    }
}

/// Extract plain text from rendered heading HTML: tags are dropped and
/// entities decoded. ferromark-emitted tags never contain a raw `>` inside
/// attribute values (attributes are entity-escaped), so scanning to the next
/// `>` is exact for generated markup.
fn heading_plain_text(html: &[u8]) -> String {
    let mut text = Vec::with_capacity(html.len());
    let mut i = 0;
    while i < html.len() {
        if html[i] == b'<' {
            // Skip the tag; a `>` inside a quoted attribute value (possible
            // in trusted raw HTML) does not close it.
            let mut quote: Option<u8> = None;
            i += 1;
            while i < html.len() {
                let b = html[i];
                match quote {
                    Some(q) if b == q => quote = None,
                    Some(_) => {}
                    None if b == b'"' || b == b'\'' => quote = Some(b),
                    None if b == b'>' => break,
                    None => {}
                }
                i += 1;
            }
            i += 1;
        } else {
            text.push(html[i]);
            i += 1;
        }
    }
    let text = String::from_utf8(text).unwrap_or_default();
    html_escape::decode_html_entities(&text).into_owned()
}

/// Normalize the configured link base path: trailing slashes are stripped
/// and an empty result (including a bare `"/"`) disables rewriting.
fn normalized_link_base(options: &Options) -> Option<&str> {
    let base = options.link_base_path.as_deref()?.trim_end_matches('/');
    (!base.is_empty()).then_some(base)
}

/// Render a single inline event to HTML.
#[allow(clippy::too_many_arguments)]
fn render_inline_event(
    text: &[u8],
    event: &InlineEvent,
    writer: &mut HtmlWriter,
    image_state: &mut Option<ImageState>,
    link_refs: &LinkRefStore,
    filter_html: bool,
    render_policy: RenderPolicy,
    link_base: Option<&str>,
    footnote_store: Option<&FootnoteStore>,
    footnote_numbers: &mut FootnoteNumbers,
) {
    // Check if we're inside an image (for alt text rendering)
    let in_image = image_state.as_ref().is_some_and(|s| s.depth > 0);

    match event {
        InlineEvent::Text(range) => {
            // In image alt text, we still write the text (escaped for attributes)
            if in_image {
                writer.write_escaped_attr(range.slice(text));
            } else {
                // Decode HTML entities and escape for output
                writer.write_text_with_entities(range.slice(text));
            }
        }
        #[cfg(feature = "mdx")]
        InlineEvent::MdxExpression(range)
        | InlineEvent::MdxJsxOpen(range)
        | InlineEvent::MdxJsxClose(range)
        | InlineEvent::MdxJsxSelfClose(range) => {
            if in_image {
                writer.write_escaped_attr(range.slice(text));
            } else if render_policy == RenderPolicy::Trusted {
                writer.write_bytes(range.slice(text));
            } else {
                writer.write_escaped_text(range.slice(text));
            }
        }
        InlineEvent::Code(range) => {
            // In image alt text, just write the code content as plain text
            if in_image {
                let code_content = range.slice(text);
                for &b in code_content {
                    if b == b'\n' {
                        writer.write_str(" ");
                    } else if b == b'<' {
                        writer.write_str("&lt;");
                    } else if b == b'>' {
                        writer.write_str("&gt;");
                    } else if b == b'&' {
                        writer.write_str("&amp;");
                    } else if b == b'"' {
                        writer.write_str("&quot;");
                    } else {
                        writer.buffer_mut().push(b);
                    }
                }
            } else {
                writer.write_str("<code>");
                // CommonMark: line endings in code spans are converted to spaces
                let code_content = range.slice(text);
                for &b in code_content {
                    if b == b'\n' {
                        writer.write_str(" ");
                    } else if b == b'<' {
                        writer.write_str("&lt;");
                    } else if b == b'>' {
                        writer.write_str("&gt;");
                    } else if b == b'&' {
                        writer.write_str("&amp;");
                    } else if b == b'"' {
                        writer.write_str("&quot;");
                    } else {
                        writer.buffer_mut().push(b);
                    }
                }
                writer.write_str("</code>");
            }
        }
        InlineEvent::EmphasisStart => {
            // Suppress HTML tags inside image alt text
            if !in_image {
                writer.write_str("<em>");
            }
        }
        InlineEvent::EmphasisEnd => {
            if !in_image {
                writer.write_str("</em>");
            }
        }
        InlineEvent::StrongStart => {
            if !in_image {
                writer.write_str("<strong>");
            }
        }
        InlineEvent::StrongEnd => {
            if !in_image {
                writer.write_str("</strong>");
            }
        }
        InlineEvent::StrikethroughStart => {
            if !in_image {
                writer.write_str("<del>");
            }
        }
        InlineEvent::StrikethroughEnd => {
            if !in_image {
                writer.write_str("</del>");
            }
        }
        InlineEvent::SubscriptStart => {
            if !in_image {
                writer.write_str("<sub>");
            }
        }
        InlineEvent::SubscriptEnd => {
            if !in_image {
                writer.write_str("</sub>");
            }
        }
        InlineEvent::SuperscriptStart => {
            if !in_image {
                writer.write_str("<sup>");
            }
        }
        InlineEvent::SuperscriptEnd => {
            if !in_image {
                writer.write_str("</sup>");
            }
        }
        InlineEvent::HighlightStart => {
            if !in_image {
                writer.write_str("<mark>");
            }
        }
        InlineEvent::HighlightEnd => {
            if !in_image {
                writer.write_str("</mark>");
            }
        }
        InlineEvent::LinkStart { url, title } => {
            // Suppress link tags inside image alt text
            if !in_image {
                writer.write_str("<a href=\"");
                writer.write_link_url_with_policy_and_base(
                    url.slice(text),
                    render_policy,
                    link_base,
                );
                writer.write_str("\"");
                if let Some(t) = title {
                    writer.write_str(" title=\"");
                    writer.write_link_title(t.slice(text));
                    writer.write_str("\"");
                }
                writer.write_str(">");
            }
        }
        InlineEvent::LinkStartRef { def_index } => {
            if !in_image && let Some(def) = link_refs.get(*def_index as usize) {
                writer.write_str("<a href=\"");
                writer.write_link_url_with_policy_and_base(&def.url, render_policy, link_base);
                writer.write_str("\"");
                if let Some(title) = &def.title {
                    writer.write_str(" title=\"");
                    writer.write_link_title(title);
                    writer.write_str("\"");
                }
                writer.write_str(">");
            }
        }
        InlineEvent::LinkEnd => {
            if !in_image {
                writer.write_str("</a>");
            }
        }
        InlineEvent::ImageStart { url, title } => {
            // If we're already inside an image, just increment depth
            // (the inner image's alt text becomes plain text in outer alt)
            if let Some(state) = image_state.as_mut() {
                state.depth += 1;
            } else {
                // Outermost image - emit the img tag start
                writer.write_str("<img src=\"");
                writer.write_link_url_with_policy(url.slice(text), render_policy);
                writer.write_str("\" alt=\"");
                *image_state = Some(ImageState {
                    title_range: *title,
                    title_bytes: None,
                    depth: 1,
                });
            }
        }
        InlineEvent::ImageStartRef { def_index } => {
            if let Some(state) = image_state.as_mut() {
                state.depth += 1;
            } else if let Some(def) = link_refs.get(*def_index as usize) {
                writer.write_str("<img src=\"");
                writer.write_link_url_with_policy(&def.url, render_policy);
                writer.write_str("\" alt=\"");
                *image_state = Some(ImageState {
                    title_range: None,
                    title_bytes: def.title.clone(),
                    depth: 1,
                });
            }
        }
        InlineEvent::ImageEnd => {
            if let Some(state) = image_state.as_mut() {
                state.depth -= 1;
                // Only close when we exit the outermost image
                if state.depth == 0 {
                    writer.write_str("\"");
                    // Add title attribute if present
                    let title_range = state.title_range;
                    let title_bytes = state.title_bytes.clone();
                    *image_state = None;
                    if let Some(bytes) = title_bytes {
                        writer.write_str(" title=\"");
                        writer.write_link_title(&bytes);
                        writer.write_str("\"");
                    } else if let Some(title_range) = title_range {
                        writer.write_str(" title=\"");
                        writer.write_link_title(title_range.slice(text));
                        writer.write_str("\"");
                    }
                    writer.write_str(" />");
                }
            }
        }
        InlineEvent::AutolinkLiteral { url, kind } => {
            use crate::inline::AutolinkLiteralKind;
            if in_image {
                writer.write_escaped_attr(url.slice(text));
            } else {
                writer.write_str("<a href=\"");
                match kind {
                    AutolinkLiteralKind::Url => {
                        writer.write_link_url_with_policy(url.slice(text), render_policy);
                    }
                    AutolinkLiteralKind::Www => {
                        writer.write_str("http://");
                        writer.write_link_url(url.slice(text));
                    }
                    AutolinkLiteralKind::Email => {
                        writer.write_str("mailto:");
                        writer.write_link_url(url.slice(text));
                    }
                }
                writer.write_str("\">");
                writer.write_escaped_text(url.slice(text));
                writer.write_str("</a>");
            }
        }
        InlineEvent::Autolink { url, is_email } => {
            // In image alt text, just output the URL as plain text
            if in_image {
                writer.write_escaped_attr(url.slice(text));
            } else {
                writer.write_str("<a href=\"");
                if *is_email {
                    writer.write_str("mailto:");
                    writer.write_url_encoded(url.slice(text));
                } else {
                    writer.write_url_encoded_with_policy(url.slice(text), render_policy);
                }
                writer.write_str("\">");
                // Display text is shown as-is (with HTML escaping)
                writer.write_escaped_text(url.slice(text));
                writer.write_str("</a>");
            }
        }
        InlineEvent::Html(range) => {
            if in_image {
                writer.write_escaped_attr(range.slice(text));
            } else if render_policy == RenderPolicy::Untrusted {
                writer.write_escaped_text(range.slice(text));
            } else if filter_html {
                writer.write_html_filtered(range.slice(text));
            } else {
                writer.write_bytes(range.slice(text));
            }
        }
        InlineEvent::SoftBreak => {
            // In image alt text, use space instead of newline
            if in_image {
                writer.write_str(" ");
            } else {
                writer.write_str("\n");
            }
        }
        InlineEvent::HardBreak => {
            // In image alt text, use space instead of <br />
            if in_image {
                writer.write_str(" ");
            } else {
                writer.write_str("<br />\n");
            }
        }
        InlineEvent::EscapedChar(ch) => {
            // Write the escaped character (the actual char, not the backslash)
            let bytes = [*ch];
            if in_image {
                writer.write_escaped_attr(&bytes);
            } else {
                writer.write_escaped_text(&bytes);
            }
        }
        InlineEvent::FootnoteRef { def_index } => {
            if !in_image && let Some(fn_store) = footnote_store {
                let def_idx = *def_index as usize;
                if let (Some(number), Some(def)) = (
                    footnote_numbers.number_reference(def_idx),
                    fn_store.get(def_idx),
                ) {
                    writer.write_str("<sup><a href=\"#user-content-fn-");
                    writer.write_string(&def.label);
                    writer.write_str("\" id=\"user-content-fnref-");
                    writer.write_string(&def.label);
                    writer.write_str("\" data-footnote-ref>");
                    let num_str = number.to_string();
                    writer.write_string(&num_str);
                    writer.write_str("</a></sup>");
                }
            }
        }
        InlineEvent::InlineFootnote(range) => {
            if !in_image {
                let (number, definition_index) =
                    footnote_numbers.register_inline(range.slice(text));
                writer.write_str("<sup><a href=\"#user-content-inline-fn-");
                writer.write_string(&(definition_index + 1).to_string());
                writer.write_str("\" id=\"user-content-inline-fnref-");
                writer.write_string(&(definition_index + 1).to_string());
                writer.write_str("\" data-footnote-ref>");
                writer.write_string(&number.to_string());
                writer.write_str("</a></sup>");
            }
        }
        InlineEvent::MathInline(range) => {
            if in_image {
                writer.write_escaped_attr(range.slice(text));
            } else {
                writer.write_str("<code class=\"language-math math-inline\">");
                let content = range.slice(text);
                for &b in content {
                    if b == b'\n' {
                        writer.write_str(" ");
                    } else if b == b'<' {
                        writer.write_str("&lt;");
                    } else if b == b'>' {
                        writer.write_str("&gt;");
                    } else if b == b'&' {
                        writer.write_str("&amp;");
                    } else if b == b'"' {
                        writer.write_str("&quot;");
                    } else {
                        writer.buffer_mut().push(b);
                    }
                }
                writer.write_str("</code>");
            }
        }
        InlineEvent::MathDisplay(range) => {
            if in_image {
                writer.write_escaped_attr(range.slice(text));
            } else {
                writer.write_str("<code class=\"language-math math-display\">");
                let content = range.slice(text);
                for &b in content {
                    if b == b'\n' {
                        writer.write_str(" ");
                    } else if b == b'<' {
                        writer.write_str("&lt;");
                    } else if b == b'>' {
                        writer.write_str("&gt;");
                    } else if b == b'&' {
                        writer.write_str("&amp;");
                    } else if b == b'"' {
                        writer.write_str("&quot;");
                    } else {
                        writer.buffer_mut().push(b);
                    }
                }
                writer.write_str("</code>");
            }
        }
    }
}

impl<R: FencedCodeRenderer + ?Sized> RenderContext<'_, '_, R> {
    /// Render collected footnotes with a fresh block state per definition.
    fn render_footnote_section(&mut self, input: &[u8]) {
        let footnote_store = self.footnote_store;
        let order = self.footnote_numbers.order.clone();
        self.writer
            .write_str("<section data-footnotes class=\"footnotes\">\n<ol>\n");

        for (seq_num, target) in order.into_iter().enumerate() {
            let number = seq_num + 1;
            match target {
                FootnoteTarget::Reference(def_idx) => {
                    let Some(footnote_store) = footnote_store else {
                        continue;
                    };
                    let Some(def) = footnote_store.get(def_idx) else {
                        continue;
                    };
                    self.writer.write_str("<li id=\"user-content-fn-");
                    self.writer.write_string(&def.label);
                    self.writer.write_str("\">\n");

                    let last_paragraph_end = def
                        .events
                        .iter()
                        .rposition(|event| matches!(event, BlockEvent::ParagraphEnd));
                    let renderer = self.fenced_code_renderer.as_deref_mut();
                    let nested_options = Options {
                        inline_footnotes: false,
                        ..self.options.clone()
                    };
                    let mut nested_state = RenderState::new(&nested_options);
                    let mut nested = RenderContext::new(
                        &mut *self.writer,
                        &mut *self.inline_parser,
                        &mut *self.inline_events,
                        &mut nested_state,
                        self.link_refs,
                        Some(footnote_store),
                        &nested_options,
                        renderer,
                        None,
                    );
                    for (index, event) in def.events.iter().enumerate() {
                        if Some(index) == last_paragraph_end {
                            *nested.pending_footnote_backref = Some((def.label.clone(), number));
                        }
                        nested.render_block_event(input, event);
                    }
                }
                FootnoteTarget::Inline(definition_index) => {
                    let Some(content) = self
                        .footnote_numbers
                        .inline_definitions
                        .get(definition_index)
                        .cloned()
                    else {
                        continue;
                    };
                    self.writer.write_str("<li id=\"user-content-inline-fn-");
                    self.writer
                        .write_string(&(definition_index + 1).to_string());
                    self.writer.write_str("\">\n<p>");

                    let mut nested_numbers = FootnoteNumbers::new(0);
                    let nested_options = Options {
                        footnotes: false,
                        inline_footnotes: false,
                        ..self.options.clone()
                    };
                    // Reuse the parent's idle inline parser and event buffer
                    // instead of allocating fresh ones per inline footnote.
                    render_inline_content(
                        &content,
                        &mut *self.writer,
                        &mut *self.inline_parser,
                        &mut *self.inline_events,
                        self.link_refs,
                        None,
                        &mut nested_numbers,
                        &nested_options,
                    );
                    write_inline_footnote_backref(&mut *self.writer, definition_index, number);
                    self.writer.write_str("</p>\n");
                }
            }

            self.writer.write_str("</li>\n");
        }

        self.writer.write_str("</ol>\n</section>\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footnote_numbers_assign_constant_time_stable_ordinals() {
        let mut numbers = FootnoteNumbers::new(4);

        assert_eq!(numbers.number_reference(2), Some(1));
        assert_eq!(numbers.number_reference(0), Some(2));
        assert_eq!(numbers.number_reference(2), Some(1));
        assert_eq!(numbers.number_reference(3), Some(3));
        assert_eq!(numbers.number_reference(4), None);
        assert_eq!(
            numbers.order,
            vec![
                FootnoteTarget::Reference(2),
                FootnoteTarget::Reference(0),
                FootnoteTarget::Reference(3)
            ]
        );
    }

    #[test]
    fn test_basic_paragraph() {
        let html = to_html("Hello, world!");
        assert_eq!(html, "<p>Hello, world!</p>\n");
    }

    #[test]
    fn test_paragraph_escaping() {
        let html = to_html("<script>alert('xss')</script>");
        assert_eq!(html, "&lt;script&gt;alert('xss')&lt;/script&gt;");
    }

    #[test]
    fn test_heading_h1() {
        let html = to_html("# Hello");
        assert!(html.contains("Hello</h1>"));
    }

    #[test]
    fn test_heading_h2() {
        let html = to_html("## World");
        assert!(html.contains("World</h2>"));
    }

    #[test]
    fn test_heading_all_levels() {
        for level in 1..=6 {
            let input = format!("{} Heading", "#".repeat(level));
            let html = to_html(&input);
            assert!(
                html.contains(&format!("Heading</h{level}>")),
                "Failed for level {level}: {html}"
            );
        }
    }

    #[test]
    fn test_thematic_break() {
        let html = to_html("---");
        assert_eq!(html, "<hr />\n");
    }

    #[test]
    fn test_thematic_break_variants() {
        assert_eq!(to_html("---"), "<hr />\n");
        assert_eq!(to_html("***"), "<hr />\n");
        assert_eq!(to_html("___"), "<hr />\n");
        assert_eq!(to_html("- - -"), "<hr />\n");
        assert_eq!(to_html("----------"), "<hr />\n");
    }

    #[test]
    fn test_multiple_paragraphs() {
        let html = to_html("First\n\nSecond");
        assert!(html.contains("<p>First</p>"));
        assert!(html.contains("<p>Second</p>"));
    }

    #[test]
    fn test_heading_and_paragraph() {
        let html = to_html("# Title\n\nContent here.");
        assert!(html.contains("Title</h1>"));
        assert!(html.contains("<p>Content here.</p>"));
    }

    #[test]
    fn test_heading_with_closing_hashes() {
        let html = to_html("# Hello #");
        assert!(html.contains("Hello</h1>"));
    }

    #[test]
    fn test_complex_document() {
        let input = r#"# Main Title

This is the first paragraph.

## Section 1

More content here.

---

## Section 2

Final paragraph."#;

        let html = to_html(input);

        assert!(html.contains("Main Title</h1>"));
        assert!(html.contains("Section 1</h2>"));
        assert!(html.contains("Section 2</h2>"));
        assert!(html.contains("<hr />"));
        assert!(html.contains("<p>This is the first paragraph.</p>"));
    }

    #[test]
    fn test_multiline_paragraph() {
        let html = to_html("Line 1\nLine 2\nLine 3");
        // All lines should be in the same paragraph
        assert!(html.starts_with("<p>"));
        assert!(html.contains("Line 1"));
        assert!(html.contains("Line 2"));
        assert!(html.contains("Line 3"));
        assert!(html.ends_with("</p>\n"));
    }

    #[test]
    fn test_empty_input() {
        let html = to_html("");
        assert_eq!(html, "");
    }

    #[test]
    fn test_only_whitespace() {
        let html = to_html("   \n\n   ");
        assert_eq!(html, "");
    }

    #[test]
    fn test_to_html_into() {
        let mut buffer = Vec::new();
        to_html_into("# Test", &mut buffer);
        let html = String::from_utf8(buffer).unwrap();
        assert!(html.contains("Test</h1>"));
    }

    // Code block tests

    #[test]
    fn test_code_block_basic() {
        let html = to_html("```\ncode\n```");
        assert!(html.contains("<pre><code>"));
        assert!(html.contains("code"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn test_code_block_with_language() {
        let html = to_html("```rust\nfn main() {}\n```");
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn test_code_block_escapes_html() {
        let html = to_html("```\n<script>alert('xss')</script>\n```");
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_code_block_multiline() {
        let html = to_html("```\nline1\nline2\n```");
        assert!(html.contains("line1"));
        assert!(html.contains("line2"));
    }

    #[test]
    fn test_code_block_in_document() {
        let input = r#"# Title

Some text.

```python
print("hello")
```

More text."#;
        let html = to_html(input);
        assert!(html.contains("Title</h1>"));
        assert!(html.contains("<p>Some text.</p>"));
        assert!(html.contains("<pre><code class=\"language-python\">"));
        assert!(html.contains("print"));
        assert!(html.contains("<p>More text.</p>"));
    }

    // Tight/loose list tests

    #[test]
    fn test_tight_list_unordered() {
        let html = to_html("- foo\n- bar\n- baz");
        // Tight list: no <p> tags inside list items
        assert!(html.contains("<li>foo</li>"));
        assert!(html.contains("<li>bar</li>"));
        assert!(html.contains("<li>baz</li>"));
        assert!(!html.contains("<li><p>"));
    }

    #[test]
    fn test_loose_list_unordered() {
        let html = to_html("- foo\n\n- bar\n\n- baz");
        // Loose list: <p> tags inside list items (with newline after <li>)
        assert!(html.contains("<li>\n<p>foo</p>"));
        assert!(html.contains("<li>\n<p>bar</p>"));
        assert!(html.contains("<li>\n<p>baz</p>"));
    }

    #[test]
    fn test_tight_list_ordered() {
        let html = to_html("1. first\n2. second\n3. third");
        // Tight list: no <p> tags
        assert!(html.contains("<li>first</li>"));
        assert!(html.contains("<li>second</li>"));
        assert!(html.contains("<li>third</li>"));
        assert!(!html.contains("<li><p>"));
    }

    #[test]
    fn test_loose_list_ordered() {
        let html = to_html("1. first\n\n2. second");
        // Loose list: <p> tags (with newline after <li>)
        assert!(html.contains("<li>\n<p>first</p>"));
        assert!(html.contains("<li>\n<p>second</p>"));
    }

    // Image tests

    #[test]
    fn test_image_basic() {
        let html = to_html("![alt](image.png)");
        // Should have img tag with src and alt
        assert!(html.contains("<img src=\"image.png\""), "Missing img src");
        assert!(html.contains("alt=\"alt\""), "Missing alt attribute");
        // Should NOT have standalone ! before the img tag
        assert!(!html.contains("!<img"), "Stray ! before img tag");
    }

    #[test]
    fn test_image_with_title() {
        let html = to_html("![alt](image.png \"title\")");
        // Should have img tag with title
        assert!(html.contains("<img"), "No img tag found");
        assert!(html.contains("title=\"title\""), "Missing title attribute");
        assert!(!html.contains("!<img"), "Stray ! before img tag");
    }

    #[test]
    fn test_image_in_text() {
        let html = to_html("text before ![img](url) text after");
        // Image should be between text
        assert!(html.contains("text before"));
        assert!(html.contains("<img src=\"url\""));
        assert!(html.contains("text after"));
    }

    #[test]
    fn test_image_with_nested_emphasis() {
        // CommonMark: alt text should be plain text, not HTML
        let html = to_html("![foo *bar*](/url)");
        // Should have alt="foo bar" (plain text, no <em> tags)
        assert!(
            html.contains("alt=\"foo bar\""),
            "Alt text should be plain: {html}"
        );
        assert!(!html.contains("<em>"), "No <em> tags in alt text");
    }

    #[test]
    fn test_image_with_nested_strong() {
        let html = to_html("![foo **bar**](/url)");
        // Should have alt="foo bar" (plain text, no <strong> tags)
        assert!(
            html.contains("alt=\"foo bar\""),
            "Alt text should be plain: {html}"
        );
        assert!(!html.contains("<strong>"), "No <strong> tags in alt text");
    }
}

#[cfg(test)]
mod crate_docs_tests {
    use super::{Options, RenderPolicy};

    fn documented_default_policy(crate_docs: &str) -> RenderPolicy {
        const UNTRUSTED_DEFAULT: &str = "Its default\n//! [`RenderPolicy::Untrusted`] escapes raw HTML and restricts unsafe URL";
        const TRUSTED_DEFAULT: &str =
            "Its default\n//! [`RenderPolicy::Trusted`] preserves raw HTML and arbitrary URL";

        match (
            crate_docs.contains(UNTRUSTED_DEFAULT),
            crate_docs.contains(TRUSTED_DEFAULT),
        ) {
            (true, false) => RenderPolicy::Untrusted,
            (false, true) => RenderPolicy::Trusted,
            _ => panic!("crate docs must identify exactly one default render policy"),
        }
    }

    #[test]
    fn crate_docs_describe_current_security_feature_and_simd_contracts() {
        let source = include_str!("lib.rs").replace("\r\n", "\n");
        let crate_docs = source
            .split_once("pub mod block;")
            .expect("crate docs must precede the public module declarations")
            .0;
        let cargo_toml = include_str!("../Cargo.toml");
        let simd_source = include_str!("inline/simd.rs").replace("\r\n", "\n");

        assert_eq!(
            documented_default_policy(crate_docs),
            Options::default().render_policy,
            "crate docs must describe the policy used by default public render entry points",
        );
        assert!(crate_docs.contains("`mdx` Cargo feature"));
        assert!(crate_docs.contains("AArch64 builds with NEON enabled"));
        assert!(crate_docs.contains("x86-64"));
        assert!(!crate_docs.contains("targeting 20-30% better throughput"));
        assert!(!crate_docs.contains("Future Optimizations"));
        assert!(!crate_docs.contains("NEON intrinsics for ARM"));

        assert!(cargo_toml.contains("mdx = []"));
        assert!(simd_source.contains(
            r#"#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[target_feature(enable = "neon")]
pub unsafe fn has_inline_specials_simd"#,
        ));
        assert!(simd_source.contains(
            r#"#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn has_inline_specials_simd"#,
        ));
        assert!(simd_source.contains(
            r#"#[cfg(not(any(
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_feature = "neon")
)))]
#[allow(dead_code)]
pub fn has_inline_specials_simd"#,
        ));
    }
}

#[cfg(test)]
mod entity_tests {
    #[test]
    fn test_html_escape_entities() {
        use html_escape::decode_html_entities;

        assert_eq!(decode_html_entities("&auml;").as_ref(), "ä");
        assert_eq!(decode_html_entities("&#228;").as_ref(), "ä");
        assert_eq!(decode_html_entities("&#xE4;").as_ref(), "ä");
        assert_eq!(decode_html_entities("&amp;").as_ref(), "&");
        assert_eq!(decode_html_entities("foo%20b&auml;").as_ref(), "foo%20bä");
    }
}
