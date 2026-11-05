//! The single JavaScript code-block channel shared by HTML and JSX rendering.
//!
//! The facade composes application code hooks into one dispatcher. Ferromark
//! calls it once per block for whole-block replacement (`stage: "render"`) and,
//! when no replacement handled the block, once more for line highlighting
//! (`stage: "highlight"`). The stages mirror the Rust render hooks: whole-block
//! replacements precede highlighting, and highlighting keeps Ferromark's code
//! block wrapper, annotations and component.

use ferromark::ast::Span;
use ferromark::{
    CodeHighlightInput, HighlightedCodeBlock, HtmlCodeBlockInput, HtmlRenderHooks,
    JsxCodeBlockInput, JsxHighlightedCodeBlock, html_to_jsx,
};
use napi::bindgen_prelude::{Either, Error, FnArgs, Function, Result, Status};
use napi_derive::napi;

/// The rendering stage and original location of one code block.
#[napi(object)]
pub struct CodeBlockStage {
    /// `"render"` for whole-block replacement, `"highlight"` for line fragments.
    pub stage: String,
    /// Start of the complete block as a UTF-8 byte offset into the source.
    pub start: u32,
    /// End of the complete block as a UTF-8 byte offset into the source.
    pub end: u32,
}

/// Object results of the code-block channel.
///
/// In the render stage, `html` replaces the block with trusted HTML; JSX output
/// converts it to JSX. In the highlight stage, `lines` holds one balanced HTML
/// fragment per code line, with optional block colors.
#[napi(object)]
pub struct CodeHookResult {
    pub html: Option<String>,
    pub lines: Option<Vec<String>>,
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub dark_foreground: Option<String>,
    pub dark_background: Option<String>,
}

pub(crate) type CodeCallbackResult = Option<Either<String, CodeHookResult>>;

pub(crate) type CodeCallback<'a> = Function<
    'a,
    FnArgs<(String, Option<String>, Option<String>, CodeBlockStage)>,
    CodeCallbackResult,
>;

/// Calls the dispatcher and keeps the first JavaScript error for the caller.
pub(crate) struct JsCodeHooks<'a> {
    callback: CodeCallback<'a>,
    error: Option<Error>,
}

impl<'a> JsCodeHooks<'a> {
    pub(crate) const fn new(callback: CodeCallback<'a>) -> Self {
        Self {
            callback,
            error: None,
        }
    }

    pub(crate) fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }

    fn call(
        &mut self,
        stage: &str,
        code: &str,
        language: Option<&str>,
        meta: Option<&str>,
        span: Span,
    ) -> CodeCallbackResult {
        if self.error.is_some() {
            return None;
        }
        let stage = CodeBlockStage {
            stage: stage.to_owned(),
            start: span.start,
            end: span.end,
        };
        match self.callback.call(FnArgs::from((
            code.to_owned(),
            language.map(str::to_owned),
            meta.map(str::to_owned),
            stage,
        ))) {
            Ok(result) => result,
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }

    fn fail<T>(&mut self, message: &str) -> Option<T> {
        self.error = Some(Error::new(Status::InvalidArg, message.to_owned()));
        None
    }

    /// Whole-block replacement for JSX output: JSX source, or HTML to convert.
    pub(crate) fn render_jsx(&mut self, input: JsxCodeBlockInput<'_>) -> Option<String> {
        match self.call("render", input.code, input.language, input.meta, input.span)? {
            Either::A(jsx) => Some(jsx),
            Either::B(object) => match object.html {
                Some(html) => Some(html_to_jsx(&html)),
                None => self.fail(RENDER_RESULT),
            },
        }
    }

    /// Line highlighting for JSX output.
    pub(crate) fn highlight_jsx(
        &mut self,
        input: JsxCodeBlockInput<'_>,
    ) -> Option<JsxHighlightedCodeBlock> {
        let object = self.highlight_object(input.code, input.language, input.meta, input.span)?;
        let lines = object.lines?;
        let mut block = JsxHighlightedCodeBlock::new(lines);
        block.foreground = object.foreground;
        block.background = object.background;
        block.dark_foreground = object.dark_foreground;
        block.dark_background = object.dark_background;
        Some(block)
    }

    fn highlight_object(
        &mut self,
        code: &str,
        language: Option<&str>,
        meta: Option<&str>,
        span: Span,
    ) -> Option<CodeHookResult> {
        match self.call("highlight", code, language, meta, span)? {
            Either::B(object) if object.lines.is_some() => Some(object),
            _ => self.fail(HIGHLIGHT_RESULT),
        }
    }
}

const RENDER_RESULT: &str =
    "code renderers must return a string, { html }, null, or undefined synchronously";
const HIGHLIGHT_RESULT: &str =
    "code highlighters must return { lines }, null, or undefined synchronously";

impl HtmlRenderHooks for JsCodeHooks<'_> {
    fn render_code_block(&mut self, input: HtmlCodeBlockInput<'_>) -> Option<String> {
        match self.call("render", input.code, input.language, input.meta, input.span)? {
            Either::A(html) => Some(html),
            Either::B(object) => match object.html {
                Some(html) => Some(html),
                None => self.fail(RENDER_RESULT),
            },
        }
    }

    fn highlight_code_block(
        &mut self,
        input: CodeHighlightInput<'_>,
    ) -> Option<HighlightedCodeBlock> {
        let object =
            self.highlight_object(input.code, input.language, input.raw_meta, input.span)?;
        let lines = object.lines?;
        let block = HighlightedCodeBlock::new(lines);
        Some(match (object.foreground, object.background) {
            (Some(foreground), Some(background)) => block.with_colors(foreground, background),
            _ => block,
        })
    }
}

/// Converts a pending callback error into the render result.
pub(crate) fn finish<T>(hooks: &mut JsCodeHooks<'_>, output: Result<T>) -> Result<T> {
    match hooks.take_error() {
        Some(error) => Err(error),
        None => output,
    }
}
