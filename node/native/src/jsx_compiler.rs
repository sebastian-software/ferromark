//! Reusable Ferriki ownership for the native JSX compiler.

use std::path::Path;
use std::{cell::RefCell, rc::Rc};

use ferromark::ferriki::{Highlighter, RemoteAssets, StandardAssetCatalogs};
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use serde_json::{Map, Value};

use crate::jsx::{
    Compiled, JsxModuleResult, JsxOptions, JsxPreparedMetadata, JsxResult, NativeHighlighting,
    ParsedJsxDocument, RenderCodeBlockInput, RenderCodeBlockResult, compile_jsx_with_highlighter,
    prepare_jsx_document, render_code_block_with_highlighter,
};
use crate::{CodeCallback, Options, input::Utf8Input};

/// Owns a Ferriki highlighter, asset catalogs, and theme registrations across documents.
#[napi]
pub struct JsxCompiler {
    highlighter: Rc<RefCell<Highlighter>>,
    light_theme: String,
    dark_theme: Option<String>,
    line_numbers: bool,
}

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
        let mut highlighter = Highlighter::builder()
            .with_assets(asset_catalogs(settings.get("assets"))?)
            .build()
            .map_err(engine_error)?;
        if let Some(languages) = settings.get("languages") {
            let languages = languages
                .as_array()
                .ok_or_else(|| usage("languages must be an array of grammar registrations"))?;
            for language in languages {
                object(language, "language registration")?;
                highlighter
                    .register_language_json(&language.to_string())
                    .map_err(engine_error)?;
            }
        }
        let (light, dark) = theme_values(settings.get("theme"))?;
        let light_theme = register_theme(&mut highlighter, &light)?;
        let dark_theme = dark
            .as_ref()
            .map(|theme| register_theme(&mut highlighter, theme))
            .transpose()?;
        Ok(Self {
            highlighter: Rc::new(RefCell::new(highlighter)),
            light_theme,
            dark_theme,
            line_numbers,
        })
    }

    /// Parses source and runs native transforms once, returning an owned document handle.
    #[napi(catch_unwind)]
    pub fn prepare(
        &self,
        #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
        options: Option<Options>,
        jsx_options: Option<JsxOptions>,
    ) -> Result<PreparedJsxDocument> {
        validate_preparation_options(options.as_ref())?;
        let jsx = jsx_options.unwrap_or_default();
        if jsx.component_prefix.is_some()
            || jsx.callout_components.is_some()
            || jsx.code_components.is_some()
            || jsx.code_block_component.is_some()
            || jsx.omit_title_heading.is_some()
            || jsx.heading_ids.is_some()
            || jsx.heading_offset.is_some()
            || jsx.heading_id_prefix.is_some()
            || jsx.callouts.is_some()
            || jsx.provider_import_source.is_some()
            || jsx.filename.is_some()
            || jsx.default_export.is_some()
            || jsx.reserved_bindings.is_some()
        {
            return Err(usage(
                "prepare accepts only the JSX format; pass render choices to render or renderModule",
            ));
        }
        let parsed = prepare_jsx_document(markdown, options, jsx.format.as_deref())?;
        let metadata = parsed.metadata()?;
        Ok(PreparedJsxDocument {
            parsed,
            highlighter: Rc::clone(&self.highlighter),
            light_theme: self.light_theme.clone(),
            dark_theme: self.dark_theme.clone(),
            line_numbers: self.line_numbers,
            metadata,
        })
    }

    /// Compiles source and reuses the loaded native grammars and themes.
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

    /// Renders one standalone code block with the compiler's native highlighter.
    #[napi(catch_unwind)]
    pub fn render_code_block(&self, input: RenderCodeBlockInput) -> Result<RenderCodeBlockResult> {
        let mut highlighter = self.highlighter.try_borrow_mut().map_err(|_| {
            usage("A JSX compiler cannot be used recursively from its code callback")
        })?;
        render_code_block_with_highlighter(
            input,
            NativeHighlighting {
                highlighter: &mut highlighter,
                light_theme: &self.light_theme,
                dark_theme: self.dark_theme.as_deref(),
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
        let mut highlighter = self.highlighter.try_borrow_mut().map_err(|_| {
            usage("A JSX compiler cannot be used recursively from its code callback")
        })?;
        compile_jsx_with_highlighter(
            markdown,
            options,
            jsx_options,
            render_code,
            Some(NativeHighlighting {
                highlighter: &mut highlighter,
                light_theme: &self.light_theme,
                dark_theme: self.dark_theme.as_deref(),
                line_numbers: self.line_numbers,
            }),
            module,
        )
    }
}

/// Immutable, source-owning prepared JSX document with repeatable render methods.
#[napi]
pub struct PreparedJsxDocument {
    parsed: ParsedJsxDocument,
    highlighter: Rc<RefCell<Highlighter>>,
    light_theme: String,
    dark_theme: Option<String>,
    line_numbers: bool,
    metadata: JsxPreparedMetadata,
}

#[napi]
impl PreparedJsxDocument {
    /// Metadata computed from the prepared tree without rendering its body.
    #[napi(getter)]
    pub fn metadata(&self) -> JsxPreparedMetadata {
        self.metadata.clone()
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
        let mut highlighter = self.highlighter.try_borrow_mut().map_err(|_| {
            usage("A JSX compiler cannot be used recursively from its code callback")
        })?;
        self.parsed.render_body(
            options,
            render_code,
            Some(NativeHighlighting {
                highlighter: &mut highlighter,
                light_theme: &self.light_theme,
                dark_theme: self.dark_theme.as_deref(),
                line_numbers: self.line_numbers,
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
        let mut highlighter = self.highlighter.try_borrow_mut().map_err(|_| {
            usage("A JSX compiler cannot be used recursively from its code callback")
        })?;
        self.parsed.render_module(
            options,
            render_code,
            Some(NativeHighlighting {
                highlighter: &mut highlighter,
                light_theme: &self.light_theme,
                dark_theme: self.dark_theme.as_deref(),
                line_numbers: self.line_numbers,
            }),
        )
    }
}

fn asset_catalogs(value: Option<&Value>) -> Result<StandardAssetCatalogs> {
    let empty = Map::new();
    let options = value
        .map(|value| object(value, "assets"))
        .transpose()?
        .unwrap_or(&empty);
    known_keys(
        options,
        &["assetRoot", "remote", "baseUrl", "cacheDir", "commit"],
        "assets",
    )?;
    if let Some(root) = string(options, "assetRoot")? {
        if options.len() != 1 {
            return Err(usage(
                "assets.assetRoot cannot be combined with CDN/cache settings",
            ));
        }
        return StandardAssetCatalogs::load_from_root(Path::new(root)).map_err(engine_error);
    }
    let remote = RemoteAssets::default()
        .with_remote(boolean(options, "remote")?)
        .with_base_url(string(options, "baseUrl")?.map(str::to_owned))
        .with_cache_dir(string(options, "cacheDir")?.map(Into::into))
        .with_commit(string(options, "commit")?.map(str::to_owned));
    StandardAssetCatalogs::remote(remote).map_err(engine_error)
}

fn theme_values(theme: Option<&Value>) -> Result<(Value, Option<Value>)> {
    let Some(theme) = theme else {
        return Ok((
            Value::String("github-light-default".to_owned()),
            Some(Value::String("github-dark-default".to_owned())),
        ));
    };
    if let Some(pair) = theme.as_object()
        && (pair.contains_key("light") || pair.contains_key("dark"))
    {
        known_keys(pair, &["light", "dark"], "theme pair")?;
        return Ok((
            pair.get("light")
                .ok_or_else(|| usage("theme.light is required"))?
                .clone(),
            Some(
                pair.get("dark")
                    .ok_or_else(|| usage("theme.dark is required"))?
                    .clone(),
            ),
        ));
    }
    Ok((theme.clone(), None))
}

fn register_theme(highlighter: &mut Highlighter, theme: &Value) -> Result<String> {
    if let Some(name) = theme.as_str() {
        if !highlighter.load_theme(name).map_err(engine_error)? {
            return Err(usage(format!("Unknown Ferriki theme '{name}'")));
        }
        return Ok(name.to_owned());
    }
    let theme_object = object(theme, "theme registration")?;
    let name =
        string(theme_object, "name")?.ok_or_else(|| usage("A custom theme requires a name"))?;
    if let Some(include) = string(theme_object, "include")? {
        highlighter.load_theme(include).map_err(engine_error)?;
    }
    highlighter
        .register_theme_json(&theme.to_string())
        .map_err(engine_error)?;
    Ok(name.to_owned())
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| usage(format!("{name} must be an object")))
}

fn known_keys(value: &Map<String, Value>, keys: &[&str], context: &str) -> Result<()> {
    for key in value.keys() {
        if !keys.contains(&key.as_str()) {
            return Err(usage(format!("Unknown {context} option '{key}'")));
        }
    }
    Ok(())
}

fn boolean(value: &Map<String, Value>, key: &str) -> Result<Option<bool>> {
    value
        .get(key)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| usage(format!("{key} must be a boolean")))
        })
        .transpose()
}

fn string<'a>(value: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>> {
    value
        .get(key)
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| usage(format!("{key} must be a nonempty string")))
        })
        .transpose()
}

fn usage(message: impl ToString) -> Error {
    Error::new(Status::InvalidArg, message.to_string())
}

fn validate_preparation_options(options: Option<&Options>) -> Result<()> {
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

pub(crate) fn engine_error(error: ferromark::ferriki::Error) -> Error {
    Error::new(
        Status::GenericFailure,
        format!("Ferriki JSX highlighting: {error}"),
    )
}
