use ferromark::{
    FencedCodeBlock, FencedCodeRenderer, InputSizeError, Options as CoreOptions, RenderPolicy,
    Renderer as CoreRenderer, TrustedHtml,
};
use napi::bindgen_prelude::{Error, FnArgs, Function, Result, Status};
use napi_derive::napi;

#[cfg(feature = "panic-test")]
#[napi(catch_unwind)]
pub fn __test_panic_unwind() {
    panic!("ferromark N-API panic-unwind verification");
}

#[napi(object)]
pub struct Options {
    pub render_policy: Option<String>,
    pub allow_html: Option<bool>,
    pub allow_link_refs: Option<bool>,
    pub tables: Option<bool>,
    pub merged_table_cells: Option<bool>,
    pub table_column_widths: Option<bool>,
    pub strikethrough: Option<bool>,
    pub highlight: Option<bool>,
    pub superscript: Option<bool>,
    pub subscript: Option<bool>,
    pub task_lists: Option<bool>,
    pub autolink_literals: Option<bool>,
    pub disallowed_raw_html: Option<bool>,
    pub footnotes: Option<bool>,
    pub inline_footnotes: Option<bool>,
    pub front_matter: Option<bool>,
    pub heading_ids: Option<bool>,
    pub math: Option<bool>,
    pub callouts: Option<bool>,
    pub definition_lists: Option<bool>,
    pub line_comments: Option<bool>,
    pub indented_code_blocks: Option<bool>,
    pub link_base_path: Option<String>,
}

impl Options {
    fn into_core(self) -> Result<CoreOptions> {
        let mut options = CoreOptions::default();

        if let Some(policy) = self.render_policy {
            options.render_policy = match policy.as_str() {
                "untrusted" => RenderPolicy::Untrusted,
                "trusted" => RenderPolicy::Trusted,
                _ => {
                    return Err(Error::new(
                        Status::InvalidArg,
                        "renderPolicy must be either 'untrusted' or 'trusted'",
                    ));
                }
            };
        }

        apply(&mut options.allow_html, self.allow_html);
        apply(&mut options.allow_link_refs, self.allow_link_refs);
        apply(&mut options.tables, self.tables);
        apply(&mut options.merged_table_cells, self.merged_table_cells);
        apply(&mut options.table_column_widths, self.table_column_widths);
        apply(&mut options.strikethrough, self.strikethrough);
        apply(&mut options.highlight, self.highlight);
        apply(&mut options.superscript, self.superscript);
        apply(&mut options.subscript, self.subscript);
        apply(&mut options.task_lists, self.task_lists);
        apply(&mut options.autolink_literals, self.autolink_literals);
        apply(&mut options.disallowed_raw_html, self.disallowed_raw_html);
        apply(&mut options.footnotes, self.footnotes);
        apply(&mut options.inline_footnotes, self.inline_footnotes);
        apply(&mut options.front_matter, self.front_matter);
        apply(&mut options.heading_ids, self.heading_ids);
        apply(&mut options.math, self.math);
        apply(&mut options.callouts, self.callouts);
        apply(&mut options.definition_lists, self.definition_lists);
        apply(&mut options.line_comments, self.line_comments);
        apply(&mut options.indented_code_blocks, self.indented_code_blocks);
        options.link_base_path = self.link_base_path.map(String::into_boxed_str);

        Ok(options)
    }
}

fn apply(target: &mut bool, value: Option<bool>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn core_options(options: Option<Options>) -> Result<CoreOptions> {
    options.map_or_else(|| Ok(CoreOptions::default()), Options::into_core)
}

fn input_size_error(error: InputSizeError) -> Error {
    Error::new(Status::InvalidArg, error.to_string())
}

#[napi(catch_unwind)]
pub fn to_html(markdown: String, options: Option<Options>) -> Result<String> {
    ferromark::try_to_html_with_options(&markdown, &core_options(options)?)
        .map_err(input_size_error)
}

/// Reusable Markdown-to-HTML renderer with fixed options.
#[napi]
pub struct Renderer {
    inner: CoreRenderer,
}

#[napi]
impl Renderer {
    /// Create a renderer whose parser scratch space is retained between calls.
    #[napi(constructor, catch_unwind)]
    pub fn new(options: Option<Options>) -> Result<Self> {
        Ok(Self {
            inner: CoreRenderer::with_options(core_options(options)?),
        })
    }

    /// Render one Markdown document and retain scratch allocations for the next.
    #[napi(catch_unwind, js_name = "toHtml")]
    pub fn to_html(&mut self, markdown: String) -> Result<String> {
        self.inner.try_render(&markdown).map_err(input_size_error)
    }
}

/// One document heading, in source order.
#[napi(object)]
pub struct Heading {
    /// Heading level, 1-6.
    pub level: u32,
    /// The generated slug; present when the headingIds option is enabled.
    pub id: Option<String>,
    /// Plain heading text with inline markup and HTML tags removed.
    pub text: String,
}

/// Result of `transform`: HTML plus document metadata.
#[napi(object)]
pub struct TransformResult {
    pub html: String,
    /// Document headings for table-of-contents rendering.
    pub headings: Vec<Heading>,
    /// Raw front matter text (between the delimiters); present when the
    /// frontMatter option is enabled and the document starts with a block.
    pub front_matter: Option<String>,
}

fn transform_result(result: ferromark::ParseResult<'_>) -> TransformResult {
    TransformResult {
        html: result.html,
        headings: result
            .headings
            .into_iter()
            .map(|heading| Heading {
                level: u32::from(heading.level),
                id: heading.id,
                text: heading.text,
            })
            .collect(),
        front_matter: result.front_matter.map(str::to_owned),
    }
}

/// Render Markdown and return HTML together with headings and front matter.
#[napi(catch_unwind)]
pub fn transform(markdown: String, options: Option<Options>) -> Result<TransformResult> {
    let options = core_options(options)?;
    ferromark::try_parse_with_options(&markdown, &options)
        .map(transform_result)
        .map_err(input_size_error)
}

#[allow(clippy::type_complexity)]
struct CallbackRenderer<'scope> {
    callback: Function<'scope, FnArgs<(String, Option<String>, Option<String>)>, Option<String>>,
}

impl FencedCodeRenderer for CallbackRenderer<'_> {
    fn render(&mut self, block: FencedCodeBlock<'_>) -> Option<TrustedHtml> {
        self.callback
            .call(FnArgs::from((
                block.code.to_owned(),
                block.language.map(str::to_owned),
                block.meta.map(str::to_owned),
            )))
            .ok()
            .flatten()
            .map(TrustedHtml::from_trusted)
    }
}

#[napi(catch_unwind)]
#[allow(clippy::type_complexity)]
pub fn to_html_with_renderer(
    markdown: String,
    options: Option<Options>,
    renderer: Function<FnArgs<(String, Option<String>, Option<String>)>, Option<String>>,
) -> Result<String> {
    let options = core_options(options)?;
    let mut renderer = CallbackRenderer { callback: renderer };
    ferromark::try_to_html_with_renderer(&markdown, &options, &mut renderer)
        .map_err(input_size_error)
}

/// `transform` with an opt-in fenced-code renderer callback.
///
/// The callback receives `(code, language, meta)` and must return trusted,
/// fully escaped HTML — or null/undefined to fall back to the default
/// escaped `<pre><code>` output.
#[napi(catch_unwind)]
#[allow(clippy::type_complexity)]
pub fn transform_with_renderer(
    markdown: String,
    options: Option<Options>,
    renderer: Function<FnArgs<(String, Option<String>, Option<String>)>, Option<String>>,
) -> Result<TransformResult> {
    let options = core_options(options)?;
    let mut renderer = CallbackRenderer { callback: renderer };
    ferromark::try_parse_with_renderer(&markdown, &options, &mut renderer)
        .map(transform_result)
        .map_err(input_size_error)
}
