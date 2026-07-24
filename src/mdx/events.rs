//! Opt-in semantic MDX event stream.
//!
//! This module composes the existing MDX segmenter, Markdown block parser, and
//! MDX-aware inline parser. It does not participate in the default HTML
//! rendering path.

use crate::block::CodeBlockKind;
use crate::{
    BlockEvent, BlockParser, InlineEvent, InlineParser, LinkRefDef, LinkRefStore, Options, Range,
    fixup_list_tight,
};

use super::{MdxDiagnostic, Segment, segment_spanned};

/// Version of the public MDX event ordering and balancing contract.
///
/// Consumers that persist or exchange event data can record this value
/// alongside their derived representation.
pub const MDX_EVENT_STREAM_VERSION: u16 = 1;

/// A semantic event in an MDX document.
///
/// Flow-level MDX and ESM events appear between balanced Markdown block
/// streams. Within a Markdown block, [`BlockEvent::Text`] placeholders are
/// replaced by their resolved [`InlineEvent`] sequence so source text is not
/// emitted twice.
///
/// Ranges embedded in any event are absolute byte ranges into the original
/// input passed to [`parse_events`](super::parse_events).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdxEvent {
    /// Front matter including its opening and closing delimiter lines.
    FrontMatter {
        /// Full front matter range, including delimiters.
        range: Range,
        /// Front matter payload between the delimiter lines.
        content: Range,
    },
    /// Root-level ESM statement.
    Esm(Range),
    /// Flow-level JavaScript expression.
    FlowExpression(Range),
    /// Flow-level JSX opening tag.
    FlowJsxOpen(Range),
    /// Flow-level JSX closing tag.
    FlowJsxClose(Range),
    /// Flow-level self-closing JSX tag.
    FlowJsxSelfClose(Range),
    /// Markdown block structure or block-owned source.
    Block(BlockEvent),
    /// Resolved inline Markdown or inline MDX structure.
    Inline(InlineEvent),
}

impl MdxEvent {
    /// Return the event's primary source range when it carries one.
    ///
    /// For container and formatting boundary events the compact underlying
    /// parser representation intentionally carries no delimiter range, so this
    /// method returns `None`. For a fenced code-block start it returns the info
    /// string, when present. For links, images, autolinks, and math, the primary
    /// range has the same meaning as on the embedded [`InlineEvent`].
    #[must_use]
    pub fn source_range(&self) -> Option<Range> {
        match self {
            Self::FrontMatter { range, .. }
            | Self::Esm(range)
            | Self::FlowExpression(range)
            | Self::FlowJsxOpen(range)
            | Self::FlowJsxClose(range)
            | Self::FlowJsxSelfClose(range) => Some(*range),
            Self::Block(event) => block_event_range(event),
            Self::Inline(event) => inline_event_range(event),
        }
    }
}

/// Collected, flat semantic events for one MDX document.
///
/// Events are ordered in source order. Markdown block start/end events and
/// inline formatting start/end events retain the balancing guarantees of the
/// existing parsers. Each Markdown segment is closed before the following
/// flow-level MDX or ESM event.
///
/// A paragraph whose physical source is contiguous is inline-parsed as one
/// unit, including line endings. When Markdown container prefixes make its
/// logical content non-contiguous, each source-contiguous line is inline-parsed
/// independently so every emitted range remains exact. Same-line JSX and
/// expressions are still recognized in those containers; cross-line container
/// MDX remains Markdown recovery.
#[derive(Debug)]
pub struct MdxEventStream {
    /// Version of this event contract.
    pub version: u16,
    /// Ordered semantic events.
    pub events: Vec<MdxEvent>,
    /// Resolved reference-link definitions used by `LinkStartRef` and
    /// `ImageStartRef` inline events.
    pub link_references: Vec<LinkRefDef>,
}

impl MdxEventStream {
    /// Resolve a reference-link definition by the index carried in an inline
    /// reference event.
    #[must_use]
    pub fn link_reference(&self, index: u32) -> Option<&LinkRefDef> {
        self.link_references.get(index as usize)
    }
}

/// Build the permissive semantic event stream for an MDX document.
///
/// Malformed flow or inline MDX follows the existing permissive recovery
/// contract and remains Markdown text. Use
/// [`parse_events_strict`](super::parse_events_strict) when structural
/// diagnostics are required.
#[must_use]
pub fn parse_events(input: &str) -> MdxEventStream {
    assert!(
        u32::try_from(input.len()).is_ok(),
        "MDX input exceeds the supported u32 source range"
    );

    let (content_start, front_matter) = front_matter_event(input);
    let content = &input[content_start..];
    let segments = segment_spanned(content);
    build_event_stream(input, content_start, front_matter, segments)
}

/// Build a strict semantic MDX event stream.
///
/// On structurally valid input this produces the same event ordering as
/// [`parse_events`]. When strict validation finds malformed flow MDX, no
/// partial event stream is returned; all available diagnostics are returned
/// instead. Front matter is excluded from structural MDX validation and every
/// diagnostic range is translated back to an absolute input range.
///
/// JavaScript and TypeScript syntax inside well-delimited constructs remains
/// outside this validator's scope. The existing strict validator covers flow
/// constructs; malformed inline MDX retains the permissive `InlineEvent::Text`
/// recovery in both event modes.
pub fn parse_events_strict(input: &str) -> Result<MdxEventStream, Vec<MdxDiagnostic>> {
    assert!(
        u32::try_from(input.len()).is_ok(),
        "MDX input exceeds the supported u32 source range"
    );

    let (content_start, front_matter) = front_matter_event(input);
    match super::segment_strict(&input[content_start..]) {
        Ok(segments) => Ok(build_event_stream(
            input,
            content_start,
            front_matter,
            segments,
        )),
        Err(mut diagnostics) => {
            for diagnostic in &mut diagnostics {
                diagnostic.primary_range = offset_range(diagnostic.primary_range, content_start);
                diagnostic.related_range = diagnostic
                    .related_range
                    .map(|range| offset_range(range, content_start));
            }
            Err(diagnostics)
        }
    }
}

fn build_event_stream(
    input: &str,
    content_start: usize,
    front_matter: Option<MdxEvent>,
    segments: Vec<super::SpannedSegment<'_>>,
) -> MdxEventStream {
    let mut stream = MdxEventStream {
        version: MDX_EVENT_STREAM_VERSION,
        events: Vec::with_capacity((input.len() / 12).max(32)),
        link_references: Vec::new(),
    };

    if let Some(event) = front_matter {
        stream.events.push(event);
    }

    let (link_refs, markdown_block_events) = parse_markdown_segments(&segments);
    link_refs.append_definitions_to(&mut stream.link_references);
    let mut markdown_block_events = markdown_block_events.into_iter();

    for spanned in segments {
        let segment_start = content_start + spanned.range.start_usize();
        match spanned.segment {
            Segment::Esm(_) => stream
                .events
                .push(MdxEvent::Esm(offset_range(spanned.range, content_start))),
            Segment::Markdown(markdown) => {
                let block_events = markdown_block_events
                    .next()
                    .expect("every Markdown segment must have parsed block events");
                emit_markdown_events(
                    markdown,
                    segment_start,
                    &link_refs,
                    block_events,
                    &mut stream.events,
                );
            }
            Segment::JsxBlockOpen(_) => stream.events.push(MdxEvent::FlowJsxOpen(offset_range(
                spanned.range,
                content_start,
            ))),
            Segment::JsxBlockClose(_) => stream.events.push(MdxEvent::FlowJsxClose(offset_range(
                spanned.range,
                content_start,
            ))),
            Segment::JsxBlockSelfClose(_) => stream.events.push(MdxEvent::FlowJsxSelfClose(
                offset_range(spanned.range, content_start),
            )),
            Segment::Expression(_) => stream.events.push(MdxEvent::FlowExpression(offset_range(
                spanned.range,
                content_start,
            ))),
        }
    }
    debug_assert!(markdown_block_events.next().is_none());

    stream
}

fn front_matter_event(input: &str) -> (usize, Option<MdxEvent>) {
    let Some((content, rest_offset)) = crate::extract_front_matter(input) else {
        return (0, None);
    };

    let input_start = input.as_ptr() as usize;
    let content_start = (content.as_ptr() as usize)
        .checked_sub(input_start)
        .expect("front matter must borrow from its input");
    let content_end = content_start + content.len();

    (
        rest_offset,
        Some(MdxEvent::FrontMatter {
            range: Range::from_usize(0, rest_offset),
            content: Range::from_usize(content_start, content_end),
        }),
    )
}

fn semantic_options() -> Options {
    Options {
        allow_html: false,
        front_matter: false,
        ..Options::default()
    }
}

fn parse_markdown_segments(
    segments: &[super::SpannedSegment<'_>],
) -> (LinkRefStore, Vec<Vec<BlockEvent>>) {
    let mut link_refs = LinkRefStore::new();
    let mut parsed = Vec::new();

    for segment in segments {
        let Segment::Markdown(markdown) = segment.segment else {
            continue;
        };

        let mut parser = BlockParser::new_with_options(markdown.as_bytes(), semantic_options());
        let mut events = Vec::new();
        parser.parse(&mut events);
        fixup_list_tight(&mut events);
        link_refs.merge_first_wins(parser.take_link_refs());
        parsed.push(events);
    }

    (link_refs, parsed)
}

fn emit_markdown_events(
    markdown: &str,
    source_offset: usize,
    link_refs: &LinkRefStore,
    block_events: Vec<BlockEvent>,
    events: &mut Vec<MdxEvent>,
) {
    let mut inline_parser = InlineParser::new();
    let mut inline_events = Vec::new();
    let mut inline_group = Vec::new();
    for block_event in block_events {
        match block_event {
            BlockEvent::Text(range) => {
                inline_group.push(InlineSourcePart::Text(range));
            }
            BlockEvent::SoftBreak => {
                inline_group.push(InlineSourcePart::SoftBreak);
            }
            event => {
                flush_inline_group(
                    markdown,
                    source_offset,
                    link_refs,
                    &mut inline_parser,
                    &mut inline_events,
                    &mut inline_group,
                    events,
                );
                events.push(MdxEvent::Block(offset_block_event(event, source_offset)));
            }
        }
    }
    flush_inline_group(
        markdown,
        source_offset,
        link_refs,
        &mut inline_parser,
        &mut inline_events,
        &mut inline_group,
        events,
    );
}

#[derive(Debug, Clone, Copy)]
enum InlineSourcePart {
    Text(Range),
    SoftBreak,
}

#[allow(clippy::too_many_arguments)]
fn flush_inline_group(
    markdown: &str,
    source_offset: usize,
    link_refs: &LinkRefStore,
    inline_parser: &mut InlineParser,
    inline_events: &mut Vec<InlineEvent>,
    parts: &mut Vec<InlineSourcePart>,
    events: &mut Vec<MdxEvent>,
) {
    if parts.is_empty() {
        return;
    }

    if let Some(range) = contiguous_inline_range(markdown.as_bytes(), parts) {
        emit_inline_range(
            range,
            markdown.as_bytes(),
            source_offset,
            link_refs,
            inline_parser,
            inline_events,
            events,
        );
    } else {
        for part in parts.iter().copied() {
            match part {
                InlineSourcePart::Text(range) => emit_inline_range(
                    range,
                    markdown.as_bytes(),
                    source_offset,
                    link_refs,
                    inline_parser,
                    inline_events,
                    events,
                ),
                InlineSourcePart::SoftBreak => {
                    events.push(MdxEvent::Inline(InlineEvent::SoftBreak));
                }
            }
        }
    }

    parts.clear();
}

fn contiguous_inline_range(input: &[u8], parts: &[InlineSourcePart]) -> Option<Range> {
    let mut parts = parts.iter();
    let InlineSourcePart::Text(first) = *parts.next()? else {
        return None;
    };
    let mut previous = first;
    let mut expect_text = false;

    for part in parts {
        match (*part, expect_text) {
            (InlineSourcePart::SoftBreak, false) => {
                expect_text = true;
            }
            (InlineSourcePart::Text(range), true) => {
                let gap = &input[previous.end_usize()..range.start_usize()];
                if gap != b"\n" && gap != b"\r\n" {
                    return None;
                }
                previous = range;
                expect_text = false;
            }
            _ => return None,
        }
    }
    if expect_text {
        return None;
    }

    Some(Range::from_usize(first.start_usize(), previous.end_usize()))
}

#[allow(clippy::too_many_arguments)]
fn emit_inline_range(
    range: Range,
    markdown: &[u8],
    source_offset: usize,
    link_refs: &LinkRefStore,
    inline_parser: &mut InlineParser,
    inline_events: &mut Vec<InlineEvent>,
    events: &mut Vec<MdxEvent>,
) {
    inline_events.clear();
    inline_parser.parse_mdx(range.slice(markdown), Some(link_refs), inline_events);
    let inline_offset = source_offset + range.start_usize();

    for event in inline_events.drain(..) {
        events.push(MdxEvent::Inline(offset_inline_event(event, inline_offset)));
    }
}

fn offset_range(range: Range, offset: usize) -> Range {
    Range::from_usize(offset + range.start_usize(), offset + range.end_usize())
}

fn offset_block_event(mut event: BlockEvent, offset: usize) -> BlockEvent {
    match &mut event {
        BlockEvent::CodeBlockStart {
            kind: CodeBlockKind::Fenced { info: Some(range) },
        }
        | BlockEvent::HtmlBlockText(range)
        | BlockEvent::Code(range) => *range = offset_range(*range, offset),
        _ => {}
    }
    event
}

fn offset_inline_event(mut event: InlineEvent, offset: usize) -> InlineEvent {
    match &mut event {
        InlineEvent::Text(range)
        | InlineEvent::Code(range)
        | InlineEvent::Html(range)
        | InlineEvent::MathInline(range)
        | InlineEvent::MathDisplay(range)
        | InlineEvent::MdxExpression(range)
        | InlineEvent::MdxJsxOpen(range)
        | InlineEvent::MdxJsxClose(range)
        | InlineEvent::MdxJsxSelfClose(range) => *range = offset_range(*range, offset),
        InlineEvent::LinkStart { url, title } | InlineEvent::ImageStart { url, title } => {
            *url = offset_range(*url, offset);
            if let Some(range) = title {
                *range = offset_range(*range, offset);
            }
        }
        InlineEvent::Autolink { url, .. } | InlineEvent::AutolinkLiteral { url, .. } => {
            *url = offset_range(*url, offset);
        }
        _ => {}
    }
    event
}

fn block_event_range(event: &BlockEvent) -> Option<Range> {
    match event {
        BlockEvent::CodeBlockStart {
            kind: CodeBlockKind::Fenced { info: Some(range) },
        }
        | BlockEvent::HtmlBlockText(range)
        | BlockEvent::Text(range)
        | BlockEvent::Code(range) => Some(*range),
        _ => None,
    }
}

fn inline_event_range(event: &InlineEvent) -> Option<Range> {
    match event {
        InlineEvent::Text(range)
        | InlineEvent::Code(range)
        | InlineEvent::Html(range)
        | InlineEvent::MathInline(range)
        | InlineEvent::MathDisplay(range)
        | InlineEvent::MdxExpression(range)
        | InlineEvent::MdxJsxOpen(range)
        | InlineEvent::MdxJsxClose(range)
        | InlineEvent::MdxJsxSelfClose(range) => Some(*range),
        InlineEvent::LinkStart { url, .. }
        | InlineEvent::ImageStart { url, .. }
        | InlineEvent::Autolink { url, .. }
        | InlineEvent::AutolinkLiteral { url, .. } => Some(*url),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_event_stays_compact() {
        assert!(std::mem::size_of::<MdxEvent>() <= 40);
    }
}
