//! Public configuration types for HTML rendering.
//!
//! Keeping options separate from the renderer implementation makes the public API easy
//! to scan: this module owns only user-supplied configuration and lightweight enums.

use std::borrow::Cow;

/// HTML renderer options.
///
/// Use [`HtmlRendererOptions::new`] or [`Default::default`] for the documented
/// defaults.
///
/// # String-valued fields
///
/// Every string-valued field is a [`Cow<'static, str>`](Cow), and
/// [`Self::autolink_patterns`] is a borrowed-or-owned list of them. Every
/// documented default is a compile-time constant, so the defaults are borrowed:
/// building and cloning a default options value never touches the allocator,
/// and a renderer constructed per document from such a value costs nothing for
/// its configuration.
///
/// Supply your own values with `.into()`. A `&'static str` borrows and an owned
/// `String` moves:
///
/// ```
/// use ferromark::HtmlRendererOptions;
///
/// let configured_base = String::from("/docs/");
/// let options = HtmlRendererOptions {
///     hard_break: "<br />\n".into(),
///     base_url: configured_base.into(),
///     autolink_patterns: vec!["https://".into(), "mailto:".into()].into(),
///     ..HtmlRendererOptions::default()
/// };
/// assert_eq!(&*options.hard_break, "<br />\n");
/// assert_eq!(options.autolink_patterns.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct HtmlRendererOptions {
    /// Use XHTML-style self-closing tags (e.g., `<br />`).
    ///
    /// Default: `false`.
    pub xhtml: bool,

    /// String emitted for a soft line break.
    ///
    /// A soft break is a line ending inside inline content: the newline that
    /// joins two lines of the same paragraph, heading, or table cell. A hard
    /// break is a separate AST node and uses [`Self::hard_break`] instead.
    ///
    /// The value replaces every line ending in rendered inline text, including
    /// one written as a character reference such as `&#10;`. It is written
    /// verbatim, exactly like [`Self::hard_break`], so [`Self::xhtml`] does not
    /// rewrite it; supply `"<br />"` yourself when the output has to be XHTML.
    ///
    /// Default: `"\n"`.
    pub soft_break: Cow<'static, str>,

    /// String emitted for a hard line break.
    ///
    /// The value is written verbatim; [`Self::xhtml`] does not rewrite it.
    ///
    /// Default: `"<br>\n"`.
    pub hard_break: Cow<'static, str>,

    /// Sanitize HTML output.
    ///
    /// Default: `false`.
    pub sanitize: bool,

    /// Apply the GFM `tagfilter` extension ("Disallowed Raw HTML"):
    /// neutralize `<title>`, `<textarea>`, `<style>`, `<xmp>`, `<iframe>`,
    /// `<noembed>`, `<noframes>`, `<script>`, and `<plaintext>` by escaping
    /// their leading `<`, leaving all other raw HTML untouched.
    ///
    /// Unlike [`Self::sanitize`], which escapes every raw HTML node, this
    /// keeps ordinary markup working. It is off by default because raw HTML
    /// passthrough is standard Markdown behaviour that embeds rely on.
    ///
    /// Default: `false`.
    pub disallow_raw_html: bool,

    /// Convert `.md` links to `.html` links for SSG output.
    ///
    /// Default: `false`.
    pub convert_md_links: bool,

    /// Base URL for absolute link conversion (e.g., "/" or "/docs/").
    ///
    /// Default: `"/"`.
    pub base_url: Cow<'static, str>,

    /// Source file path for relative link resolution.
    /// Used to determine if the current file is an index file.
    ///
    /// Default: empty string.
    pub source_path: Cow<'static, str>,

    /// Enable line annotations for code blocks using fence meta.
    ///
    /// Default: `false`.
    pub code_annotations: bool,

    /// Fence meta key used to read code annotations.
    ///
    /// Default: `"annotate"`.
    pub code_annotation_meta_key: Cow<'static, str>,

    /// Code annotation syntax mode.
    ///
    /// Default: [`CodeAnnotationSyntax::Attribute`].
    pub code_annotation_syntax: CodeAnnotationSyntax,

    /// Enable line numbers for all code blocks by default.
    ///
    /// Default: `false`.
    pub code_annotation_default_line_numbers: bool,

    /// Maximum heading depth included in inline TOCs.
    ///
    /// Default: `3`.
    pub toc_max_depth: u8,

    /// Auto-link bare URLs in text. When enabled, any occurrence in a text
    /// node that starts with one of [`Self::autolink_patterns`] is wrapped
    /// in an `<a>` tag. Auto-linking is suppressed inside an existing link.
    ///
    /// Default: `true`.
    pub autolink_urls: bool,

    /// URL prefix patterns recognised by [`Self::autolink_urls`]. Defaults
    /// to `["http://", "https://"]`, borrowed from static data. Register
    /// additional schemes (e.g. `"ftp://"`, `"mailto:"`) by replacing the
    /// list: `vec!["https://".into(), "ftp://".into()].into()`. An empty
    /// list disables auto-linking just as [`Self::autolink_urls`] does.
    ///
    /// Default: `["http://", "https://"]`.
    pub autolink_patterns: Cow<'static, [Cow<'static, str>]>,

    /// When auto-linking, emit `target="_blank" rel="noopener noreferrer"`.
    /// Independent from markdown-link behaviour; use
    /// [`Self::link_target_blank`] for parsed `Link` nodes.
    ///
    /// Default: `true`.
    pub autolink_target_blank: bool,

    /// When rendering Markdown `Link` nodes with http(s) hrefs, emit
    /// `target="_blank" rel="noopener noreferrer"`.
    ///
    /// Default: `true`.
    pub link_target_blank: bool,

    /// Render footnotes as one ordered section with numeric display markers.
    ///
    /// Off by default so the established v2 HTML stays stable. When on, source
    /// identifiers are used only for lookup and slugs; visible markers are
    /// 1, 2, … in document order, and definitions emit as
    /// `<section class="footnotes"><ol><li>…`.
    ///
    /// Default: `false`.
    pub semantic_footnotes: bool,

    /// Append a visible heading permalink after the heading children.
    ///
    /// Default: `false`. Off output is byte-identical to previous releases.
    /// When on, each heading that does not already contain the permalink
    /// marker (`class="header-anchor"` or a `#` link to the same id) gains:
    ///
    /// ```html
    /// <a class="header-anchor" href="#{id}" aria-label="Permalink to &quot;{text}&quot;">#</a>
    /// ```
    ///
    /// `{id}` is the exact generated heading id (including `-N` suffixes).
    /// Empty headings use `aria-label="Permalink to this section"`. Visibility
    /// (always vs hover/focus-visible) is CSS-only and does not change this
    /// markup.
    pub heading_permalinks: bool,

    /// Emit `data-source-span="start-end"` on rendered block elements.
    ///
    /// Values are byte offsets into the original Markdown source, matching
    /// the AST [`crate::ast::Span`] contract. Raw HTML nodes are left
    /// untouched.
    ///
    /// Default: `false`.
    pub source_spans: bool,

    /// Emit `id` attributes on headings, including explicit IDs from heading
    /// attributes and generated IDs for ordinary headings.
    ///
    /// Default: `true`. The strict CommonMark and GFM profiles disable this
    /// product convenience because heading IDs are not part of the HTML
    /// defined by the Markdown specifications.
    pub heading_ids: bool,

    /// Render GitHub-style `[!NOTE]` block quotes as themed callouts.
    ///
    /// Default: `true`; strict profiles disable this product extension.
    pub callouts: bool,

    /// Interpret standalone `[[toc]]` paragraphs as inline tables of contents.
    ///
    /// Default: `true`; strict profiles disable this product extension.
    pub inline_toc: bool,

    /// Parse and clean VitePress-style fenced-code metadata and annotations.
    ///
    /// Default: `true`. When disabled, fenced code uses the plain fence path:
    /// annotations are not applied and the parser-provided first info token is
    /// emitted as the language verbatim (after ordinary HTML escaping).
    pub code_fence_metadata: bool,

    /// Emit a `<colgroup>` containing one CSS-named column for each table
    /// alignment entry.
    ///
    /// Default: `false`.
    pub table_colgroup: bool,

    /// Add `col-name-<slug>` CSS classes derived from the first table row.
    ///
    /// Uses heading text/slug rules and retains positional `col-N` classes.
    /// Requires [`Self::table_colgroup`]. Default: `false` in every preset.
    pub table_column_names: bool,
}

const DEFAULT_SOFT_BREAK: &str = "\n";
const DEFAULT_HARD_BREAK: &str = "<br>\n";
const DEFAULT_BASE_URL: &str = "/";
const DEFAULT_SOURCE_PATH: &str = "";
const DEFAULT_CODE_ANNOTATION_META_KEY: &str = "annotate";
const DEFAULT_AUTOLINK_PATTERNS: &[Cow<'static, str>] =
    &[Cow::Borrowed("http://"), Cow::Borrowed("https://")];

/// Internal form of [`HtmlRendererOptions`], holding only what rendering reads.
///
/// The public options are moved in field by field, so every value arrives
/// exactly as the caller wrote it — including empty strings and an empty
/// pattern list, which are meaningful and must not be read as "use the
/// default". Defaults borrow static data, so a renderer built from a default
/// options value performs no allocation for its configuration and frees
/// nothing when it drops.
pub(super) struct RendererOptions {
    pub(super) xhtml: bool,
    soft_break: Cow<'static, str>,
    /// `true` when `soft_break` differs from the default line ending, so the
    /// text path can skip the soft-break check for the common configuration.
    pub(super) custom_soft_break: bool,
    hard_break: Cow<'static, str>,
    pub(super) sanitize: bool,
    pub(super) disallow_raw_html: bool,
    pub(super) convert_md_links: bool,
    base_url: Cow<'static, str>,
    source_path: Cow<'static, str>,
    pub(super) code_annotations: bool,
    code_annotation_meta_key: Cow<'static, str>,
    pub(super) code_annotation_syntax: CodeAnnotationSyntax,
    pub(super) code_annotation_default_line_numbers: bool,
    pub(super) toc_max_depth: u8,
    pub(super) autolink_urls: bool,
    autolink_patterns: Cow<'static, [Cow<'static, str>]>,
    pub(super) autolink_target_blank: bool,
    pub(super) link_target_blank: bool,
    pub(super) semantic_footnotes: bool,
    pub(super) heading_permalinks: bool,
    pub(super) source_spans: bool,
    pub(super) heading_ids: bool,
    pub(super) callouts: bool,
    pub(super) inline_toc: bool,
    pub(super) code_fence_metadata: bool,
    pub(super) table_colgroup: bool,
    pub(super) table_column_names: bool,
}

impl RendererOptions {
    pub(super) fn soft_break(&self) -> &str {
        &self.soft_break
    }

    pub(super) fn hard_break(&self) -> &str {
        &self.hard_break
    }

    pub(super) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(super) fn source_path(&self) -> &str {
        &self.source_path
    }

    pub(super) fn code_annotation_meta_key(&self) -> &str {
        &self.code_annotation_meta_key
    }

    /// The configured URL prefixes, empty when auto-linking has none left.
    pub(super) fn autolink_patterns(&self) -> &[Cow<'static, str>] {
        &self.autolink_patterns
    }
}

impl From<HtmlRendererOptions> for RendererOptions {
    fn from(options: HtmlRendererOptions) -> Self {
        Self {
            xhtml: options.xhtml,
            custom_soft_break: options.soft_break != DEFAULT_SOFT_BREAK,
            soft_break: options.soft_break,
            hard_break: options.hard_break,
            sanitize: options.sanitize,
            disallow_raw_html: options.disallow_raw_html,
            convert_md_links: options.convert_md_links,
            base_url: options.base_url,
            source_path: options.source_path,
            code_annotations: options.code_annotations,
            code_annotation_meta_key: options.code_annotation_meta_key,
            code_annotation_syntax: options.code_annotation_syntax,
            code_annotation_default_line_numbers: options.code_annotation_default_line_numbers,
            toc_max_depth: options.toc_max_depth,
            autolink_urls: options.autolink_urls,
            autolink_patterns: options.autolink_patterns,
            autolink_target_blank: options.autolink_target_blank,
            link_target_blank: options.link_target_blank,
            semantic_footnotes: options.semantic_footnotes,
            heading_permalinks: options.heading_permalinks,
            source_spans: options.source_spans,
            heading_ids: options.heading_ids,
            callouts: options.callouts,
            inline_toc: options.inline_toc,
            code_fence_metadata: options.code_fence_metadata,
            table_colgroup: options.table_colgroup,
            table_column_names: options.table_column_names,
        }
    }
}

impl HtmlRendererOptions {
    /// Creates new options with default values.
    ///
    /// Every default is static data, so this performs no heap allocation, and
    /// neither does cloning the result.
    #[must_use]
    pub fn new() -> Self {
        Self {
            xhtml: false,
            soft_break: Cow::Borrowed(DEFAULT_SOFT_BREAK),
            hard_break: Cow::Borrowed(DEFAULT_HARD_BREAK),
            sanitize: false,
            disallow_raw_html: false,
            convert_md_links: false,
            base_url: Cow::Borrowed(DEFAULT_BASE_URL),
            source_path: Cow::Borrowed(DEFAULT_SOURCE_PATH),
            code_annotations: false,
            code_annotation_meta_key: Cow::Borrowed(DEFAULT_CODE_ANNOTATION_META_KEY),
            code_annotation_syntax: CodeAnnotationSyntax::Attribute,
            code_annotation_default_line_numbers: false,
            toc_max_depth: 3,
            autolink_urls: true,
            autolink_patterns: Cow::Borrowed(DEFAULT_AUTOLINK_PATTERNS),
            autolink_target_blank: true,
            link_target_blank: true,
            semantic_footnotes: false,
            heading_permalinks: false,
            source_spans: false,
            heading_ids: true,
            callouts: true,
            inline_toc: true,
            code_fence_metadata: true,
            table_colgroup: false,
            table_column_names: false,
        }
    }

    /// Creates the strict CommonMark HTML profile.
    ///
    /// Product conveniences remain available through [`Self::new`] and
    /// [`Default::default`]. This profile keeps raw HTML passthrough but does
    /// not add IDs, callouts, TOCs, URL autolinks, link targets, or
    /// VitePress fence metadata cleanup.
    #[must_use]
    pub fn commonmark() -> Self {
        let mut options = Self::new();
        options.autolink_urls = false;
        options.autolink_target_blank = false;
        options.link_target_blank = false;
        options.heading_ids = false;
        options.callouts = false;
        options.inline_toc = false;
        options.code_fence_metadata = false;
        options
    }

    /// Creates the GFM convenience HTML profile.
    ///
    /// This adds GFM tag filtering to [`Self::new`], so it keeps the product
    /// conveniences — heading IDs, callouts, TOC substitution, URL
    /// autolinking, link targets, and VitePress fence metadata cleanup. It
    /// pairs with [`ParserOptions::gfm`](crate::ParserOptions::gfm); use
    /// [`Self::gfm_spec`] for specification-oriented output.
    #[must_use]
    pub fn gfm() -> Self {
        let mut options = Self::new();
        options.disallow_raw_html = true;
        options
    }

    /// Creates the strict GFM HTML profile.
    ///
    /// This adds GFM tag filtering to the strict CommonMark HTML profile and
    /// pairs with [`ParserOptions::gfm_spec`](crate::ParserOptions::gfm_spec).
    #[must_use]
    pub fn gfm_spec() -> Self {
        let mut options = Self::commonmark();
        options.disallow_raw_html = true;
        options
    }
}

impl Default for HtmlRendererOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Which fenced-code annotation dialect [`HtmlRendererOptions::code_annotations`]
/// reads.
///
/// The variants select the metadata parsers that run for a fenced code block:
/// the ox-content attribute syntax, the VitePress-compatible syntax, or both.
/// Only [`HtmlRendererOptions::code_annotations`] decides whether annotations
/// are applied at all; this decides how they are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeAnnotationSyntax {
    /// Read `annotate="kind:line"` style metadata from the code-fence info string.
    ///
    /// This is the stable ox-content syntax and is useful when authored Markdown should
    /// stay independent from a particular documentation theme.
    Attribute,

    /// Read VitePress-compatible fence metadata and inline `// [!code ...]` directives.
    ///
    /// Use this when importing or sharing Markdown with VitePress projects that already
    /// use `{1,3}`, `[title]`, `:line-numbers`, or inline diff/focus annotations.
    VitePress,

    /// Accept both ox-content attributes and VitePress-compatible directives.
    ///
    /// Attribute annotations are applied first, then VitePress metadata can add titles,
    /// line numbers, and inline directives without replacing existing classes.
    Both,
}

impl CodeAnnotationSyntax {
    pub(super) fn includes_attribute(self) -> bool {
        matches!(self, Self::Attribute | Self::Both)
    }

    pub(super) fn includes_vitepress(self) -> bool {
        matches!(self, Self::VitePress | Self::Both)
    }
}
