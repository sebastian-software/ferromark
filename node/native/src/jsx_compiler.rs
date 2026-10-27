//! Reusable native plugin ownership and immutable prepared JSX documents.

use std::{cell::RefCell, rc::Rc};

use napi::bindgen_prelude::Result;
use napi_derive::napi;
use serde_json::Value;

use crate::plugins::{NativePlugins, boolean, known_keys, object, usage};

use crate::jsx::{
    Compiled, JsxModuleResult, JsxOptions, JsxPreparedMetadata, JsxResult, NativeHighlighting,
    ParsedJsxDocument, RenderCodeBlockInput, RenderCodeBlockResult, compile_jsx_with_highlighter,
    prepare_jsx_document, render_code_block_with_highlighter,
};
use crate::{CodeCallback, Options, input::Utf8Input};

/// Preparation-only choices kept separate from JSX rendering options.
#[napi(object)]
#[derive(Default)]
pub struct PreparationOptions {
    pub format: Option<String>,
    /// Capture source reference provenance before native passes. Default: false.
    pub track_references: Option<bool>,
}

/// Owns a selected set of native code plugins across HTML and JSX documents.
#[napi]
pub struct CodeCompiler {
    plugins: Rc<RefCell<NativePlugins>>,
    line_numbers: bool,
}

#[napi]
impl CodeCompiler {
    #[napi(constructor, catch_unwind)]
    pub fn new(settings: String) -> Result<Self> {
        let value: Value = serde_json::from_str(&settings).map_err(usage)?;
        let settings = object(&value, "code compiler options")?;
        known_keys(settings, &["plugins", "lineNumbers"], "code compiler")?;
        let line_numbers = boolean(settings, "lineNumbers")?.unwrap_or(false);
        Ok(Self {
            plugins: Rc::new(RefCell::new(NativePlugins::from_config(
                settings.get("plugins"),
            )?)),
            line_numbers,
        })
    }

    /// Renders HTML without calling JavaScript for any code block.
    #[napi(catch_unwind)]
    pub fn to_html(
        &self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
        options: Option<Options>,
    ) -> Result<String> {
        Ok(self.render_html(markdown, options, false)?.html)
    }

    /// Renders HTML and extracts the same metadata as the one-shot transform.
    #[napi(catch_unwind)]
    pub fn transform(
        &self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
        options: Option<Options>,
    ) -> Result<crate::TransformResult> {
        self.render_html(markdown, options, true)
    }

    fn render_html(
        &self,
        markdown: Utf8Input,
        options: Option<Options>,
        collect_metadata: bool,
    ) -> Result<crate::TransformResult> {
        let mut plugins = self.plugins.try_borrow_mut().map_err(|_| {
            usage("A code compiler cannot be used recursively from its code callback")
        })?;
        let mut options = crate::core_options(options)?;
        if self.line_numbers {
            options.html.code_annotations = true;
            options.html.code_fence_metadata = true;
            options.html.code_annotation_syntax = ferromark::CodeAnnotationSyntax::VitePress;
        }
        options.html.code_annotation_default_line_numbers = self.line_numbers;
        let result = crate::render_document_with_hooks(
            markdown.try_as_str()?,
            options,
            Some(&mut *plugins),
            collect_metadata,
        );
        if let Some(error) = plugins.take_error() {
            return Err(error);
        }
        result
    }

    /// Parses source and runs native transforms once, returning an owned document handle.
    #[napi(catch_unwind)]
    pub fn prepare(
        &self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
        options: Option<Options>,
        preparation_options: Option<PreparationOptions>,
    ) -> Result<PreparedJsxDocument> {
        validate_preparation_options(options.as_ref())?;
        let preparation = preparation_options.unwrap_or_default();
        let parsed = prepare_jsx_document(
            markdown,
            options,
            preparation.format.as_deref(),
            preparation.track_references.unwrap_or(false),
        )?;
        let metadata = parsed.metadata()?;
        Ok(PreparedJsxDocument {
            parsed,
            highlighting: Some(PreparedHighlighting {
                plugins: Rc::clone(&self.plugins),
                line_numbers: self.line_numbers,
            }),
            metadata,
        })
    }

    /// Compiles source and reuses the initialized native plugins.
    #[napi(catch_unwind)]
    pub fn compile(
        &self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
        options: Option<Options>,
        jsx_options: Option<JsxOptions>,
        #[napi(
            ts_arg_type = "(code: string, language?: string | null, meta?: string | null) => string | null | undefined"
        )]
        render_code: Option<CodeCallback<'_>>,
    ) -> Result<JsxResult> {
        self.compile_with(markdown, options, jsx_options, render_code, false)?
            .body()
    }

    /// Compiles source to an MDX module; the public facade selects this entry.
    #[napi(catch_unwind)]
    pub fn compile_module(
        &self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
        options: Option<Options>,
        jsx_options: Option<JsxOptions>,
        #[napi(
            ts_arg_type = "(code: string, language?: string | null, meta?: string | null) => string | null | undefined"
        )]
        render_code: Option<CodeCallback<'_>>,
    ) -> Result<JsxModuleResult> {
        self.compile_with(markdown, options, jsx_options, render_code, true)?
            .module()
    }

    /// Renders one standalone code block with the compiler's native highlighting plugins.
    #[napi(catch_unwind)]
    pub fn render_code_block(&self, input: RenderCodeBlockInput) -> Result<RenderCodeBlockResult> {
        let mut plugins = self.plugins.try_borrow_mut().map_err(|_| {
            usage("A JSX compiler cannot be used recursively from its code callback")
        })?;
        render_code_block_with_highlighter(
            input,
            NativeHighlighting {
                plugins: &mut plugins,
                line_numbers: self.line_numbers,
            },
        )
    }

    fn compile_with(
        &self,
        markdown: Utf8Input,
        options: Option<Options>,
        jsx_options: Option<JsxOptions>,
        render_code: Option<CodeCallback<'_>>,
        module: bool,
    ) -> Result<Compiled> {
        let mut plugins = self.plugins.try_borrow_mut().map_err(|_| {
            usage("A JSX compiler cannot be used recursively from its code callback")
        })?;
        compile_jsx_with_highlighter(
            markdown,
            options,
            jsx_options,
            render_code,
            Some(NativeHighlighting {
                plugins: &mut plugins,
                line_numbers: self.line_numbers,
            }),
            module,
        )
    }
}

/// Immutable, source-owning prepared JSX document with repeatable render methods.
#[napi]
pub struct PreparedJsxDocument {
    pub(crate) parsed: ParsedJsxDocument,
    highlighting: Option<PreparedHighlighting>,
    metadata: JsxPreparedMetadata,
}

struct PreparedHighlighting {
    plugins: Rc<RefCell<NativePlugins>>,
    line_numbers: bool,
}

impl PreparedJsxDocument {
    pub(crate) fn without_highlighting(parsed: ParsedJsxDocument) -> Result<Self> {
        let metadata = parsed.metadata()?;
        Ok(Self {
            parsed,
            metadata,
            highlighting: None,
        })
    }
}

#[napi]
impl PreparedJsxDocument {
    /// Projects original static Markdown without rendering or highlighting.
    #[napi(catch_unwind)]
    pub fn project_markdown(
        &self,
        options: Option<crate::projection::ProjectMarkdownOptions>,
    ) -> Result<crate::projection::MarkdownProjectionResult> {
        crate::projection::project_markdown(&self.parsed, options)
    }

    /// Metadata computed from the prepared tree without rendering its body.
    #[napi(getter)]
    pub fn metadata(&self) -> JsxPreparedMetadata {
        self.metadata.clone()
    }

    /// Extracts static text and final heading sections without highlighting or rendering.
    #[napi(catch_unwind)]
    pub fn extract_text(
        &self,
        options: Option<crate::text::TextOptions>,
    ) -> Result<crate::text::TextExtraction> {
        self.parsed.extract_text(options)
    }

    /// Renders the prepared document as a JSX body.
    #[napi(catch_unwind)]
    pub fn render(
        &self,
        options: Option<JsxOptions>,
        #[napi(
            ts_arg_type = "(code: string, language?: string | null, meta?: string | null) => string | null | undefined"
        )]
        render_code: Option<CodeCallback<'_>>,
    ) -> Result<JsxResult> {
        let Some(settings) = &self.highlighting else {
            return self.parsed.render_body(options, render_code, None);
        };
        let mut plugins = settings.plugins.try_borrow_mut().map_err(|_| {
            usage("A JSX compiler cannot be used recursively from its code callback")
        })?;
        self.parsed.render_body(
            options,
            render_code,
            Some(NativeHighlighting {
                plugins: &mut plugins,
                line_numbers: settings.line_numbers,
            }),
        )
    }

    /// Renders the prepared document as a complete MDX module.
    #[napi(catch_unwind)]
    pub fn render_module(
        &self,
        options: Option<JsxOptions>,
        #[napi(
            ts_arg_type = "(code: string, language?: string | null, meta?: string | null) => string | null | undefined"
        )]
        render_code: Option<CodeCallback<'_>>,
    ) -> Result<JsxModuleResult> {
        let Some(settings) = &self.highlighting else {
            return self.parsed.render_module(options, render_code, None);
        };
        let mut plugins = settings.plugins.try_borrow_mut().map_err(|_| {
            usage("A JSX compiler cannot be used recursively from its code callback")
        })?;
        self.parsed.render_module(
            options,
            render_code,
            Some(NativeHighlighting {
                plugins: &mut plugins,
                line_numbers: settings.line_numbers,
            }),
        )
    }
}

pub(crate) fn validate_preparation_options(options: Option<&Options>) -> Result<()> {
    let Some(options) = options else {
        return Ok(());
    };
    macro_rules! reject {
        ($field:ident, $name:literal) => {
            if options.$field.is_some() {
                return Err(usage(format!(
                    "{0} cannot be used during JSX preparation; use a supported parser option or pass render choices to render",
                    $name
                )));
            }
        };
    }
    reject!(render_policy, "renderPolicy");
    reject!(allow_html, "allowHtml");
    reject!(table_colgroup, "tableColgroup");
    reject!(table_column_names, "tableColumnNames");
    reject!(disallowed_raw_html, "disallowedRawHtml");
    reject!(heading_ids, "headingIds");
    reject!(heading_offset, "headingOffset");
    reject!(heading_id_prefix, "headingIdPrefix");
    reject!(callouts, "callouts");
    reject!(link_base_path, "linkBasePath");
    reject!(auto_abbreviations, "autoAbbreviations");
    reject!(abbreviations, "abbreviations");
    reject!(preset, "preset");
    reject!(mdx, "mdx");
    Ok(())
}

/// Compatibility compiler for existing Ferriki-specific 3.x settings.
#[cfg(feature = "ferriki")]
#[napi]
pub struct JsxCompiler {
    compiler: CodeCompiler,
}

#[cfg(feature = "ferriki")]
#[napi]
impl JsxCompiler {
    #[napi(constructor, catch_unwind)]
    pub fn new(settings: String) -> Result<Self> {
        let value: Value = serde_json::from_str(&settings).map_err(usage)?;
        let settings = object(&value, "JSX compiler options")?;
        known_keys(
            settings,
            &["theme", "lineNumbers", "languages", "assets"],
            "JSX compiler",
        )?;
        let line_numbers = boolean(settings, "lineNumbers")?.unwrap_or(false);
        let mut options = settings.clone();
        options.remove("lineNumbers");
        let config = serde_json::json!({"plugins": [{"name": "ferriki", "options": options}], "lineNumbers": line_numbers});
        Ok(Self {
            compiler: CodeCompiler::new(config.to_string())?,
        })
    }

    #[napi(catch_unwind)]
    pub fn prepare(
        &self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
        options: Option<Options>,
        preparation_options: Option<PreparationOptions>,
    ) -> Result<PreparedJsxDocument> {
        self.compiler
            .prepare(markdown, options, preparation_options)
    }

    #[napi(catch_unwind)]
    pub fn compile(
        &self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
        options: Option<Options>,
        jsx_options: Option<JsxOptions>,
        #[napi(
            ts_arg_type = "(code: string, language?: string | null, meta?: string | null) => string | null | undefined"
        )]
        render_code: Option<CodeCallback<'_>>,
    ) -> Result<JsxResult> {
        self.compiler
            .compile(markdown, options, jsx_options, render_code)
    }

    #[napi(catch_unwind)]
    pub fn compile_module(
        &self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
        options: Option<Options>,
        jsx_options: Option<JsxOptions>,
        #[napi(
            ts_arg_type = "(code: string, language?: string | null, meta?: string | null) => string | null | undefined"
        )]
        render_code: Option<CodeCallback<'_>>,
    ) -> Result<JsxModuleResult> {
        self.compiler
            .compile_module(markdown, options, jsx_options, render_code)
    }

    #[napi(catch_unwind)]
    pub fn render_code_block(&self, input: RenderCodeBlockInput) -> Result<RenderCodeBlockResult> {
        self.compiler.render_code_block(input)
    }
}
