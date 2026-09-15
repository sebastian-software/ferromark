//! Small owned-output helpers around the arena parser and HTML renderer.
use crate::{Allocator, HtmlRenderer, HtmlRendererOptions, ParseResult, Parser, ParserOptions};

/// Converts Markdown to owned HTML with the default Rust parser and renderer.
///
/// The defaults preserve raw HTML. For untrusted input, use
/// [`to_html_with_options`] with [`HtmlRendererOptions::sanitize`] enabled.
///
/// # Errors
/// Returns the parser error when the input cannot be parsed within its limits.
///
/// ```
/// assert_eq!(ferromark::to_html("**Hello**")?, "<p><strong>Hello</strong></p>\n");
/// # Ok::<(), ferromark::ParseError>(())
/// ```
pub fn to_html(source: &str) -> ParseResult<String> {
    // Built through `HtmlRenderer::new` rather than
    // `to_html_with_options(.., HtmlRendererOptions::default())`: the owned
    // options struct heap-allocates every default string and the autolink
    // pattern list, which the renderer's internal form represents with static
    // data instead. The two configurations are the same values — see
    // `static_default_options_match_owned_defaults`.
    let arena = Allocator::for_source_len(source.len());
    let document = Parser::with_options(&arena, source, ParserOptions::default()).parse()?;
    Ok(HtmlRenderer::new().render(&document))
}

/// Converts Markdown to owned HTML with explicit syntax and output options.
///
/// Each call owns a temporary parser arena and renderer. The returned string
/// is independent of both the source and the arena.
///
/// # Errors
/// Returns the parser error when the input cannot be parsed within its limits.
///
/// ```
/// use ferromark::{to_html_with_options, HtmlRendererOptions, ParserOptions};
/// let html = to_html_with_options(
///     "==Important== <b>authored HTML</b>",
///     ParserOptions { highlight: true, ..ParserOptions::default() },
///     HtmlRendererOptions { sanitize: true, ..HtmlRendererOptions::default() },
/// )?;
/// assert!(html.contains("<mark>Important</mark>"));
/// assert!(html.contains("&lt;b&gt;authored HTML&lt;/b&gt;"));
/// # Ok::<(), ferromark::ParseError>(())
/// ```
pub fn to_html_with_options(
    source: &str,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> ParseResult<String> {
    let arena = Allocator::for_source_len(source.len());
    let document = Parser::with_options(&arena, source, parser_options).parse()?;
    Ok(HtmlRenderer::with_options(renderer_options).render(&document))
}

/// Appends rendered HTML to a string using the default Rust options.
///
/// Existing output is preserved. The defaults preserve raw HTML; use
/// [`to_html_into_with_options`] to select sanitization or syntax extensions.
///
/// # Errors
/// Returns a parser error and leaves `output` unchanged if parsing fails.
pub fn to_html_into(source: &str, output: &mut String) -> ParseResult<()> {
    // See `to_html` for why the default path skips the owned options struct.
    let arena = Allocator::for_source_len(source.len());
    let document = Parser::with_options(&arena, source, ParserOptions::default()).parse()?;
    output.push_str(HtmlRenderer::new().render_borrowed(&document));
    Ok(())
}

/// Appends rendered HTML to a string with explicit syntax and output options.
///
/// Reuses the caller's output string capacity. Each call owns its parser arena
/// and renderer, and heading and reference state belongs to that call's document.
///
/// # Errors
/// Returns a parser error and leaves `output` unchanged if parsing fails.
pub fn to_html_into_with_options(
    source: &str,
    output: &mut String,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
) -> ParseResult<()> {
    let arena = Allocator::for_source_len(source.len());
    let document = Parser::with_options(&arena, source, parser_options).parse()?;
    output.push_str(HtmlRenderer::with_options(renderer_options).render_borrowed(&document));
    Ok(())
}
