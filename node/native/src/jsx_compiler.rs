//! Reusable Ferriki ownership for the native JSX compiler.

use std::cell::RefCell;
use std::path::Path;

use ferromark::ferriki::{Highlighter, RemoteAssets, StandardAssetCatalogs};
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use serde_json::{Map, Value};

use crate::jsx::{
    Compiled, JsxModuleResult, JsxOptions, JsxResult, NativeHighlighting, RenderCodeBlockInput,
    RenderCodeBlockResult, compile_jsx_with_highlighter, render_code_block_with_highlighter,
};
use crate::{CodeCallback, Options, input::Utf8Input};

/// Owns a Ferriki highlighter, asset catalogs, and theme registrations across documents.
#[napi]
pub struct JsxCompiler {
    highlighter: RefCell<Highlighter>,
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
            highlighter: RefCell::new(highlighter),
            light_theme,
            dark_theme,
            line_numbers,
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

pub(crate) fn engine_error(error: ferromark::ferriki::Error) -> Error {
    Error::new(
        Status::GenericFailure,
        format!("Ferriki JSX highlighting: {error}"),
    )
}
