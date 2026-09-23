mod options;

use ferromark::{
    Allocator, HeadingIdPlanner, HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks,
    HtmlRenderer, Parser, ParserOptions,
    ast::{Node, Visit},
};
use napi::bindgen_prelude::{Buffer, Error, FnArgs, Function, Result, Status};
use napi_derive::napi;

use crate::options::{CoreOptions, addon_defaults};

#[cfg(feature = "panic-test")]
#[napi(catch_unwind)]
pub fn __test_panic_unwind() {
    panic!("ferromark N-API panic-unwind verification");
}

#[napi(object)]
pub struct Options {
    pub render_policy: Option<String>,
    pub allow_html: Option<bool>,
    pub tables: Option<bool>,
    pub merged_table_cells: Option<bool>,
    pub table_colgroup: Option<bool>,
    pub table_column_names: Option<bool>,
    pub table_attributes: Option<bool>,
    pub strikethrough: Option<bool>,
    pub superscript: Option<bool>,
    pub subscript: Option<bool>,
    pub task_lists: Option<bool>,
    pub autolink_literals: Option<bool>,
    pub disallowed_raw_html: Option<bool>,
    pub footnotes: Option<bool>,
    pub highlight: Option<bool>,
    pub inline_footnotes: Option<bool>,
    pub allow_link_refs: Option<bool>,
    pub front_matter: Option<bool>,
    pub heading_ids: Option<bool>,
    pub heading_offset: Option<f64>,
    pub heading_attributes: Option<bool>,
    pub math: Option<bool>,
    pub callouts: Option<bool>,
    pub definition_lists: Option<bool>,
    pub line_comments: Option<bool>,
    pub wiki_links: Option<bool>,
    pub cjk_emphasis: Option<bool>,
    pub mdx: Option<bool>,
    pub link_base_path: Option<String>,
}

fn core_options(options: Option<Options>) -> Result<CoreOptions> {
    let CoreOptions {
        mut parser,
        mut html,
        mut heading_level_offset,
    } = addon_defaults();
    if let Some(options) = options {
        if let Some(policy) = options.render_policy {
            html.sanitize = match policy.as_str() {
                "untrusted" => true,
                "trusted" => false,
                _ => {
                    return Err(Error::new(
                        Status::InvalidArg,
                        "renderPolicy must be either 'untrusted' or 'trusted'",
                    ));
                }
            };
        }
        if options.allow_html == Some(false) {
            html.sanitize = true;
        }
        macro_rules! apply {
            ($target:expr, $value:expr) => {
                if let Some(value) = $value {
                    $target = value;
                }
            };
        }
        apply!(parser.tables, options.tables);
        apply!(parser.merged_table_cells, options.merged_table_cells);
        apply!(html.table_colgroup, options.table_colgroup);
        apply!(html.table_column_names, options.table_column_names);
        apply!(parser.table_attributes, options.table_attributes);
        apply!(parser.strikethrough, options.strikethrough);
        apply!(parser.superscript, options.superscript);
        apply!(parser.subscript, options.subscript);
        apply!(parser.task_lists, options.task_lists);
        apply!(parser.autolinks, options.autolink_literals);
        apply!(html.disallow_raw_html, options.disallowed_raw_html);
        apply!(parser.footnotes, options.footnotes);
        apply!(parser.highlight, options.highlight);
        apply!(parser.inline_footnotes, options.inline_footnotes);
        apply!(parser.allow_link_refs, options.allow_link_refs);
        apply!(parser.front_matter, options.front_matter);
        apply!(html.heading_ids, options.heading_ids);
        if let Some(offset) = options.heading_offset {
            if !offset.is_finite()
                || offset.fract() != 0.0
                || offset < f64::from(i32::MIN)
                || offset > f64::from(i32::MAX)
            {
                return Err(Error::new(
                    Status::InvalidArg,
                    "headingOffset must be an integer in the signed 32-bit range",
                ));
            }
            heading_level_offset = offset as i32;
        }
        apply!(parser.heading_attributes, options.heading_attributes);
        apply!(parser.math, options.math);
        apply!(html.callouts, options.callouts);
        apply!(parser.definition_lists, options.definition_lists);
        apply!(parser.line_comments, options.line_comments);
        apply!(parser.wiki_links, options.wiki_links);
        apply!(parser.cjk_emphasis, options.cjk_emphasis);
        apply!(parser.mdx, options.mdx);
        if let Some(base) = options.link_base_path {
            // The JavaScript string is owned, so this becomes `Cow::Owned`;
            // every other renderer option keeps its borrowed default.
            html.base_url = base.into();
            html.convert_md_links = true;
        }
    }
    Ok(CoreOptions {
        parser,
        html,
        heading_level_offset,
    })
}

fn parse_error(error: ferromark::ParseError) -> Error {
    Error::new(Status::InvalidArg, error.to_string())
}

#[napi(catch_unwind)]
pub fn to_html(markdown: String, options: Option<Options>) -> Result<String> {
    // A one-shot renderer is dropped with this call, so its output buffer is
    // handed over whole rather than copied out of a borrow.
    let mut renderer = Renderer::new(options)?;
    let document = Parser::with_options(&renderer.allocator, &markdown, renderer.parser.clone())
        .parse()
        .map_err(parse_error)?;
    Ok(renderer.html.render(&document))
}

#[napi(catch_unwind)]
pub fn to_html_buffer(markdown: String, options: Option<Options>) -> Result<Buffer> {
    Renderer::new(options)?.to_html_buffer(markdown)
}

/// Reuses arena storage and HTML buffers; documents never outlive a call.
#[napi]
pub struct Renderer {
    allocator: Allocator,
    parser: ParserOptions,
    html: HtmlRenderer,
}

#[napi]
impl Renderer {
    #[napi(constructor, catch_unwind)]
    pub fn new(options: Option<Options>) -> Result<Self> {
        let options = core_options(options)?;
        let html = HtmlRenderer::with_options(options.html)
            .with_heading_level_offset(options.heading_level_offset);
        Ok(Self {
            allocator: Allocator::new(),
            parser: options.parser,
            html,
        })
    }

    // Returns a borrow of the renderer's own output buffer. N-API copies it
    // into a JavaScript string before anything else can touch the renderer,
    // and keeping the buffer, instead of handing it away with
    // `HtmlRenderer::render`, spares every later call a fresh allocation and
    // its regrowth. (A plain comment: doc comments become the published
    // TypeScript declarations.)
    #[napi(catch_unwind, js_name = "toHtml")]
    pub fn to_html(&mut self, markdown: String) -> Result<&str> {
        self.allocator.reset();
        let document = Parser::with_options(&self.allocator, &markdown, self.parser.clone())
            .parse()
            .map_err(parse_error)?;
        Ok(self.html.render_borrowed(&document))
    }

    #[napi(catch_unwind, js_name = "toHtmlBuffer")]
    pub fn to_html_buffer(&mut self, markdown: String) -> Result<Buffer> {
        self.allocator.reset();
        let document = Parser::with_options(&self.allocator, &markdown, self.parser.clone())
            .parse()
            .map_err(parse_error)?;
        Ok(self
            .html
            .render_borrowed(&document)
            .as_bytes()
            .to_vec()
            .into())
    }
}

#[napi(object)]
pub struct Heading {
    pub level: u32,
    pub id: Option<String>,
    pub text: String,
}

#[napi(object)]
pub struct TransformResult {
    pub html: String,
    pub headings: Vec<Heading>,
    pub front_matter: Option<String>,
}

struct Metadata {
    headings: Vec<Heading>,
    id_planner: HeadingIdPlanner,
    heading_ids: bool,
    heading_level_offset: i32,
}

impl<'a> Visit<'a> for Metadata {
    fn visit_heading(&mut self, heading: &ferromark::ast::Heading<'a>) {
        let text = ferromark::collect_heading_text(&heading.children);
        let id = if self.heading_ids {
            let base = heading
                .id
                .map_or_else(|| ferromark::slugify_heading(&text), str::to_owned);
            Some(self.id_planner.plan(&base))
        } else {
            None
        };
        self.headings.push(Heading {
            level: u32::from(ferromark::map_heading_level(
                heading.depth,
                self.heading_level_offset,
            )),
            id,
            text,
        });
    }
}

type CodeCallback<'a> =
    Function<'a, FnArgs<(String, Option<String>, Option<String>)>, Option<String>>;

struct CallbackRenderer<'a> {
    callback: CodeCallback<'a>,
    error: Option<Error>,
}

impl HtmlRenderHooks for CallbackRenderer<'_> {
    fn render_node(
        &mut self,
        node: &Node<'_>,
        cx: &mut HtmlRenderContext<'_>,
    ) -> HtmlRenderControl {
        if self.error.is_none()
            && let Node::CodeBlock(block) = node
        {
            // V2 hooks receive both fenced and indented code blocks.
            match self.callback.call(FnArgs::from((
                block.value.to_owned(),
                block.lang.map(str::to_owned),
                block.meta.map(str::to_owned),
            ))) {
                Ok(Some(html)) => {
                    cx.write(&html);
                    return HtmlRenderControl::Handled;
                }
                Ok(None) => {}
                Err(error) => self.error = Some(error),
            }
        }
        HtmlRenderControl::Default
    }
}

fn render_document(
    markdown: &str,
    options: CoreOptions,
    callback: Option<CodeCallback<'_>>,
) -> Result<TransformResult> {
    let allocator = Allocator::for_source_len(markdown.len());
    let document = Parser::with_options(&allocator, markdown, options.parser)
        .parse()
        .map_err(parse_error)?;
    let mut metadata = Metadata {
        headings: Vec::new(),
        id_planner: HeadingIdPlanner::new(),
        heading_ids: options.html.heading_ids,
        heading_level_offset: options.heading_level_offset,
    };
    metadata.visit_document(&document);
    let front_matter = document
        .front_matter
        .as_ref()
        .map(|front| front.value.to_owned());
    let mut renderer = HtmlRenderer::with_options(options.html)
        .with_heading_level_offset(options.heading_level_offset);
    let html = if let Some(callback) = callback {
        let mut hooks = CallbackRenderer {
            callback,
            error: None,
        };
        let html = renderer.render_with_hooks(&document, &mut hooks);
        if let Some(error) = hooks.error {
            return Err(error);
        }
        html
    } else {
        renderer.render(&document)
    };
    Ok(TransformResult {
        html,
        headings: metadata.headings,
        front_matter,
    })
}

#[napi(catch_unwind)]
pub fn transform(markdown: String, options: Option<Options>) -> Result<TransformResult> {
    render_document(&markdown, core_options(options)?, None)
}

#[napi(catch_unwind)]
#[allow(clippy::type_complexity)]
pub fn to_html_with_renderer(
    markdown: String,
    options: Option<Options>,
    renderer: Function<FnArgs<(String, Option<String>, Option<String>)>, Option<String>>,
) -> Result<String> {
    Ok(render_document(&markdown, core_options(options)?, Some(renderer))?.html)
}

#[napi(catch_unwind)]
#[allow(clippy::type_complexity)]
pub fn transform_with_renderer(
    markdown: String,
    options: Option<Options>,
    renderer: Function<FnArgs<(String, Option<String>, Option<String>)>, Option<String>>,
) -> Result<TransformResult> {
    render_document(&markdown, core_options(options)?, Some(renderer))
}
