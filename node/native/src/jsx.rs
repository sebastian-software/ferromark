//! Framework-neutral JSX compilation at the native binding boundary.

use std::collections::{BTreeMap, HashMap};

use ferromark::ast::{Node, Span, Visit};
use ferromark::ferriki::Highlighter;
use ferromark::{
    Allocator, FerrikiJsxHooks, JsxCodeBlockInput, JsxHighlightedCodeBlock, JsxModuleOptions,
    JsxRenderHooks, JsxRenderer, JsxRendererOptions, Parser,
};
use ferromark_transforms::TransformContext;
use napi::bindgen_prelude::{Error, FnArgs, Result, Status};
use napi_derive::napi;

use crate::{CodeCallback, Options, core_options, input::Utf8Input, parse_error};

#[napi(object)]
pub struct JsxOptions {
    pub format: Option<String>,
    pub component_prefix: Option<String>,
    pub callout_components: Option<HashMap<String, String>>,
    pub code_components: Option<HashMap<String, String>>,
    pub code_block_component: Option<String>,
    pub omit_title_heading: Option<String>,
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
pub struct JsxModuleSource {
    pub value: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
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
pub struct JsxHeading {
    pub level: u32,
    pub id: Option<String>,
    pub text: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
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

pub(crate) fn compile_jsx_with_highlighter(
    markdown: Utf8Input,
    options: Option<Options>,
    jsx_options: Option<JsxOptions>,
    render_code: Option<CodeCallback<'_>>,
    highlighting: Option<NativeHighlighting<'_>>,
    module: bool,
) -> Result<Compiled> {
    let jsx = jsx_options.unwrap_or(JsxOptions {
        format: None,
        component_prefix: None,
        callout_components: None,
        code_components: None,
        code_block_component: None,
        omit_title_heading: None,
        provider_import_source: None,
        filename: None,
        default_export: None,
        reserved_bindings: None,
    });
    if module && jsx.provider_import_source.as_deref() == Some("") {
        return Err(Error::new(
            Status::InvalidArg,
            "providerImportSource must be a nonempty string",
        ));
    }
    let module = module.then(|| JsxModuleOptions {
        provider_import_source: jsx.provider_import_source,
        default_export: jsx.default_export.unwrap_or(true),
        reserved_bindings: jsx.reserved_bindings.unwrap_or_default(),
        filename: jsx.filename,
    });
    let mut resolved = core_options(options)?;
    resolved.parser.mdx = match jsx.format.as_deref() {
        None | Some("md") => false,
        Some("mdx") => true,
        _ => {
            return Err(Error::new(
                Status::InvalidArg,
                "format must be 'md' or 'mdx'",
            ));
        }
    };
    resolved.parser.mdx_compatible = resolved.parser.mdx;
    if let Some(prefix) = &jsx.component_prefix {
        component_name(prefix)?;
    }
    if let Some(component) = &jsx.code_block_component {
        component_name(component)?;
    }
    let allocator = Allocator::for_source_len(markdown.len());
    let mut document = Parser::with_options(&allocator, &markdown, resolved.parser)
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
    if !resolved.pipeline.is_empty() {
        let context = TransformContext::new(&allocator, &markdown, &resolved.html);
        resolved
            .pipeline
            .run(&mut document, &context)
            .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;
    }
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
    let renderer = JsxRenderer::with_options(JsxRendererOptions {
        component_prefix: jsx.component_prefix,
        heading_ids: resolved.html.heading_ids,
        heading_level_offset: resolved.heading_level_offset,
        heading_id_prefix: resolved.heading_id_prefix,
        callouts: resolved.html.callouts,
        omit_title_heading: jsx.omit_title_heading,
        callout_components: component_map(jsx.callout_components)?,
        code_block_components: component_map(jsx.code_components)?,
        code_block_component: jsx.code_block_component,
        show_line_numbers: highlighting
            .as_ref()
            .is_some_and(|settings| settings.line_numbers),
    });
    let mut hook = CodeHook {
        callback: render_code,
        highlighting,
        error: None,
    };
    if let Some(module) = module {
        let result = renderer.render_module_with_hooks(&document, &markdown, &module, &mut hook);
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
    let output = renderer.render_with_hooks(&document, &markdown, &mut hook);
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
