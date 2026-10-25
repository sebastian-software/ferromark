//! An optional, Node-independent adapter from Mermaid code blocks to inline SVG.
//!
//! Reuse one hook set across renders. Supply an ID prefix unique to the output
//! namespace; each recognized block gets a fresh suffix, including repeated
//! renders. Check [`FerriguraHooks::take_error`] after rendering. Unknown
//! languages decline the block and keep Ferromark's regular code rendering.

use ferromark::{HtmlCodeBlockInput, HtmlRenderHooks};

/// The engine's configuration and renderer types.
pub use ferrigura;

#[cfg(feature = "jsx")]
mod jsx;

/// Adapter initialization, rendering, or conversion failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The output namespace is not a valid SVG ID prefix.
    #[error(
        "idPrefix must start with an ASCII letter and contain only ASCII letters, digits, '-' and '_'"
    )]
    InvalidIdPrefix,
    /// All suffixes in this namespace have been used.
    #[error("Ferrigura diagram ID counter exhausted")]
    IdExhausted,
    /// Ferrigura rejected the diagram or its configuration.
    #[error("Ferrigura rendering failed: {0}")]
    Render(#[from] ferrigura::Error),
    /// The engine returned an SVG that cannot be converted to intrinsic JSX.
    #[cfg(feature = "jsx")]
    #[error("Ferrigura SVG conversion failed: {0}")]
    Svg(#[from] roxmltree::Error),
}

/// Owns and reuses one Ferrigura renderer for HTML and optional JSX hooks.
pub struct FerriguraHooks {
    renderer: ferrigura::Renderer,
    id_prefix: String,
    next_id: u64,
    color_scheme: Option<ferrigura::ColorScheme>,
    error: Option<Error>,
}

impl FerriguraHooks {
    /// Takes an initialized renderer and a caller-owned output namespace.
    ///
    /// # Errors
    /// Returns [`Error::InvalidIdPrefix`] for an invalid namespace.
    pub fn new(renderer: ferrigura::Renderer, id_prefix: impl Into<String>) -> Result<Self, Error> {
        let id_prefix = id_prefix.into();
        if !id_prefix
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic)
            || !id_prefix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(Error::InvalidIdPrefix);
        }
        Ok(Self {
            renderer,
            id_prefix,
            next_id: 1,
            color_scheme: None,
            error: None,
        })
    }

    /// Selects Ferrigura's light, dark, or automatic color scheme.
    #[must_use]
    pub const fn with_color_scheme(mut self, scheme: ferrigura::ColorScheme) -> Self {
        self.color_scheme = Some(scheme);
        self
    }

    /// Takes the first error since the previous check and permits later renders.
    #[must_use]
    pub fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }

    fn render_svg(&mut self, code: &str, language: Option<&str>) -> Option<String> {
        if self.error.is_some() || language != Some("mermaid") {
            return None;
        }
        let Some(next_id) = self.next_id.checked_add(1) else {
            self.error = Some(Error::IdExhausted);
            return None;
        };
        let id = format!("{}-{}", self.id_prefix, self.next_id);
        self.next_id = next_id;
        match self.renderer.render_with(
            code,
            &ferrigura::RenderOptions {
                id: Some(id),
                color_scheme: self.color_scheme,
                ..ferrigura::RenderOptions::default()
            },
        ) {
            Ok(mut svg) => {
                svg.push('\n');
                Some(svg)
            }
            Err(error) => {
                self.error = Some(error.into());
                None
            }
        }
    }
}

impl HtmlRenderHooks for FerriguraHooks {
    fn render_code_block(&mut self, input: HtmlCodeBlockInput<'_>) -> Option<String> {
        self.render_svg(input.code, input.language)
    }
}

#[cfg(feature = "jsx")]
impl ferromark::JsxRenderHooks for FerriguraHooks {
    fn render_code_block(&mut self, input: ferromark::JsxCodeBlockInput<'_>) -> Option<String> {
        let svg = self.render_svg(input.code, input.language)?;
        match jsx::convert(&svg) {
            Ok(jsx) => Some(jsx),
            Err(error) => {
                self.error = Some(error.into());
                None
            }
        }
    }
}
