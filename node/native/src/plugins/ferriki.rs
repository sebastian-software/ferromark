//! Optional Ferriki factory. The host and all other plugins stay engine-neutral.

use std::path::Path;

use ferromark::ferriki::{Highlighter, RemoteAssets, StandardAssetCatalogs};
use ferromark::{
    CodeHighlightInput, FerrikiHighlightHooks, FerrikiJsxHooks, HighlightedCodeBlock,
    HtmlRenderHooks, JsxCodeBlockInput, JsxHighlightedCodeBlock, JsxRenderHooks,
};
use napi::bindgen_prelude::{Error, Result, Status};
use serde_json::{Map, Value};

use super::{NativeCodePlugin, boolean, known_keys, object, string, usage};

pub(super) struct FerrikiPlugin {
    highlighter: Highlighter,
    light_theme: String,
    dark_theme: Option<String>,
    error: Option<Error>,
}

impl FerrikiPlugin {
    pub(super) fn new(value: &Value) -> Result<Self> {
        let settings = object(value, "Ferriki options")?;
        known_keys(settings, &["theme", "languages", "assets"], "Ferriki")?;
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
            highlighter,
            light_theme,
            dark_theme,
            error: None,
        })
    }
}

impl NativeCodePlugin for FerrikiPlugin {
    fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }
}

impl HtmlRenderHooks for FerrikiPlugin {
    fn highlight_code_block(
        &mut self,
        input: CodeHighlightInput<'_>,
    ) -> Option<HighlightedCodeBlock> {
        let mut error = None;
        let mut on_error = |cause: &ferromark::ferriki::Error| {
            error = Some(Error::new(
                Status::GenericFailure,
                format!("Ferriki HTML highlighting: {cause}"),
            ));
        };
        let result = FerrikiHighlightHooks::new(&mut self.highlighter, &self.light_theme)
            .with_error_handler(&mut on_error)
            .highlight_code_block(input);
        self.error = error;
        result
    }
}

impl JsxRenderHooks for FerrikiPlugin {
    fn highlight_code_block(
        &mut self,
        input: JsxCodeBlockInput<'_>,
    ) -> Option<JsxHighlightedCodeBlock> {
        let mut error = None;
        let mut on_error = |cause: &ferromark::ferriki::Error| {
            error = Some(engine_error(cause));
        };
        let mut adapter = match self.dark_theme.as_deref() {
            Some(dark) => FerrikiJsxHooks::with_light_dark_themes(
                &mut self.highlighter,
                &self.light_theme,
                dark,
            ),
            None => FerrikiJsxHooks::new(&mut self.highlighter, &self.light_theme),
        }
        .with_error_handler(&mut on_error);
        let output = adapter.highlight_code_block(input);
        self.error = error;
        output
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

fn engine_error(error: impl std::fmt::Display) -> Error {
    Error::new(
        Status::GenericFailure,
        format!("Ferriki JSX highlighting: {error}"),
    )
}
