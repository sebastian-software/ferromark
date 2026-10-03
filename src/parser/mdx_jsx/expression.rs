//! MDX `{expression}` parse: flow, text, JSX comments, and prose braces.
//!
//! Source inside the braces is stored. Nothing is evaluated.

use crate::ast::{MdxFlowExpression, MdxTextExpression, Node, Span};

use super::Parser;
use super::scan;
use crate::parser::error::ParseResult;

impl<'a> Parser<'a> {
    /// Whether a flow `{expression}` stands at `at`, for the block-start
    /// probe that runs before the parse below.
    pub(in crate::parser) fn looks_like_mdx_flow_expression(&self, at: usize) -> bool {
        self.matching_brace_end(self.source, at)
            .is_some_and(|brace_end| scan::only_ws_until_eol(self.source.as_bytes(), brace_end))
    }

    /// Parses a block `{expression}` or `{/* comment */}` at the current line.
    ///
    /// Succeeds only when the closing `}` is followed by line whitespace.
    /// On failure the cursor is left unchanged.
    pub(in crate::parser) fn try_parse_mdx_flow_expression(
        &mut self,
        start: usize,
        trimmed_start: usize,
    ) -> ParseResult<Option<Node<'a>>> {
        if !self.options.mdx {
            return Ok(None);
        }
        // Same bound as the block-start check: without it every line that
        // opens a brace and never closes it walks the rest of the source.
        let source = self.source;
        if !self.has_closer_from(source, trimmed_start + 1, b'}') {
            #[cfg(feature = "jsx")]
            if self.options.mdx_compatible {
                return Err(super::super::mdx_compatible::invalid(
                    Span::new(trimmed_start as u32, source.len() as u32),
                    "unclosed JavaScript expression; expected a closing }",
                ));
            }
            return Ok(None);
        }
        let Some(brace_end) = self.matching_brace_end(source, trimmed_start) else {
            #[cfg(feature = "jsx")]
            if self.options.mdx_compatible {
                return Err(super::super::mdx_compatible::invalid(
                    Span::new(trimmed_start as u32, source.len() as u32),
                    "invalid JavaScript expression or missing closing }; MDX braces must contain a complete expression",
                ));
            }
            return Ok(None);
        };
        if !scan::only_ws_until_eol(self.source.as_bytes(), brace_end) {
            return Ok(None);
        }

        #[cfg(feature = "jsx")]
        if self.options.mdx_compatible {
            super::super::mdx_compatible::validate_child_expression(
                &source[trimmed_start + 1..brace_end - 1],
                Span::new(trimmed_start as u32, brace_end as u32),
            )?;
        }

        self.position = scan::after_trailing_line_ws(self.source.as_bytes(), brace_end);
        Ok(Some(Node::MdxFlowExpression(MdxFlowExpression {
            value: &source[trimmed_start + 1..brace_end - 1],
            span: Span::new(start as u32, self.position as u32),
        })))
    }

    /// Parses an inline `{expression}` or `{/* comment */}` at `pos`.
    ///
    /// Used for document-level prose and for JSX children.
    pub(in crate::parser) fn try_parse_mdx_text_expression(
        &self,
        content: &'a str,
        pos: usize,
        offset: usize,
    ) -> ParseResult<Option<(Node<'a>, usize)>> {
        if !self.options.mdx {
            return Ok(None);
        }
        // `skip_braces` only reports that nothing closed after walking to
        // the end of the content, so a run of unclosed braces would pay one
        // walk each. The guard settles the run that has no `}` behind it at
        // all; `matching_brace_end` settles the rest.
        if !self.has_closer_from(content, pos + 1, b'}') {
            #[cfg(feature = "jsx")]
            if self.options.mdx_compatible {
                return Err(super::super::mdx_compatible::invalid(
                    Span::new((offset + pos) as u32, (offset + content.len()) as u32),
                    "unclosed JavaScript expression; expected a closing }",
                ));
            }
            return Ok(None);
        }
        let Some(end) = self.matching_brace_end(content, pos) else {
            #[cfg(feature = "jsx")]
            if self.options.mdx_compatible {
                return Err(super::super::mdx_compatible::invalid(
                    Span::new((offset + pos) as u32, (offset + content.len()) as u32),
                    "invalid JavaScript expression or missing closing }; MDX braces must contain a complete expression",
                ));
            }
            return Ok(None);
        };
        #[cfg(feature = "jsx")]
        if self.options.mdx_compatible {
            super::super::mdx_compatible::validate_child_expression(
                &content[pos + 1..end - 1],
                Span::new((offset + pos) as u32, (offset + end) as u32),
            )?;
        }
        Ok(Some((
            Node::MdxTextExpression(MdxTextExpression {
                value: &content[pos + 1..end - 1],
                span: Span::new((offset + pos) as u32, (offset + end) as u32),
            }),
            end,
        )))
    }
}
