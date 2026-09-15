use ferromark_ast::{Node, Span};
use smallvec::SmallVec;

use super::Parser;

mod remap;

/// Shared traversal for stripped sub-sources and root character replacement.
pub(in crate::parser) trait SpanMap {
    fn map_span(&self, span: Span) -> Span;

    fn map_inline_span(&self, span: Span) -> Span {
        self.map_span(span)
    }

    /// Maps the synthetic parser document span back to the caller's source.
    /// Most maps have no distinction between a document and node span; the
    /// root normalization map overrides this because it can strip a leading
    /// BOM or metadata block while the document still covers the full input.
    fn map_document_span(&self, span: Span) -> Span {
        self.map_span(span)
    }
}

#[derive(Debug, Default)]
pub(in crate::parser) struct SourceMap {
    lines: SmallVec<[SourceMapLine; 8]>,
}

#[derive(Debug, Clone, Copy)]
struct SourceMapLine {
    generated_start: usize,
    generated_end: usize,
    source_block_start: usize,
    source_start: usize,
    source_end: usize,
}

impl SourceMap {
    pub(in crate::parser) fn line_origins(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.lines
            .iter()
            .map(|line| (line.generated_start, line.source_block_start))
    }

    pub(in crate::parser) fn generated_line_start(&self, original: usize) -> Option<usize> {
        let index = self
            .lines
            .partition_point(|line| line.source_block_start < original);
        self.lines
            .get(index)
            .filter(|line| line.source_block_start == original)
            .map(|line| line.generated_start)
    }

    pub(in crate::parser) fn push_line(
        &mut self,
        generated_start: usize,
        generated_len: usize,
        source_start: usize,
        source_len: usize,
    ) {
        self.push_line_with_block_start(
            generated_start,
            generated_len,
            source_start,
            source_start,
            source_len,
        );
    }

    pub(in crate::parser) fn push_line_with_block_start(
        &mut self,
        generated_start: usize,
        generated_len: usize,
        source_block_start: usize,
        source_start: usize,
        source_len: usize,
    ) {
        self.lines.push(SourceMapLine {
            generated_start,
            generated_end: generated_start + generated_len,
            source_block_start,
            source_start,
            source_end: source_start + source_len,
        });
    }

    pub(in crate::parser) fn remap_node_spans<'a>(&self, node: &mut Node<'a>) {
        Parser::remap_node_spans(node, self);
    }
}

impl SpanMap for SourceMap {
    fn map_span(&self, span: Span) -> Span {
        self.map_with_indent(span, true)
    }

    fn map_inline_span(&self, span: Span) -> Span {
        self.map_with_indent(span, false)
    }
}

impl SourceMap {
    fn map_with_indent(&self, span: Span, include_indent: bool) -> Span {
        if self.lines.is_empty() {
            return span;
        }

        let start = self.map_start(span.start as usize, include_indent);
        let end = if span.start == span.end {
            start
        } else {
            self.map_end(span.end as usize)
        };
        Span::new(start, end)
    }
}

impl SourceMap {
    fn map_start(&self, generated: usize, include_indent: bool) -> u32 {
        let index = self
            .lines
            .partition_point(|line| generated >= line.generated_end);
        let Some(line) = self.lines.get(index).copied() else {
            return self.lines.last().map_or(generated, |line| line.source_end) as u32;
        };
        if generated == line.generated_start && !include_indent {
            line.source_start as u32
        } else {
            Self::map_inside(line, generated) as u32
        }
    }

    fn map_end(&self, generated: usize) -> u32 {
        let index = self
            .lines
            .partition_point(|line| generated > line.generated_end);
        let Some(line) = self.lines.get(index).copied() else {
            return self.lines.last().map_or(generated, |line| line.source_end) as u32;
        };
        Self::map_inside(line, generated) as u32
    }

    fn map_inside(line: SourceMapLine, generated: usize) -> usize {
        if generated == line.generated_start {
            return line.source_block_start;
        }
        let generated_delta = generated.saturating_sub(line.generated_start);
        let source_len = line.source_end.saturating_sub(line.source_start);
        line.source_start + generated_delta.min(source_len)
    }
}

/// Constant translation for nodes parsed from a borrowed sub-source.
struct OffsetMap(u32);

impl SpanMap for OffsetMap {
    fn map_span(&self, span: Span) -> Span {
        Span::new(span.start + self.0, span.end + self.0)
    }
}

impl<'a> Parser<'a> {
    pub(super) fn offset_node_spans(node: &mut Node<'a>, offset: u32) {
        Self::remap_node_spans(node, &OffsetMap(offset));
    }
}
