#[cfg(feature = "boundary-bench")]
pub mod boundary;
mod default_renderer;
pub mod input;
mod options;
pub mod packed;

use std::collections::HashMap;

use crate::input::Utf8Input;
use crate::options::{CoreOptions, addon_defaults};
use ferromark::{
    Allocator, HeadingIdPlanner, HtmlRenderContext, HtmlRenderControl, HtmlRenderHooks,
    HtmlRenderer, Parser, ParserOptions,
    ast::{Node, Visit},
};
use napi::bindgen_prelude::{Buffer, Error, FnArgs, Function, Result, Status};
use napi::{Env, JsString};
use napi_derive::napi;

#[cfg(feature = "panic-test")]
#[napi(catch_unwind)]
pub fn __test_panic_unwind() {
    panic!("ferromark N-API panic-unwind verification");
}

// Renders `markdown` on the thread's kept renderer and panics while the
// renderer is still out of its slot, so the verification can check that later
// calls render as before.
#[cfg(feature = "panic-test")]
#[napi(catch_unwind)]
pub fn __test_panic_in_default_renderer(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
) -> Result<()> {
    render_one_shot(&markdown, None, |_| {
        panic!("ferromark N-API panic-unwind verification in the kept renderer")
    })
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
    pub heading_id_prefix: Option<String>,
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
        mut heading_id_prefix,
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
        if let Some(prefix) = options.heading_id_prefix {
            HtmlRenderer::validate_heading_id_prefix(&prefix)
                .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
            heading_id_prefix = prefix;
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
        heading_id_prefix,
    })
}

fn parse_error(error: ferromark::ParseError) -> Error {
    Error::new(Status::InvalidArg, error.to_string())
}

/// The Rust side of `toHtml` and `toHtmlBuffer`, shared with the
/// `boundary-bench` diagnostics so they time exactly the code the exports run.
///
/// Without options, the call renders with this thread's kept renderer (see
/// `default_renderer.rs`); with options, with a renderer of its own. Either
/// way `emit` converts the HTML for JavaScript while the renderer still holds
/// it, and its result is the call's.
fn render_one_shot<T>(
    markdown: &str,
    options: Option<Options>,
    emit: impl FnOnce(&str) -> Result<T>,
) -> Result<T> {
    match options {
        None => default_renderer::render(markdown, emit),
        Some(_) => render_fresh(markdown, options, emit),
    }
}

/// Renders `markdown` with a renderer built for this call, hands the HTML to
/// `emit`, and drops the renderer.
fn render_fresh<T>(
    markdown: &str,
    options: Option<Options>,
    emit: impl FnOnce(&str) -> Result<T>,
) -> Result<T> {
    let mut renderer = Renderer::new(options)?;
    emit(renderer.render_reused(markdown)?)
}

/// Copies HTML into a JavaScript string as napi-rs converts a returned
/// `&str` or `String`: one `napi_create_string_utf8`, with napi-rs's error.
fn js_string<'env>(env: &'env Env, html: &str) -> Result<JsString<'env>> {
    env.create_string(html).map_err(|error| {
        Error::new(
            error.status,
            "Failed to convert rust `&str` into napi `string`",
        )
    })
}

/// Copies HTML into a `Buffer`, as `toHtmlBuffer` returns it.
fn html_buffer(html: &str) -> Result<Buffer> {
    Ok(html.as_bytes().to_vec().into())
}

// Every export that takes Markdown accepts a string or a `Uint8Array` of UTF-8
// through `Utf8Input` (see `input.rs`), which copies the bytes when the export
// first uses the text. `toHtml` creates its JavaScript string itself, from HTML
// the renderer still holds, and napi-rs declares the `JsString` it returns as
// `string`. (Plain comments: doc comments on exports become the published
// TypeScript declarations.)
#[napi(catch_unwind)]
pub fn to_html<'env>(
    env: &'env Env,
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    options: Option<Options>,
) -> Result<JsString<'env>> {
    render_one_shot(&markdown, options, |html| js_string(env, html))
}

#[napi(catch_unwind)]
pub fn to_html_buffer(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    options: Option<Options>,
) -> Result<Buffer> {
    render_one_shot(&markdown, options, html_buffer)
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
            .with_heading_level_offset(options.heading_level_offset)
            .try_with_heading_id_prefix(options.heading_id_prefix)
            .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
        Ok(Self {
            allocator: Allocator::new(),
            parser: options.parser,
            html,
        })
    }

    /// Internal to the `ferromark` facade: the constructor with packed options.
    #[napi(factory, catch_unwind, js_name = "withPackedOptions")]
    pub fn with_packed_options(
        set: u32,
        on: u32,
        heading_offset: Option<f64>,
        heading_id_prefix: Option<String>,
        link_base_path: Option<String>,
    ) -> Result<Self> {
        let options = packed::unpack(set, on, heading_offset, heading_id_prefix, link_base_path);
        Self::new(Some(options))
    }

    // Returns a borrow of the renderer's own output buffer. N-API copies it
    // into a JavaScript string before anything else can touch the renderer,
    // and keeping the buffer, instead of handing it away with
    // `HtmlRenderer::render`, spares every later call a fresh allocation and
    // its regrowth. (A plain comment: doc comments become the published
    // TypeScript declarations.)
    #[napi(catch_unwind, js_name = "toHtml")]
    pub fn to_html(
        &mut self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    ) -> Result<&str> {
        self.render_reused(&markdown)
    }

    #[napi(catch_unwind, js_name = "toHtmlBuffer")]
    pub fn to_html_buffer(
        &mut self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    ) -> Result<Buffer> {
        html_buffer(self.render_reused(&markdown)?)
    }
}

impl Renderer {
    /// The Rust side of the reused `toHtml` and `toHtmlBuffer`, and of the
    /// one-shot ones, shared with the `boundary-bench` diagnostics so they
    /// time exactly this code.
    ///
    /// The arena is reset, and `HtmlRenderer::render_borrowed` clears the
    /// output, heading IDs and footnote state before it writes, so a document
    /// renders the same whatever the renderer rendered before.
    fn render_reused(&mut self, markdown: &str) -> Result<&str> {
        self.allocator.reset();
        let document = Parser::with_options(&self.allocator, markdown, self.parser.clone())
            .parse()
            .map_err(parse_error)?;
        Ok(self.html.render_borrowed(&document))
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
    legacy_footnote_targets: HashMap<String, String>,
    legacy_footnote_reference_counts: HashMap<String, usize>,
    legacy_footnote_first_references: HashMap<String, String>,
    heading_ids: bool,
    heading_level_offset: i32,
    heading_id_prefix: String,
}

impl Metadata {
    fn legacy_footnote_target_id(&mut self, identifier: &str) -> String {
        if let Some(id) = self.legacy_footnote_targets.get(identifier) {
            return id.clone();
        }
        let mut base = String::from("fn-");
        base.push_str(identifier);
        let id = self.id_planner.plan(&base);
        self.legacy_footnote_targets
            .insert(identifier.to_owned(), id.clone());
        id
    }

    fn plan_legacy_footnote_reference(&mut self, identifier: &str) {
        let _ = self.legacy_footnote_target_id(identifier);
        let occurrence = self
            .legacy_footnote_reference_counts
            .entry(identifier.to_owned())
            .or_default();
        *occurrence += 1;
        let occurrence = *occurrence;
        if occurrence == 1 {
            let _ = self.legacy_footnote_first_reference_id(identifier);
            return;
        }
        let mut base = String::from("fnref-");
        base.push_str(identifier);
        if occurrence > 1 {
            base.push('-');
            base.push_str(&occurrence.to_string());
        }
        let _ = self.id_planner.plan(&base);
    }

    fn legacy_footnote_first_reference_id(&mut self, identifier: &str) -> String {
        if let Some(id) = self.legacy_footnote_first_references.get(identifier) {
            return id.clone();
        }
        let mut base = String::from("fnref-");
        base.push_str(identifier);
        let id = self.id_planner.plan(&base);
        self.legacy_footnote_first_references
            .insert(identifier.to_owned(), id.clone());
        id
    }
}

impl<'a> Visit<'a> for Metadata {
    fn visit_heading(&mut self, heading: &ferromark::ast::Heading<'a>) {
        let text = ferromark::collect_heading_text(&heading.children);
        let id = if self.heading_ids {
            let base = heading
                .id
                .map_or_else(|| ferromark::slugify_heading(&text), str::to_owned);
            let mut requested = self.heading_id_prefix.clone();
            requested.push_str(&base);
            Some(self.id_planner.plan(&requested))
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
        ferromark::ast::walk_heading(self, heading);
    }

    fn visit_footnote_reference(&mut self, footnote_ref: &ferromark::ast::FootnoteReference<'a>) {
        self.plan_legacy_footnote_reference(footnote_ref.identifier);
    }

    fn visit_footnote_definition(&mut self, footnote_def: &ferromark::ast::FootnoteDefinition<'a>) {
        let _ = self.legacy_footnote_target_id(footnote_def.identifier);
        ferromark::ast::walk_footnote_definition(self, footnote_def);
        let _ = self.legacy_footnote_first_reference_id(footnote_def.identifier);
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
        legacy_footnote_targets: HashMap::new(),
        legacy_footnote_reference_counts: HashMap::new(),
        legacy_footnote_first_references: HashMap::new(),
        heading_ids: options.html.heading_ids,
        heading_level_offset: options.heading_level_offset,
        heading_id_prefix: options.heading_id_prefix.clone(),
    };
    metadata.visit_document(&document);
    let front_matter = document
        .front_matter
        .as_ref()
        .map(|front| front.value.to_owned());
    let mut renderer = HtmlRenderer::with_options(options.html)
        .with_heading_level_offset(options.heading_level_offset)
        .try_with_heading_id_prefix(options.heading_id_prefix)
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
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
pub fn transform(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    options: Option<Options>,
) -> Result<TransformResult> {
    render_document(&markdown, core_options(options)?, None)
}

#[napi(catch_unwind)]
#[allow(clippy::type_complexity)]
pub fn to_html_with_renderer(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    options: Option<Options>,
    renderer: Function<FnArgs<(String, Option<String>, Option<String>)>, Option<String>>,
) -> Result<String> {
    Ok(render_document(&markdown, core_options(options)?, Some(renderer))?.html)
}

#[napi(catch_unwind)]
#[allow(clippy::type_complexity)]
pub fn transform_with_renderer(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    options: Option<Options>,
    renderer: Function<FnArgs<(String, Option<String>, Option<String>)>, Option<String>>,
) -> Result<TransformResult> {
    render_document(&markdown, core_options(options)?, Some(renderer))
}
