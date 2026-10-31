//! Optional Ferrigura integration: Mermaid code blocks become inline SVG.
//!
//! IDs derive from the namespace and original fence position, so repeated renders
//! are identical. Failed or unsafe diagrams retain ordinary code output; drain
//! [`FerriguraHooks::take_diagnostics`] to report their original source ranges.

use std::fmt::Write;

use crate::ast::Span;
use crate::renderer::{HtmlCodeBlockInput, HtmlRenderHooks};

mod svg;

#[cfg(feature = "jsx")]
mod jsx;

/// Adapter initialization, rendering, or conversion failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FerriguraError {
    /// The output namespace is not a valid SVG ID prefix.
    #[error(
        "idPrefix must start with an ASCII letter and contain only ASCII letters, digits, '-' and '_'"
    )]
    InvalidIdPrefix,
    /// Ferrigura rejected the diagram or its configuration.
    #[error("Ferrigura rendering failed: {0}")]
    Render(#[from] ferrigura::Error),
    /// The engine returned invalid XML.
    #[error("Ferrigura SVG parsing failed: {0}")]
    Svg(#[from] roxmltree::Error),
    /// SVG violates the static, local-resource output policy.
    #[error("Ferrigura SVG policy rejected {0}")]
    UnsafeSvg(String),
}

/// A failed Mermaid fence that fell back to ordinary code rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FerriguraDiagnostic {
    /// Original UTF-8 range of the complete fence.
    pub span: Span,
    /// Rendering or SVG policy failure.
    pub message: String,
}

/// Owns and reuses one Ferrigura renderer for HTML and optional JSX hooks.
pub struct FerriguraHooks {
    renderer: ferrigura::Renderer,
    id_prefix: String,
    color_scheme: Option<ferrigura::ColorScheme>,
    error: Option<FerriguraError>,
    diagnostics: Vec<FerriguraDiagnostic>,
}

impl FerriguraHooks {
    /// Takes an initialized renderer and a caller-owned output namespace.
    ///
    /// # Errors
    /// Returns [`FerriguraError::InvalidIdPrefix`] for an invalid namespace.
    pub fn new(
        renderer: ferrigura::Renderer,
        id_prefix: impl Into<String>,
    ) -> Result<Self, FerriguraError> {
        let id_prefix = id_prefix.into();
        if !id_prefix
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic)
            || !id_prefix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(FerriguraError::InvalidIdPrefix);
        }
        Ok(Self {
            renderer,
            id_prefix,
            color_scheme: Some(ferrigura::ColorScheme::Auto),
            error: None,
            diagnostics: Vec::new(),
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
    pub fn take_error(&mut self) -> Option<FerriguraError> {
        self.error.take()
    }

    /// Drains failed-fence diagnostics, preserving their encounter order.
    pub fn take_diagnostics(&mut self) -> Vec<FerriguraDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    fn diagram_id(&self, span: Span) -> String {
        let mut id = String::with_capacity(self.id_prefix.len() + 21);
        id.push_str(&self.id_prefix);
        let _ = write!(id, "-{}", span.start);
        id
    }

    fn render_svg(&self, code: &str, span: Span) -> Result<String, FerriguraError> {
        let svg = self.renderer.render_with(
            code,
            &ferrigura::RenderOptions {
                id: Some(self.diagram_id(span)),
                color_scheme: self.color_scheme,
                // Gantt's implicit dates and today marker must not read the clock.
                now: Some(0),
                ..ferrigura::RenderOptions::default()
            },
        )?;
        let mut svg = svg::sanitize(&svg)?;
        svg.push('\n');
        Ok(svg)
    }

    fn record(&mut self, span: Span, error: FerriguraError) {
        self.diagnostics.push(FerriguraDiagnostic {
            span,
            message: error.to_string(),
        });
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn render(
        &mut self,
        code: &str,
        language: Option<&str>,
        span: Span,
        jsx: bool,
    ) -> Option<String> {
        if language != Some("mermaid") {
            return None;
        }
        let result = self.render_svg(code, span).and_then(|svg| {
            #[cfg(feature = "jsx")]
            if jsx {
                return jsx::convert(&svg).map_err(FerriguraError::from);
            }
            let _ = jsx;
            Ok(svg)
        });
        match result {
            Ok(output) => Some(output),
            Err(error) => {
                self.record(span, error);
                None
            }
        }
    }
}

impl HtmlRenderHooks for FerriguraHooks {
    fn render_code_block(&mut self, input: HtmlCodeBlockInput<'_>) -> Option<String> {
        self.render(input.code, input.language, input.span, false)
    }
}

#[cfg(feature = "jsx")]
impl crate::renderer::JsxRenderHooks for FerriguraHooks {
    fn render_code_block(
        &mut self,
        input: crate::renderer::JsxCodeBlockInput<'_>,
    ) -> Option<String> {
        self.render(input.code, input.language, input.span, true)
    }
}
