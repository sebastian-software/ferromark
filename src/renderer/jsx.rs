//! Native Markdown/MDX output as JSX source.
//!
//! Ferromark writes the JSX tree and leaves module wrappers and compilation to
//! its caller. Text is represented as JavaScript string children, MDX
//! expressions and ESM are preserved as source, and no framework runtime is
//! imported or invoked here.

mod code_metadata;
#[cfg(feature = "ferriki")]
mod ferriki_integration;
mod render;

#[cfg(feature = "ferriki")]
pub use ferriki_integration::FerrikiJsxHooks;

use std::collections::BTreeMap;

use crate::ast::{Document, Span};
use crate::outline::OutlineEntry;

use super::{HtmlRenderer, InvalidHeadingIdPrefix};

/// Options for native JSX rendering.
#[derive(Debug, Clone)]
pub struct JsxRendererOptions {
    /// Optional JSX member prefix for generated Markdown elements.
    ///
    /// With `_components`, a Markdown paragraph becomes
    /// `<_components.p>…</_components.p>`. Authored MDX components retain the
    /// names written in the document. The prefix must be a valid JSX member
    /// expression; Ferromark writes it verbatim.
    pub component_prefix: Option<String>,

    /// Generate native heading IDs from the heading text.
    ///
    /// Default: `true`.
    pub heading_ids: bool,

    /// Shift heading levels before clamping them to `h1`–`h6`.
    ///
    /// Default: `0`.
    pub heading_level_offset: i32,

    /// Prefix generated and authored heading IDs.
    ///
    /// The prefix follows the same validation rules as
    /// [`HtmlRenderer::try_with_heading_id_prefix`].
    pub heading_id_prefix: String,

    /// Render GitHub-style callout block quotes with their title treatment.
    ///
    /// Default: `true`.
    pub callouts: bool,

    /// Omit the first top-level H1 when its visible text matches this title.
    ///
    /// Comparison trims surrounding whitespace but remains case-sensitive.
    /// This supports caller-owned frontmatter title handling; Ferromark does
    /// not parse or infer the title itself.
    pub omit_title_heading: Option<String>,

    /// Map callout kinds (`note`, `tip`, `important`, `warning`, `caution`) to
    /// authored JSX component names. A mapped component receives `kind` and
    /// `title` props and the callout body as children.
    pub callout_components: BTreeMap<String, String>,

    /// Map fenced code languages to authored JSX component names. A mapped
    /// component receives `language` and optional `meta` props and the code as
    /// a string child.
    pub code_block_components: BTreeMap<String, String>,

    /// Wrap rendered `<pre><code>` output in this authored component when no
    /// render override or language-specific component handles the block.
    pub code_block_component: Option<String>,

    /// Show code line numbers by default. Fence metadata can override this.
    ///
    /// Default: `false`.
    pub show_line_numbers: bool,
}

impl Default for JsxRendererOptions {
    fn default() -> Self {
        Self {
            component_prefix: None,
            heading_ids: true,
            heading_level_offset: 0,
            heading_id_prefix: String::new(),
            callouts: true,
            omit_title_heading: None,
            callout_components: BTreeMap::new(),
            code_block_components: BTreeMap::new(),
            code_block_component: None,
            show_line_numbers: false,
        }
    }
}

impl JsxRendererOptions {
    /// Returns these options with a validated prefix for heading IDs.
    pub fn try_with_heading_id_prefix(
        mut self,
        prefix: impl Into<String>,
    ) -> Result<Self, InvalidHeadingIdPrefix> {
        let prefix = prefix.into();
        HtmlRenderer::validate_heading_id_prefix(&prefix)?;
        self.heading_id_prefix = prefix;
        Ok(self)
    }

    /// Returns these options with a heading level offset.
    #[must_use]
    pub fn with_heading_level_offset(mut self, offset: i32) -> Self {
        self.heading_level_offset = offset;
        self
    }

    /// Maps one callout kind to an authored component name.
    #[must_use]
    pub fn with_callout_component(
        mut self,
        kind: impl Into<String>,
        component: impl Into<String>,
    ) -> Self {
        self.callout_components
            .insert(kind.into().to_ascii_lowercase(), component.into());
        self
    }

    /// Maps one fenced code language to an authored component name.
    #[must_use]
    pub fn with_code_block_component(
        mut self,
        language: impl Into<String>,
        component: impl Into<String>,
    ) -> Self {
        self.code_block_components
            .insert(language.into(), component.into());
        self
    }
}

/// Code block data passed to a synchronous JSX render hook.
#[derive(Debug, Clone, Copy)]
pub struct JsxCodeBlockInput<'a> {
    /// Source code inside the fence.
    pub code: &'a str,
    /// Optional first info-string token. The code hook receives it as authored;
    /// the highlighter hook receives the normalized bare language token.
    pub language: Option<&'a str>,
    /// Optional remaining fence metadata.
    pub meta: Option<&'a str>,
    /// Original source span of the complete block.
    pub span: Span,
}

/// A synchronous override for fenced code rendering.
pub trait JsxRenderHooks {
    /// Returns trusted JSX source for a block, or `None` to use the default
    /// code block or configured language component.
    ///
    /// This callback receives the original fence language and metadata.
    fn render_code_block(&mut self, _input: JsxCodeBlockInput<'_>) -> Option<String> {
        None
    }

    /// Supplies trusted, tag-balanced HTML for one fragment per code line.
    ///
    /// Ferromark converts these fragments to JSX children, including JSX-safe
    /// attribute names and string values. Implementations must escape source
    /// text and must not include line breaks or cross-line tags. Returning
    /// `None` uses escaped source code. The language on this input is the
    /// normalized bare language token, unlike the raw value offered to
    /// [`render_code_block`](Self::render_code_block).
    fn highlight_code_block(
        &mut self,
        _input: JsxCodeBlockInput<'_>,
    ) -> Option<JsxHighlightedCodeBlock> {
        None
    }
}

/// Trusted highlighted HTML fragments and optional theme colors for JSX.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct JsxHighlightedCodeBlock {
    /// One balanced inner-HTML fragment per source line, including a final
    /// empty line when the code ends with a newline.
    pub lines: Vec<String>,
    /// Light or single-theme foreground color for the outer code block.
    pub foreground: Option<String>,
    /// Light or single-theme background color for the outer code block.
    pub background: Option<String>,
    /// Optional dark-theme foreground for CSS variable output.
    pub dark_foreground: Option<String>,
    /// Optional dark-theme background for CSS variable output.
    pub dark_background: Option<String>,
}

impl JsxHighlightedCodeBlock {
    /// Creates highlighted lines without block-level theme colors.
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            foreground: None,
            background: None,
            dark_foreground: None,
            dark_background: None,
        }
    }

    /// Sets the foreground and background for a single theme.
    #[must_use]
    pub fn with_colors(
        mut self,
        foreground: impl Into<String>,
        background: impl Into<String>,
    ) -> Self {
        self.foreground = Some(foreground.into());
        self.background = Some(background.into());
        self
    }

    /// Sets both light and dark theme colors for the block and token fragments.
    #[must_use]
    pub fn with_light_dark_colors(
        mut self,
        light_foreground: impl Into<String>,
        light_background: impl Into<String>,
        dark_foreground: impl Into<String>,
        dark_background: impl Into<String>,
    ) -> Self {
        self.foreground = Some(light_foreground.into());
        self.background = Some(light_background.into());
        self.dark_foreground = Some(dark_foreground.into());
        self.dark_background = Some(dark_background.into());
        self
    }
}

/// A no-op code block hook.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoJsxRenderHooks;

impl JsxRenderHooks for NoJsxRenderHooks {}

/// An ordered ESM source snippet extracted from a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsxModuleSource {
    /// Original import or export source.
    pub value: String,
    /// Original source span.
    pub span: Span,
}

/// Metadata for one encountered Markdown code block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsxCodeBlock {
    /// Original code content.
    pub value: String,
    /// Optional first info-string token.
    pub language: Option<String>,
    /// Optional remaining fence metadata.
    pub meta: Option<String>,
    /// Original source span of the complete block.
    pub span: Span,
    /// Configured component name used for this block, if any.
    pub component: Option<String>,
}

/// A source location in generated JSX and its Markdown/MDX source.
///
/// Lines and columns are zero-based. Columns count UTF-16 code units, matching
/// JavaScript tooling conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsxSourceMapping {
    /// Line in `JsxOutput::body`.
    pub generated_line: u32,
    /// UTF-16 column in `JsxOutput::body`.
    pub generated_column: u32,
    /// Line in the source passed to [`JsxRenderer::render`].
    pub source_line: u32,
    /// UTF-16 column in the source passed to [`JsxRenderer::render`].
    pub source_column: u32,
}

/// Structured output from a native JSX render.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsxOutput {
    /// JSX for the document body. Module-level ESM is returned separately.
    pub body: String,
    /// Module imports and exports in document order.
    pub esm: Vec<JsxModuleSource>,
    /// Root identifiers referenced by authored MDX JSX components, in first
    /// reference order. Configured components emitted by callout and code
    /// mappings are included; the generated component prefix is not.
    pub components: Vec<String>,
    /// Intrinsic element names emitted for Markdown constructs, in first
    /// reference order. This list is unchanged by `component_prefix`.
    pub elements: Vec<String>,
    /// Headings emitted in the JSX body, with text and IDs following the
    /// native JSX tree's visible children.
    pub headings: Vec<OutlineEntry>,
    /// Source span of the omitted title heading, when configured.
    pub omitted_title_heading: Option<Span>,
    /// Code blocks encountered in source order.
    pub code_blocks: Vec<JsxCodeBlock>,
    /// Node-start source locations in the generated body.
    pub mappings: Vec<JsxSourceMapping>,
}

/// Converts a parsed Markdown/MDX document into framework-neutral JSX source.
#[derive(Debug, Clone, Default)]
pub struct JsxRenderer {
    options: JsxRendererOptions,
}

impl JsxRenderer {
    /// Creates a JSX renderer with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a JSX renderer with explicit options.
    #[must_use]
    pub fn with_options(options: JsxRendererOptions) -> Self {
        Self { options }
    }

    /// Returns the options used by this renderer.
    #[must_use]
    pub fn options(&self) -> &JsxRendererOptions {
        &self.options
    }

    /// Renders a document, converting AST byte spans to source coordinates
    /// with the same source string used to parse the document.
    #[must_use]
    pub fn render(&self, document: &Document<'_>, source: &str) -> JsxOutput {
        self.render_with_hooks(document, source, &mut NoJsxRenderHooks)
    }

    /// Renders a document with a synchronous code block hook.
    #[must_use]
    pub fn render_with_hooks<H: JsxRenderHooks>(
        &self,
        document: &Document<'_>,
        source: &str,
        hooks: &mut H,
    ) -> JsxOutput {
        render::render_document(document, source, &self.options, hooks)
    }
}
