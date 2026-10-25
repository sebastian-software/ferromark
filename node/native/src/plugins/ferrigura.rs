//! Optional Mermaid factory; all rendering and SVG conversion stay in Rust.

use std::sync::atomic::{AtomicU64, Ordering};

use ferromark::{HtmlCodeBlockInput, HtmlRenderHooks, JsxCodeBlockInput, JsxRenderHooks};
use ferromark_ferrigura::{
    FerriguraHooks,
    ferrigura::{ColorScheme, Renderer},
};
use napi::bindgen_prelude::{Error, Result, Status};
use serde_json::Value;

use super::{NativeCodePlugin, known_keys, object, string, usage};

static NEXT_NAMESPACE: AtomicU64 = AtomicU64::new(1);

pub(super) struct FerriguraPlugin {
    hooks: FerriguraHooks,
    fallback_to_code: bool,
}

impl FerriguraPlugin {
    pub(super) fn new(value: &Value) -> Result<Self> {
        let settings = object(value, "Ferrigura options")?;
        known_keys(
            settings,
            &["config", "idPrefix", "colorScheme", "errorPolicy"],
            "Ferrigura",
        )?;
        let scheme = match string(settings, "colorScheme")? {
            None => None,
            Some("light") => Some(ColorScheme::Light),
            Some("dark") => Some(ColorScheme::Dark),
            Some("auto") => Some(ColorScheme::Auto),
            Some(_) => {
                return Err(usage(
                    "Ferrigura colorScheme must be 'light', 'dark', or 'auto'",
                ));
            }
        };
        let fallback_to_code = match string(settings, "errorPolicy")? {
            None | Some("throw") => false,
            Some("code") => true,
            Some(_) => return Err(usage("Ferrigura errorPolicy must be 'throw' or 'code'")),
        };
        let empty = Value::Object(serde_json::Map::new());
        let config = settings.get("config").unwrap_or(&empty);
        object(config, "Ferrigura config")?;
        let config = serde_json::from_value(config.clone()).map_err(usage)?;
        let prefix = if let Some(prefix) = string(settings, "idPrefix")? {
            prefix.to_owned()
        } else {
            let namespace = NEXT_NAMESPACE
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                    next.checked_add(1)
                })
                .map_err(|_| usage("Ferrigura output namespaces exhausted"))?;
            format!("fm-mermaid-{namespace}")
        };
        let mut hooks = FerriguraHooks::new(Renderer::new(&config), prefix).map_err(usage)?;
        if let Some(scheme) = scheme {
            hooks = hooks.with_color_scheme(scheme);
        }
        Ok(Self {
            hooks,
            fallback_to_code,
        })
    }
}

impl NativeCodePlugin for FerriguraPlugin {
    fn take_error(&mut self) -> Option<Error> {
        let error = self.hooks.take_error()?;
        (!self.fallback_to_code).then(|| Error::new(Status::GenericFailure, error.to_string()))
    }
}

impl HtmlRenderHooks for FerriguraPlugin {
    fn render_code_block(&mut self, input: HtmlCodeBlockInput<'_>) -> Option<String> {
        HtmlRenderHooks::render_code_block(&mut self.hooks, input)
    }
}

impl JsxRenderHooks for FerriguraPlugin {
    fn render_code_block(&mut self, input: JsxCodeBlockInput<'_>) -> Option<String> {
        JsxRenderHooks::render_code_block(&mut self.hooks, input)
    }
}
