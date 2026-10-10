//! Framework-neutral JSX compilation at the native binding boundary.

use std::collections::{BTreeMap, HashMap};

use ferromark::ast::{CodeBlock, Document, Node, Span, Visit};
use ferromark::ferriki::Highlighter;
use ferromark::{
    Allocator, FerrikiJsxHooks, HtmlRenderer, JsxCodeBlockInput, JsxHighlightedCodeBlock,
    JsxModuleOptions, JsxRenderHooks, JsxRenderer, JsxRendererOptions, OutlineOptions, Parser,
};
use ferromark_transforms::TransformContext;
use napi::bindgen_prelude::{Error, FnArgs, Result, Status};
use napi_derive::napi;
use self_cell::self_cell;

use crate::{
    CodeCallback, Options, core_options, input::Utf8Input, options::CoreOptions, parse_error,
};

#[napi(object)]
#[derive(Default)]
pub struct JsxOptions {
    pub format: Option<String>,
    pub component_prefix: Option<String>,
    pub callout_components: Option<HashMap<String, String>>,
    pub code_components: Option<HashMap<String, String>>,
    pub code_block_component: Option<String>,
    pub omit_title_heading: Option<String>,
    /// Render-time heading ID setting. Preparation metadata uses its defaults.
    pub heading_ids: Option<bool>,
    /// Render-time signed 32-bit heading level offset.
    pub heading_offset: Option<f64>,
    /// Render-time prefix for generated and authored heading IDs.
    pub heading_id_prefix: Option<String>,
    /// Render-time callout rendering setting.
    pub callouts: Option<bool>,
    /// Module output only: module that exports `useMDXComponents`.
    pub provider_import_source: Option<String>,
    /// Module output only: source file name for the source map.
    pub filename: Option<String>,
    /// Module output only: export `MDXContent` as the default export.
    pub default_export: Option<bool>,
    /// Module output only: names the caller declares in code it adds.
    pub reserved_bindings: Option<Vec<String>>,
}

#[napi(object)]
#[derive(Clone)]
pub struct JsxModuleSource {
    pub value: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
#[derive(Clone)]
pub struct JsxCodeBlock {
    pub code: String,
    pub language: Option<String>,
    pub meta: Option<String>,
}

#[napi(object)]
pub struct JsxSourceMapping {
    pub generated_line: u32,
    pub generated_column: u32,
    pub source_line: u32,
    pub source_column: u32,
}

#[napi(object)]
#[derive(Clone)]
pub struct JsxHeading {
    pub level: u32,
    pub id: Option<String>,
    pub text: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
#[derive(Clone)]
pub struct SourceRange {
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
pub struct JsxResult {
    pub body: String,
    pub esm: Vec<JsxModuleSource>,
    pub components: Vec<String>,
    pub elements: Vec<String>,
    pub code_blocks: Vec<JsxCodeBlock>,
    pub headings: Vec<JsxHeading>,
    pub front_matter: Option<String>,
    pub front_matter_span: Option<SourceRange>,
    pub front_matter_kind: Option<String>,
    pub omitted_title_heading_span: Option<SourceRange>,
    pub mappings: Vec<JsxSourceMapping>,
}

/// Metadata available after preparation without rendering the JSX body.
#[napi(object)]
#[derive(Clone)]
pub struct JsxPreparedMetadata {
    pub esm: Vec<JsxModuleSource>,
    pub code_blocks: Vec<JsxCodeBlock>,
    /// Outline at preparation time, before render-time omission and ID choices.
    pub outline: Vec<JsxHeading>,
    pub front_matter: Option<String>,
    pub front_matter_span: Option<SourceRange>,
    pub front_matter_kind: Option<String>,
}

/// A version 3 source map for module code.
#[napi(object)]
pub struct JsxModuleMap {
    pub version: u32,
    pub sources: Vec<String>,
    pub sources_content: Vec<String>,
    pub names: Vec<String>,
    pub mappings: String,
}

#[napi(object)]
pub struct JsxModuleResult {
    pub code: String,
    pub map: JsxModuleMap,
    pub exports: Vec<String>,
    pub bindings: Vec<String>,
    pub code_blocks: Vec<JsxCodeBlock>,
    pub headings: Vec<JsxHeading>,
    pub front_matter: Option<String>,
    pub front_matter_span: Option<SourceRange>,
    pub front_matter_kind: Option<String>,
    pub omitted_title_heading_span: Option<SourceRange>,
}

/// What a compilation produces: the JSX body with its parts, or a module.
pub(crate) enum Compiled {
    Body(JsxResult),
    Module(JsxModuleResult),
}

impl Compiled {
    pub(crate) fn body(self) -> Result<JsxResult> {
        match self {
            Self::Body(result) => Ok(result),
            Self::Module(_) => Err(Error::new(
                Status::GenericFailure,
                "JSX body compilation produced a module",
            )),
        }
    }

    pub(crate) fn module(self) -> Result<JsxModuleResult> {
        match self {
            Self::Module(result) => Ok(result),
            Self::Body(_) => Err(Error::new(
                Status::GenericFailure,
                "JSX module compilation produced a body",
            )),
        }
    }
}

struct CodeHook<'callback, 'highlight> {
    callback: Option<CodeCallback<'callback>>,
    highlighting: Option<NativeHighlighting<'highlight>>,
    error: Option<Error>,
}

impl JsxRenderHooks for CodeHook<'_, '_> {
    fn render_code_block(&mut self, input: JsxCodeBlockInput<'_>) -> Option<String> {
        if self.error.is_some() {
            return None;
        }
        let callback = self.callback.as_ref()?;
        match callback.call(FnArgs::from((
            input.code.to_owned(),
            input.language.map(str::to_owned),
            input.meta.map(str::to_owned),
        ))) {
            Ok(output) => output,
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }

    fn highlight_code_block(
        &mut self,
        input: JsxCodeBlockInput<'_>,
    ) -> Option<JsxHighlightedCodeBlock> {
        if self.error.is_some() {
            return None;
        }
        let settings = self.highlighting.as_mut()?;
        let mut error = None;
        let mut on_error = |cause: &ferromark::ferriki::Error| {
            error = Some(Error::new(
                Status::GenericFailure,
                format!("Ferriki JSX highlighting: {cause}"),
            ));
        };
        let mut adapter = match settings.dark_theme {
            Some(dark) => FerrikiJsxHooks::with_light_dark_themes(
                settings.highlighter,
                settings.light_theme,
                dark,
            ),
            None => FerrikiJsxHooks::new(settings.highlighter, settings.light_theme),
        }
        .with_error_handler(&mut on_error);
        let result = adapter.highlight_code_block(input);
        self.error = error;
        result
    }
}

pub(crate) struct NativeHighlighting<'a> {
    pub highlighter: &'a mut Highlighter,
    pub light_theme: &'a str,
    pub dark_theme: Option<&'a str>,
    pub line_numbers: bool,
}

/// Render settings inherited from the old one-shot options object.
#[derive(Clone)]
pub(crate) struct JsxRenderDefaults {
    pub heading_ids: bool,
    pub heading_level_offset: i32,
    pub heading_id_prefix: String,
    pub callouts: bool,
}

struct JsxDocumentOwner {
    source: String,
    allocator: Allocator,
}

self_cell!(
    struct PreparedJsxTree {
        owner: JsxDocumentOwner,

        #[not_covariant]
        dependent: Document,
    }
);

pub(crate) struct ParsedJsxDocument {
    tree: PreparedJsxTree,
    defaults: JsxRenderDefaults,
}

impl ParsedJsxDocument {
    pub(crate) fn with_document<R>(&self, render: impl FnOnce(&str, &Document<'_>) -> R) -> R {
        self.tree
            .with_dependent(|owner, document| render(&owner.source, document))
    }

    pub(crate) fn metadata(&self) -> Result<JsxPreparedMetadata> {
        self.with_document(|_, document| prepared_metadata(document, &self.defaults))
    }

    pub(crate) fn render_body(
        &self,
        jsx: Option<JsxOptions>,
        render_code: Option<CodeCallback<'_>>,
        highlighting: Option<NativeHighlighting<'_>>,
    ) -> Result<JsxResult> {
        let jsx = jsx.unwrap_or_default();
        match self.with_document(|source, document| {
            render_parsed(
                document,
                source,
                &self.defaults,
                jsx,
                render_code,
                highlighting,
                false,
            )
        })? {
            Compiled::Body(result) => Ok(result),
            Compiled::Module(_) => Err(Error::new(
                Status::GenericFailure,
                "JSX body compilation produced a module",
            )),
        }
    }

    pub(crate) fn render_module(
        &self,
        jsx: Option<JsxOptions>,
        render_code: Option<CodeCallback<'_>>,
        highlighting: Option<NativeHighlighting<'_>>,
    ) -> Result<JsxModuleResult> {
        let jsx = jsx.unwrap_or_default();
        match self.with_document(|source, document| {
            render_parsed(
                document,
                source,
                &self.defaults,
                jsx,
                render_code,
                highlighting,
                true,
            )
        })? {
            Compiled::Module(result) => Ok(result),
            Compiled::Body(_) => Err(Error::new(
                Status::GenericFailure,
                "JSX module compilation produced a body",
            )),
        }
    }
}

pub(crate) fn prepare_jsx_document(
    markdown: Utf8Input,
    options: Option<Options>,
    format: Option<&str>,
) -> Result<ParsedJsxDocument> {
    let resolved = core_options(options)?;
    let defaults = render_defaults(&resolved);
    let tree = prepare_tree(markdown.to_string(), resolved, format)?;
    Ok(ParsedJsxDocument { tree, defaults })
}

fn render_defaults(options: &CoreOptions) -> JsxRenderDefaults {
    JsxRenderDefaults {
        heading_ids: options.html.heading_ids,
        heading_level_offset: options.heading_level_offset,
        heading_id_prefix: options.heading_id_prefix.clone(),
        callouts: options.html.callouts,
    }
}

fn prepare_tree(
    source: String,
    mut options: CoreOptions,
    format: Option<&str>,
) -> Result<PreparedJsxTree> {
    options.parser.mdx = match format {
        None | Some("md") => false,
        Some("mdx") => true,
        _ => {
            return Err(Error::new(
                Status::InvalidArg,
                "format must be 'md' or 'mdx'",
            ));
        }
    };
    options.parser.mdx_compatible = options.parser.mdx;
    let parser_options = options.parser;
    let html_options = options.html;
    let mut pipeline = options.pipeline;
    let owner = JsxDocumentOwner {
        allocator: Allocator::for_source_len(source.len()),
        source,
    };
    PreparedJsxTree::try_new(owner, move |owner| {
        let mut document = Parser::with_options(&owner.allocator, &owner.source, parser_options)
            .parse()
            .map_err(parse_error)?;
        let mut scope = ModuleScope::default();
        scope.visit_document(&document);
        if let Some(span) = scope.nested {
            return Err(Error::new(
                Status::InvalidArg,
                format!("MDX module declarations must appear at document top level (at {span:?})"),
            ));
        }
        if !pipeline.is_empty() {
            let context = TransformContext::new(&owner.allocator, &owner.source, &html_options);
            pipeline
                .run(&mut document, &context)
                .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;
        }
        Ok(document)
    })
}

fn component_name(value: &str) -> Result<()> {
    let valid = value.split('.').all(|part| {
        let mut chars = part.chars();
        chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    });
    if valid {
        Ok(())
    } else {
        Err(Error::new(
            Status::InvalidArg,
            "JSX component names must be dot-separated identifiers",
        ))
    }
}

fn component_map(values: Option<HashMap<String, String>>) -> Result<BTreeMap<String, String>> {
    values
        .unwrap_or_default()
        .into_iter()
        .map(|(key, value)| {
            component_name(&value)?;
            Ok((key, value))
        })
        .collect()
}

#[derive(Default)]
struct ModuleScope {
    depth: usize,
    nested: Option<Span>,
}

impl<'a> Visit<'a> for ModuleScope {
    fn visit_node(&mut self, node: &Node<'a>) {
        if self.depth > 0 && matches!(node, Node::MdxjsEsm(_)) {
            self.nested = Some(node.span());
        }
        self.depth += 1;
        ferromark::ast::walk_node(self, node);
        self.depth -= 1;
    }
}

/// Internal entry: the public facade validates and separates its option keys.
#[napi(catch_unwind)]
pub fn compile_jsx(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    options: Option<Options>,
    jsx_options: Option<JsxOptions>,
    #[napi(
        ts_arg_type = "(code: string, language?: string | null, meta?: string | null) => string | null | undefined"
    )]
    render_code: Option<CodeCallback<'_>>,
) -> Result<JsxResult> {
    compile_jsx_with_highlighter(markdown, options, jsx_options, render_code, None, false)?.body()
}

/// Internal entry for `output: "module"`; the public facade selects it.
#[napi(catch_unwind)]
pub fn compile_jsx_module(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    options: Option<Options>,
    jsx_options: Option<JsxOptions>,
    #[napi(
        ts_arg_type = "(code: string, language?: string | null, meta?: string | null) => string | null | undefined"
    )]
    render_code: Option<CodeCallback<'_>>,
) -> Result<JsxModuleResult> {
    compile_jsx_with_highlighter(markdown, options, jsx_options, render_code, None, true)?.module()
}

fn range(span: Span) -> SourceRange {
    SourceRange {
        start: span.start,
        end: span.end,
    }
}

fn code_blocks(blocks: Vec<ferromark::JsxCodeBlock>) -> Vec<JsxCodeBlock> {
    blocks
        .into_iter()
        .map(|item| JsxCodeBlock {
            code: item.value,
            language: item.language,
            meta: item.meta,
        })
        .collect()
}

fn headings(headings: Vec<ferromark::OutlineEntry>) -> Vec<JsxHeading> {
    headings
        .into_iter()
        .map(|item| JsxHeading {
            level: u32::from(item.level),
            id: item.id,
            text: item.text,
            start: item.span.start,
            end: item.span.end,
        })
        .collect()
}

#[derive(Default)]
struct CodeBlocksCollector {
    blocks: Vec<JsxCodeBlock>,
}

impl Visit<'_> for CodeBlocksCollector {
    fn visit_code_block(&mut self, code: &CodeBlock<'_>) {
        self.blocks.push(JsxCodeBlock {
            code: code.value.to_owned(),
            language: code.lang.map(str::to_owned),
            meta: code.meta.map(str::to_owned),
        });
    }
}

fn prepared_metadata(
    document: &Document<'_>,
    defaults: &JsxRenderDefaults,
) -> Result<JsxPreparedMetadata> {
    let mut code_blocks = CodeBlocksCollector::default();
    code_blocks.visit_document(document);
    let esm = document
        .children
        .iter()
        .filter_map(|node| match node {
            Node::MdxjsEsm(item) => Some(JsxModuleSource {
                value: item.value.to_owned(),
                start: item.span.start,
                end: item.span.end,
            }),
            _ => None,
        })
        .collect();
    let outline_options = OutlineOptions::default()
        .with_heading_ids(defaults.heading_ids)
        .with_heading_level_offset(defaults.heading_level_offset)
        .try_with_heading_id_prefix(defaults.heading_id_prefix.clone())
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
    let front = document.front_matter.as_ref();
    Ok(JsxPreparedMetadata {
        esm,
        code_blocks: code_blocks.blocks,
        outline: headings(document.outline(&outline_options)),
        front_matter: front.map(|front| front.value.to_owned()),
        front_matter_span: front.map(|front| range(front.span)),
        front_matter_kind: front.map(|front| match front.kind {
            ferromark::ast::FrontMatterKind::Yaml => "yaml".to_owned(),
            ferromark::ast::FrontMatterKind::Toml => "toml".to_owned(),
        }),
    })
}

pub(crate) fn compile_jsx_with_highlighter(
    markdown: Utf8Input,
    options: Option<Options>,
    jsx_options: Option<JsxOptions>,
    render_code: Option<CodeCallback<'_>>,
    highlighting: Option<NativeHighlighting<'_>>,
    module: bool,
) -> Result<Compiled> {
    let jsx = jsx_options.unwrap_or_default();
    let parsed = prepare_jsx_document(markdown, options, jsx.format.as_deref())?;
    if module {
        Ok(Compiled::Module(parsed.render_module(
            Some(jsx),
            render_code,
            highlighting,
        )?))
    } else {
        Ok(Compiled::Body(parsed.render_body(
            Some(jsx),
            render_code,
            highlighting,
        )?))
    }
}

fn heading_offset(value: f64) -> Result<i32> {
    if !value.is_finite()
        || value.fract() != 0.0
        || value < f64::from(i32::MIN)
        || value > f64::from(i32::MAX)
    {
        return Err(Error::new(
            Status::InvalidArg,
            "headingOffset must be an integer in the signed 32-bit range",
        ));
    }
    Ok(value as i32)
}

fn render_parsed(
    document: &Document<'_>,
    source: &str,
    defaults: &JsxRenderDefaults,
    jsx: JsxOptions,
    render_code: Option<CodeCallback<'_>>,
    highlighting: Option<NativeHighlighting<'_>>,
    module: bool,
) -> Result<Compiled> {
    if let Some(prefix) = &jsx.component_prefix {
        component_name(prefix)?;
    }
    if let Some(component) = &jsx.code_block_component {
        component_name(component)?;
    }
    let heading_id_prefix = match jsx.heading_id_prefix {
        Some(prefix) => {
            HtmlRenderer::validate_heading_id_prefix(&prefix)
                .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
            prefix
        }
        None => defaults.heading_id_prefix.clone(),
    };
    let heading_level_offset = jsx
        .heading_offset
        .map(heading_offset)
        .transpose()?
        .unwrap_or(defaults.heading_level_offset);
    let module_options = if module {
        if jsx.provider_import_source.as_deref() == Some("") {
            return Err(Error::new(
                Status::InvalidArg,
                "providerImportSource must be a nonempty string",
            ));
        }
        Some(JsxModuleOptions {
            provider_import_source: jsx.provider_import_source,
            default_export: jsx.default_export.unwrap_or(true),
            reserved_bindings: jsx.reserved_bindings.unwrap_or_default(),
            filename: jsx.filename,
        })
    } else {
        if jsx.provider_import_source.is_some()
            || jsx.filename.is_some()
            || jsx.default_export.is_some()
            || jsx.reserved_bindings.is_some()
        {
            return Err(Error::new(
                Status::InvalidArg,
                "MDX module options require output: 'module'",
            ));
        }
        None
    };
    let renderer = JsxRenderer::with_options(JsxRendererOptions {
        component_prefix: jsx.component_prefix,
        heading_ids: jsx.heading_ids.unwrap_or(defaults.heading_ids),
        heading_level_offset,
        heading_id_prefix,
        callouts: jsx.callouts.unwrap_or(defaults.callouts),
        omit_title_heading: jsx.omit_title_heading,
        callout_components: component_map(jsx.callout_components)?,
        code_block_components: component_map(jsx.code_components)?,
        code_block_component: jsx.code_block_component,
        show_line_numbers: highlighting
            .as_ref()
            .is_some_and(|settings| settings.line_numbers),
    });
    let front_matter = document
        .front_matter
        .as_ref()
        .map(|front| front.value.to_owned());
    let front_matter_span = document
        .front_matter
        .as_ref()
        .map(|front| range(front.span));
    let front_matter_kind = document
        .front_matter
        .as_ref()
        .map(|front| match front.kind {
            ferromark::ast::FrontMatterKind::Yaml => "yaml".to_owned(),
            ferromark::ast::FrontMatterKind::Toml => "toml".to_owned(),
        });
    let mut hook = CodeHook {
        callback: render_code,
        highlighting,
        error: None,
    };
    if let Some(module_options) = module_options {
        let result =
            renderer.render_module_with_hooks(document, source, &module_options, &mut hook);
        if let Some(error) = hook.error {
            return Err(error);
        }
        let output = result.map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
        return Ok(Compiled::Module(JsxModuleResult {
            code: output.code,
            map: JsxModuleMap {
                version: 3,
                sources: output.map.sources,
                sources_content: output.map.sources_content,
                names: Vec::new(),
                mappings: output.map.mappings,
            },
            exports: output.exports,
            bindings: output.bindings,
            code_blocks: code_blocks(output.code_blocks),
            headings: headings(output.headings),
            front_matter,
            front_matter_span,
            front_matter_kind,
            omitted_title_heading_span: output.omitted_title_heading.map(range),
        }));
    }
    let output = renderer.render_with_hooks(document, source, &mut hook);
    if let Some(error) = hook.error {
        return Err(error);
    }
    Ok(Compiled::Body(JsxResult {
        body: output.body,
        esm: output
            .esm
            .into_iter()
            .map(|item| JsxModuleSource {
                value: item.value,
                start: item.span.start,
                end: item.span.end,
            })
            .collect(),
        components: output.components,
        elements: output.elements,
        code_blocks: code_blocks(output.code_blocks),
        headings: headings(output.headings),
        front_matter,
        front_matter_span,
        front_matter_kind,
        omitted_title_heading_span: output.omitted_title_heading.map(range),
        mappings: output
            .mappings
            .into_iter()
            .map(|item| JsxSourceMapping {
                generated_line: item.generated_line,
                generated_column: item.generated_column,
                source_line: item.source_line,
                source_column: item.source_column,
            })
            .collect(),
    }))
}
