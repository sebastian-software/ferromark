//! Native Markdown/MDX output as JSX source.
//!
//! Ferromark writes the JSX tree and leaves compilation to its caller. Text is
//! represented as JavaScript string children, MDX expressions and ESM are
//! preserved as source, and no framework runtime is imported or invoked here.
//! With the `jsx` feature, the tree can also be wrapped in an MDX module.

mod code_metadata;
#[cfg(feature = "ferriki")]
mod ferriki_integration;
mod line_index;
#[cfg(feature = "jsx")]
mod module;
mod render;

#[cfg(test)]
mod tests;

#[cfg(feature = "jsx")]
pub use module::{JsxModuleError, JsxModuleOptions, JsxModuleOutput, JsxSourceMap};

#[cfg(feature = "ferriki")]
pub use ferriki_integration::FerrikiJsxHooks;

use std::collections::BTreeMap;

use crate::allocator::Allocator;
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

/// Standalone JSX markup and parsed fence metadata for one code block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsxCodeBlockRenderOutput {
    /// Intrinsic `<pre><code>` JSX, including the trailing newline used in a document body.
    pub jsx: String,
    /// Trimmed language identifier without a recognized metadata suffix; case is preserved.
    pub language: Option<String>,
    /// Parsed title metadata, if present.
    pub title: Option<String>,
    /// Parsed bracket label, if present.
    pub label: Option<String>,
    /// Whether line numbers are enabled after applying fence metadata.
    pub line_numbers: bool,
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

    /// Renders one standalone code block as intrinsic JSX and returns its parsed metadata.
    ///
    /// The output uses the same metadata parser, code markup, and line-number
    /// default as a fenced block in a document. It always emits plain
    /// `<pre>`, `<code>`, and `<span>` elements: component prefixes,
    /// language component mappings, whole-fence hooks, and the global code
    /// block component do not apply. NUL becomes U+FFFD and CR or CRLF line
    /// endings become LF, matching the Markdown parser's fenced-code behavior.
    #[must_use]
    pub fn render_code_block(
        &self,
        code: &str,
        language: Option<&str>,
        meta: Option<&str>,
    ) -> JsxCodeBlockRenderOutput {
        self.render_code_block_with_hooks(code, language, meta, &mut NoJsxRenderHooks)
    }

    /// Renders one standalone code block with the configured highlight hook.
    ///
    /// This method uses `highlight_code_block` when provided. The whole-block
    /// `render_code_block` override is intentionally not invoked, because its
    /// replacement need not be the intrinsic markup returned by this method.
    #[must_use]
    pub fn render_code_block_with_hooks<H: JsxRenderHooks>(
        &self,
        code: &str,
        language: Option<&str>,
        meta: Option<&str>,
        hooks: &mut H,
    ) -> JsxCodeBlockRenderOutput {
        let allocator = Allocator::new();
        let code_bytes = code.as_bytes();
        let normalized_code = if code_bytes.iter().any(|&byte| matches!(byte, b'\r' | 0)) {
            crate::parser::normalize_code_block_content(&allocator, code)
        } else {
            code
        };
        let options = JsxRendererOptions {
            component_prefix: None,
            code_block_components: BTreeMap::new(),
            code_block_component: None,
            ..self.options.clone()
        };
        render::render_code_block(normalized_code, language, meta, &options, hooks)
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

    /// Renders a document as a complete MDX module that still contains JSX.
    ///
    /// The module holds the authored ESM, a content function, and `MDXContent`.
    /// It imports no framework; the caller's JSX transform compiles it. Parse
    /// the document with [`ParserOptions::mdx_compatible`] so that module
    /// blocks have exact source ranges.
    ///
    /// ```
    /// use ferromark::{Allocator, JsxModuleOptions, JsxRenderer, Parser, ParserOptions};
    ///
    /// let source = "import { Chart } from './chart.js'\n\n# Sales\n\n<Chart />\n\n<Note />\n";
    /// let allocator = Allocator::new();
    /// let options = ParserOptions { mdx: true, mdx_compatible: true, ..ParserOptions::default() };
    /// let document = Parser::with_options(&allocator, source, options).parse().unwrap();
    /// let module = JsxRenderer::new()
    ///     .render_module(&document, source, &JsxModuleOptions {
    ///         provider_import_source: Some("docs/provider".to_owned()),
    ///         filename: Some("sales.mdx".to_owned()),
    ///         ..JsxModuleOptions::default()
    ///     })
    ///     .unwrap();
    /// assert!(module.code.contains("export default function MDXContent(props = {})"));
    /// // `Chart` is imported; `Note` comes from the provider.
    /// assert_eq!(module.bindings, ["Chart"]);
    /// assert!(module.code.contains("const {Note} = _components;"));
    /// ```
    ///
    /// [`ParserOptions::mdx_compatible`]: crate::parser::ParserOptions::mdx_compatible
    #[cfg(feature = "jsx")]
    pub fn render_module(
        &self,
        document: &Document<'_>,
        source: &str,
        module: &JsxModuleOptions,
    ) -> Result<JsxModuleOutput, JsxModuleError> {
        self.render_module_with_hooks(document, source, module, &mut NoJsxRenderHooks)
    }

    /// Renders an MDX module with a synchronous code block hook.
    ///
    /// Generated Markdown elements are members of `_components`, so
    /// [`JsxRendererOptions::component_prefix`] must be unset or `_components`.
    #[cfg(feature = "jsx")]
    pub fn render_module_with_hooks<H: JsxRenderHooks>(
        &self,
        document: &Document<'_>,
        source: &str,
        module: &JsxModuleOptions,
        hooks: &mut H,
    ) -> Result<JsxModuleOutput, JsxModuleError> {
        let prefix = self.options.component_prefix.as_deref();
        if prefix.is_some_and(|prefix| prefix != module::COMPONENTS) {
            return Err(JsxModuleError::ComponentPrefix);
        }
        let options = JsxRendererOptions {
            component_prefix: Some(module::COMPONENTS.to_owned()),
            ..self.options.clone()
        };
        let (output, references) =
            render::render_document_with_references(document, source, &options, hooks);
        module::assemble(output, &references, source, module)
    }
}
