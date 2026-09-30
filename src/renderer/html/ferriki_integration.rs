//! Optional Ferriki adapter for the synchronous HTML render hook.

use ferriki::{Error, ErrorKind, Highlighter, RenderOptions};

use super::{CodeHighlightInput, HighlightedCodeBlock, HtmlRenderHooks};

/// Connects a reusable Ferriki highlighter to Ferromark's code-block renderer.
///
/// Build the highlighter before rendering and preload the languages and themes
/// needed when rendering must avoid lazy asset I/O. Ferriki owns asset loading
/// and caching; this adapter does not select or fetch an asset source. Reuse the
/// highlighter and renderer across documents to avoid repeated setup.
/// Missing languages and highlighting errors fall back to escaped plain code.
pub struct FerrikiHighlightHooks<'a> {
    highlighter: &'a mut Highlighter,
    theme: &'a str,
    render_options: RenderOptions,
    on_error: Option<&'a mut dyn FnMut(&Error)>,
}

impl<'a> FerrikiHighlightHooks<'a> {
    /// Borrows an initialized highlighter and selects a loaded theme.
    pub fn new(highlighter: &'a mut Highlighter, theme: &'a str) -> Self {
        Self {
            highlighter,
            theme,
            render_options: RenderOptions::default(),
            on_error: None,
        }
    }

    /// Sets Ferriki's options for rendering the highlighted line fragments.
    #[must_use]
    pub fn with_render_options(mut self, options: RenderOptions) -> Self {
        self.render_options = options;
        self
    }

    /// Observes Ferriki errors other than an unknown language before fallback.
    ///
    /// The callback runs synchronously during rendering. It must not retain the
    /// error reference, and a panic from the callback propagates to the caller.
    #[must_use]
    pub fn with_error_handler(mut self, handler: &'a mut dyn FnMut(&Error)) -> Self {
        self.on_error = Some(handler);
        self
    }
}

impl HtmlRenderHooks for FerrikiHighlightHooks<'_> {
    fn highlight_code_block(
        &mut self,
        input: CodeHighlightInput<'_>,
    ) -> Option<HighlightedCodeBlock> {
        let language = input.language?.to_ascii_lowercase();
        match self.highlighter.highlight_html_lines(
            input.code,
            &language,
            self.theme,
            &self.render_options,
        ) {
            Ok(highlighted) => Some(
                HighlightedCodeBlock::new(highlighted.lines)
                    .with_colors(highlighted.foreground, highlighted.background),
            ),
            Err(error) => {
                if error.kind() != ErrorKind::UnknownLanguage
                    && let Some(handler) = self.on_error.as_mut()
                {
                    handler(&error);
                }
                None
            }
        }
    }
}
