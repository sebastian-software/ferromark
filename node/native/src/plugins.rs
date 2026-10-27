//! Composition root for Rust plugins linked into this addon.
//!
//! No Rust objects or pointers cross addon boundaries. Add an optional crate
//! dependency and a factory here to build a custom plugin set. Plugin crates
//! implement Ferromark's engine-neutral HTML/JSX hooks; Node configuration
//! selects only factories compiled into this binary.

use ferromark::{
    CodeHighlightInput, HighlightedCodeBlock, HtmlCodeBlockInput, HtmlRenderHooks,
    JsxCodeBlockInput, JsxHighlightedCodeBlock, JsxRenderHooks,
};
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use serde_json::{Map, Value};

#[cfg(feature = "ferrigura")]
mod ferrigura;
#[cfg(feature = "ferriki")]
mod ferriki;

/// A native plugin failure with the original fence's UTF-8 source range.
#[napi(object)]
pub struct CodePluginDiagnostic {
    pub plugin: String,
    pub message: String,
    pub start: u32,
    pub end: u32,
}

/// Addon-local error reporting, separate from the engine-neutral render traits.
pub(crate) trait NativeCodePlugin: HtmlRenderHooks + JsxRenderHooks {
    fn take_error(&mut self) -> Option<Error> {
        None
    }

    fn take_diagnostics(&mut self) -> Vec<CodePluginDiagnostic> {
        Vec::new()
    }
}

#[cfg(feature = "plugin-test")]
impl NativeCodePlugin for ferromark_plugin_fixture::FixturePlugin {}

/// Owns initialized engines for one compiler and its prepared documents.
#[derive(Default)]
pub(crate) struct NativePlugins {
    plugins: Vec<Box<dyn NativeCodePlugin>>,
    error: Option<Error>,
    diagnostics: Vec<CodePluginDiagnostic>,
}

impl NativePlugins {
    pub(crate) fn from_config(config: Option<&Value>) -> Result<Self> {
        let mut plugins = Self::default();
        if let Some(config) = config {
            let entries = config
                .as_array()
                .ok_or_else(|| usage("plugins must be an array"))?;
            for entry in entries {
                let entry = object(entry, "plugin")?;
                known_keys(entry, &["name", "options"], "plugin")?;
                let name =
                    string(entry, "name")?.ok_or_else(|| usage("plugin.name is required"))?;
                let empty = Value::Object(Map::new());
                let options = entry.get("options").unwrap_or(&empty);
                object(options, "plugin options")?;
                let plugin = create_plugin(name, options)?;
                plugins.plugins.push(plugin);
            }
        }
        Ok(plugins)
    }

    pub(crate) fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }

    pub(crate) fn take_diagnostics(&mut self) -> Vec<CodePluginDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    fn first<T>(
        &mut self,
        mut render: impl FnMut(&mut dyn NativeCodePlugin) -> Option<T>,
    ) -> Option<T> {
        if self.error.is_some() {
            return None;
        }
        for plugin in &mut self.plugins {
            let output = render(plugin.as_mut());
            self.diagnostics.extend(plugin.take_diagnostics());
            self.error = plugin.take_error();
            if self.error.is_some() || output.is_some() {
                return output;
            }
        }
        None
    }
}

fn create_plugin(name: &str, options: &Value) -> Result<Box<dyn NativeCodePlugin>> {
    // Keep the option type checked even in an engine-free build.
    let _ = options;
    match name {
        #[cfg(feature = "ferriki")]
        "ferriki" => Ok(Box::new(ferriki::FerrikiPlugin::new(options)?)),
        #[cfg(feature = "ferrigura")]
        "ferrigura" => Ok(Box::new(ferrigura::FerriguraPlugin::new(options)?)),
        #[cfg(feature = "plugin-test")]
        "fixture" => {
            known_keys(object(options, "fixture options")?, &[], "fixture")?;
            Ok(Box::<ferromark_plugin_fixture::FixturePlugin>::default())
        }
        _ => Err(usage(format!(
            "Native code plugin '{name}' is not available in this addon build"
        ))),
    }
}

impl HtmlRenderHooks for NativePlugins {
    fn render_code_block(&mut self, input: HtmlCodeBlockInput<'_>) -> Option<String> {
        self.first(|plugin| HtmlRenderHooks::render_code_block(plugin, input))
    }

    fn highlight_code_block(
        &mut self,
        input: CodeHighlightInput<'_>,
    ) -> Option<HighlightedCodeBlock> {
        self.first(|plugin| HtmlRenderHooks::highlight_code_block(plugin, input))
    }
}

impl JsxRenderHooks for NativePlugins {
    fn render_code_block(&mut self, input: JsxCodeBlockInput<'_>) -> Option<String> {
        self.first(|plugin| JsxRenderHooks::render_code_block(plugin, input))
    }

    fn highlight_code_block(
        &mut self,
        input: JsxCodeBlockInput<'_>,
    ) -> Option<JsxHighlightedCodeBlock> {
        self.first(|plugin| JsxRenderHooks::highlight_code_block(plugin, input))
    }
}

/// Names of the factories compiled into this particular addon.
#[napi]
pub fn available_code_plugins() -> Vec<String> {
    let names: &[&str] = &[
        #[cfg(feature = "ferriki")]
        "ferriki",
        #[cfg(feature = "ferrigura")]
        "ferrigura",
        #[cfg(feature = "plugin-test")]
        "fixture",
    ];
    names.iter().map(|name| (*name).to_owned()).collect()
}

pub(crate) fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| usage(format!("{name} must be an object")))
}

pub(crate) fn known_keys(value: &Map<String, Value>, keys: &[&str], context: &str) -> Result<()> {
    for key in value.keys() {
        if !keys.contains(&key.as_str()) {
            return Err(usage(format!("Unknown {context} option '{key}'")));
        }
    }
    Ok(())
}

pub(crate) fn boolean(value: &Map<String, Value>, key: &str) -> Result<Option<bool>> {
    value
        .get(key)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| usage(format!("{key} must be a boolean")))
        })
        .transpose()
}

pub(crate) fn string<'a>(value: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>> {
    value
        .get(key)
        .map(|value| {
            value
                .as_str()
                .filter(|text| !text.is_empty())
                .ok_or_else(|| usage(format!("{key} must be a nonempty string")))
        })
        .transpose()
}

pub(crate) fn usage(message: impl ToString) -> Error {
    Error::new(Status::InvalidArg, message.to_string())
}
