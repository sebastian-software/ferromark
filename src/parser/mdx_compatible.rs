//! Grammar validation for the opt-in MDX-to-JSX path.
//!
//! Oxc decides JavaScript boundaries. In particular, a `}` inside a regular
//! expression, comment or template cannot close an MDX expression merely
//! because a character scanner encountered it. JavaScript is retained as
//! source; no JavaScript AST escapes this module.

use crate::ast::Span;

use super::error::{MdxDiagnostic, MdxDiagnosticKind, ParseError, ParseErrorKind, ParseResult};
use super::line_scan::{line_end, line_terminator_end};

#[allow(
    clippy::disallowed_types,
    reason = "owned diagnostics cross the parser error API"
)]
pub(super) fn invalid(
    span: Span,
    kind: MdxDiagnosticKind,
    message: impl Into<String>,
) -> ParseError {
    ParseError::from(ParseErrorKind::InvalidMdx {
        span,
        message: message.into(),
    })
    .with_mdx_diagnostic(MdxDiagnostic {
        kind,
        span,
        related_span: None,
    })
}

/// The first closing brace whose interior is a complete JavaScript
/// expression (or a comment-only JSX expression). Candidate interiors are
/// parsed, never evaluated; source bytes remain borrowed from the document.
pub(super) fn expression_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let tail = source.get(start + 1..)?;
    let mut allocator = oxc_allocator::Allocator::default();
    // Fast path: when there is no `/` regex ambiguity inside a balanced brace
    // scan (strings, comments, and nested braces), jump straight to the
    // matching `}` first so strings/comments packed with `}` do not trigger
    // quadratic Oxc parses.
    if let Some(balanced_end) = balanced_expression_candidate(bytes, start) {
        let value = &source[start + 1..balanced_end - 1];
        if valid_expression(&allocator, value) {
            return Some(balanced_end);
        }
    }
    for relative in memchr::memchr_iter(b'}', tail.as_bytes()) {
        let end = start + 1 + relative;
        let value = &source[start + 1..end];
        allocator.reset();
        if valid_expression(&allocator, value) {
            return Some(end + 1);
        }
    }
    None
}

/// Finds the matching `}` for `{` at `start` by skipping strings, comments,
/// and nested `{...}` when no ambiguous `/` (potential regex literal) appears
/// before the closing brace.
fn balanced_expression_candidate(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'{') {
        return None;
    }
    let mut cursor = start + 1;
    let mut depth = 1u32;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' | b'\'' | b'`' => {
                cursor = super::mdx_jsx::braces::skip_quoted(bytes, cursor)?;
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor += 2;
                while cursor < bytes.len() && !matches!(bytes[cursor], b'\n' | b'\r') {
                    cursor += 1;
                }
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                let rel = memchr::memmem::find(&bytes[cursor + 2..], b"*/")?;
                cursor = cursor + 2 + rel + 2;
            }
            // A bare `/` might start a regex literal containing quotes or `}`.
            b'/' => return None,
            b'{' => {
                depth = depth.saturating_add(1);
                cursor += 1;
            }
            b'}' => {
                depth -= 1;
                cursor += 1;
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => cursor += 1,
        }
    }
    None
}

pub(super) fn validate_child_expression(value: &str, span: Span) -> ParseResult<()> {
    let allocator = oxc_allocator::Allocator::default();
    if oxc_parser::Parser::new(&allocator, value, oxc_span::SourceType::jsx())
        .parse_expression()
        .is_ok()
    {
        return Ok(());
    }
    // An empty/comment-only MDX expression is valid JSX. JSX spread
    // children are accepted by Oxc's JSX grammar, but MDX permits spreads
    // only in attributes, so require an actual JS expression here.
    let result = oxc_parser::Parser::new(&allocator, value, oxc_span::SourceType::jsx()).parse();
    if result.diagnostics.is_empty()
        && result.program.body.is_empty()
        && result.program.directives.is_empty()
    {
        Ok(())
    } else {
        Err(invalid(
            span,
            MdxDiagnosticKind::InvalidExpression,
            "spread syntax is only valid in JSX attributes; children must contain an expression",
        ))
    }
}

#[allow(
    clippy::disallowed_types,
    reason = "temporary JSX source crosses the JS parser input boundary"
)]
fn valid_expression(allocator: &oxc_allocator::Allocator, value: &str) -> bool {
    // JSX spread attributes contain `...expression`, unlike child/attribute
    // values. Validate the expression here; validating its opening tag later
    // enforces that a spread actually occurs in an attribute position.
    let mut synthetic = String::with_capacity(value.len() + 12);
    synthetic.push_str("<>{");
    synthetic.push_str(value);
    synthetic.push_str("}</>");
    // Include the candidate `}` in JSX syntax: parsing the interior alone
    // would allow EOF to terminate a // comment containing that brace.
    oxc_parser::Parser::new(allocator, &synthetic, oxc_span::SourceType::jsx())
        .parse_expression()
        .is_ok()
}

/// Validates a JSX opener with a synthetic empty body. Markdown children
/// remain Ferromark's responsibility, while Oxc validates JSX attributes,
/// spreads, identifier grammar and JavaScript attribute expressions.
#[allow(
    clippy::disallowed_types,
    reason = "temporary source is the JavaScript parser input boundary"
)]
pub(super) fn validate_open(source: &str, self_closing: bool, span: Span) -> ParseResult<()> {
    let allocator = oxc_allocator::Allocator::default();
    let mut synthetic = String::with_capacity(source.len() + 2);
    if self_closing {
        synthetic.push_str(source);
    } else if source == "<>" {
        synthetic.push_str("<></>");
    } else {
        synthetic.push_str(&source[..source.len() - 1]);
        synthetic.push_str("/>");
    }
    let result = oxc_parser::Parser::new(&allocator, &synthetic, oxc_span::SourceType::jsx())
        .parse_expression();
    match result {
        Ok(_) => Ok(()),
        Err(diagnostics) => Err(invalid(
            span,
            MdxDiagnosticKind::InvalidOpeningTag,
            diagnostics
                .first()
                .map_or("invalid JSX opening tag", |error| error.message.as_ref()),
        )),
    }
}

/// Reuse the JSX grammar for an unmatched closer before treating it as a
/// synchronization point. The name scanner alone also accepts `A..B`.
#[allow(clippy::disallowed_types)]
pub(super) fn valid_closing(source: &str) -> bool {
    if source == "</>" {
        return true;
    }
    let mut opener = String::with_capacity(source.len());
    opener.push('<');
    opener.push_str(&source[2..source.len() - 1]);
    opener.push_str("/>");
    validate_open(&opener, true, Span::empty()).is_ok()
}

/// Whether text that is not valid JavaScript is a valid TypeScript module.
fn is_typescript(allocator: &mut oxc_allocator::Allocator, value: &str) -> bool {
    allocator.reset();
    let result = oxc_parser::Parser::new(allocator, value, oxc_span::SourceType::tsx()).parse();
    result.diagnostics.is_empty() && !result.panicked && !result.program.body.is_empty()
}

/// MDX module blocks end at a blank line outside their JavaScript syntax.
/// Trying each blank-line boundary lets the JS parser distinguish a blank
/// line inside a template/function/import from the end of a module block.
pub(super) fn esm_end(source: &str, start: usize) -> ParseResult<usize> {
    let mut cursor = start;
    let mut allocator = oxc_allocator::Allocator::default();
    let mut first_error = None;
    let mut failed_boundaries = smallvec::SmallVec::<[usize; 4]>::new();
    loop {
        let end = line_end(source.as_bytes(), cursor);
        let next = line_terminator_end(source.as_bytes(), end);
        let at_boundary = cursor == source.len()
            || source.as_bytes()[cursor..end]
                .iter()
                .all(|byte| matches!(byte, b' ' | b'\t'));
        if at_boundary {
            allocator.reset();
            let result = oxc_parser::Parser::new(
                &allocator,
                &source[start..cursor],
                oxc_span::SourceType::jsx(),
            )
            .parse();
            if result.diagnostics.is_empty() && !result.panicked {
                let mut declarations_only = true;
                for statement in &result.program.body {
                    if !statement.is_module_declaration() {
                        declarations_only = false;
                        break;
                    }
                }
                if !result.program.body.is_empty()
                    && result.program.directives.is_empty()
                    && declarations_only
                {
                    return Ok(cursor);
                }
                return Err(invalid(
                    Span::new(start as u32, cursor as u32),
                    MdxDiagnosticKind::InvalidModule,
                    "MDX module blocks may contain only import/export declarations; separate Markdown with a blank line",
                ));
            }
            failed_boundaries.push(cursor);
            if first_error.is_none() {
                first_error = Some(invalid(
                    Span::new(start as u32, cursor as u32),
                    MdxDiagnosticKind::InvalidModule,
                    result
                        .diagnostics
                        .first()
                        .map_or("invalid import/export declaration", |error| {
                            error.message.as_ref()
                        }),
                ));
            }
        }
        if cursor == source.len() {
            // Only probe for TypeScript syntax when the module block failed to
            // close as JavaScript, instead of re-parsing TSX on every blank
            // line inside an incomplete multiline template or function.
            for boundary in failed_boundaries {
                if is_typescript(&mut allocator, &source[start..boundary]) {
                    return Err(invalid(
                        Span::new(start as u32, boundary as u32),
                        MdxDiagnosticKind::InvalidModule,
                        "TypeScript syntax is not supported in MDX module blocks; write JavaScript",
                    ));
                }
            }
            return Err(first_error.unwrap_or_else(|| {
                invalid(
                    Span::new(start as u32, cursor as u32),
                    MdxDiagnosticKind::InvalidModule,
                    "unterminated import/export declaration; separate Markdown with a blank line",
                )
            }));
        }
        cursor = next;
    }
}
