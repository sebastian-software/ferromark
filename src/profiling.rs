//! Feature-gated internal work counters for profiling builds.

use std::cell::Cell;

use crate::{BlockEvent, InlineEvent};

/// Aggregate pipeline work performed since the last reset.
#[derive(Debug, Clone, Copy, Default)]
pub struct PipelineSnapshot {
    /// Documents rendered.
    pub documents: u64,
    /// Total block events.
    pub block_events: u64,
    /// Block text and soft-break events.
    pub block_text_events: u64,
    /// List and blockquote container events.
    pub block_container_events: u64,
    /// Table events.
    pub block_table_events: u64,
    /// Code block and code content events.
    pub block_code_events: u64,
    /// Largest block-event capacity observed.
    pub max_block_event_capacity: u64,
    /// Inline parse invocations.
    pub inline_parses: u64,
    /// Inline input bytes visited at the parse boundary.
    pub inline_input_bytes: u64,
    /// Inline events emitted.
    pub inline_events: u64,
    /// Text and code inline events.
    pub inline_text_events: u64,
    /// Link, image, and autolink inline events.
    pub inline_link_events: u64,
    /// HTML inline events.
    pub inline_html_events: u64,
    /// Collected inline marks.
    pub inline_marks: u64,
    /// Generated inline emit points.
    pub inline_emit_points: u64,
    /// Inline parses completed through a plain-text fast path.
    pub inline_fast_paths: u64,
    /// Largest inline-event capacity observed.
    pub max_inline_event_capacity: u64,
    /// Largest mark-buffer capacity observed.
    pub max_mark_capacity: u64,
    /// Largest emit-point capacity observed.
    pub max_emit_point_capacity: u64,
    /// Bytes copied into paragraph scratch storage.
    pub paragraph_copied_bytes: u64,
}

thread_local! {
    static COUNTERS: Cell<PipelineSnapshot> = const { Cell::new(PipelineSnapshot {
        documents: 0,
        block_events: 0,
        block_text_events: 0,
        block_container_events: 0,
        block_table_events: 0,
        block_code_events: 0,
        max_block_event_capacity: 0,
        inline_parses: 0,
        inline_input_bytes: 0,
        inline_events: 0,
        inline_text_events: 0,
        inline_link_events: 0,
        inline_html_events: 0,
        inline_marks: 0,
        inline_emit_points: 0,
        inline_fast_paths: 0,
        max_inline_event_capacity: 0,
        max_mark_capacity: 0,
        max_emit_point_capacity: 0,
        paragraph_copied_bytes: 0,
    }) };
}

/// Reset counters for the current thread.
pub fn reset() {
    COUNTERS.with(|counters| counters.set(PipelineSnapshot::default()));
}

/// Snapshot counters for the current thread.
pub fn snapshot() -> PipelineSnapshot {
    COUNTERS.with(Cell::get)
}

pub(crate) fn record_block_events(events: &[BlockEvent], capacity: usize) {
    update(|counters| {
        counters.documents += 1;
        counters.block_events += events.len() as u64;
        counters.max_block_event_capacity = counters.max_block_event_capacity.max(capacity as u64);
        for event in events {
            match event {
                BlockEvent::Text(_) | BlockEvent::SoftBreak | BlockEvent::HtmlBlockText(_) => {
                    counters.block_text_events += 1;
                }
                BlockEvent::BlockQuoteStart { .. }
                | BlockEvent::BlockQuoteEnd
                | BlockEvent::ListStart { .. }
                | BlockEvent::ListEnd { .. }
                | BlockEvent::ListItemStart { .. }
                | BlockEvent::ListItemEnd => counters.block_container_events += 1,
                BlockEvent::TableStart
                | BlockEvent::TableEnd
                | BlockEvent::TableHeadStart
                | BlockEvent::TableHeadEnd
                | BlockEvent::TableBodyStart
                | BlockEvent::TableBodyEnd
                | BlockEvent::TableRowStart
                | BlockEvent::TableRowEnd
                | BlockEvent::TableCellStart { .. }
                | BlockEvent::TableCellEnd => counters.block_table_events += 1,
                BlockEvent::CodeBlockStart { .. }
                | BlockEvent::CodeBlockEnd
                | BlockEvent::Code(_) => counters.block_code_events += 1,
                _ => {}
            }
        }
    });
}

pub(crate) fn record_inline_events(events: &[InlineEvent], capacity: usize) {
    update(|counters| {
        counters.max_inline_event_capacity =
            counters.max_inline_event_capacity.max(capacity as u64);
        for event in events {
            match event {
                InlineEvent::Text(_) | InlineEvent::Code(_) => counters.inline_text_events += 1,
                #[cfg(feature = "mdx")]
                InlineEvent::MdxExpression(_)
                | InlineEvent::MdxJsxOpen(_)
                | InlineEvent::MdxJsxClose(_)
                | InlineEvent::MdxJsxSelfClose(_) => counters.inline_text_events += 1,
                InlineEvent::LinkStart { .. }
                | InlineEvent::LinkStartRef { .. }
                | InlineEvent::LinkEnd
                | InlineEvent::ImageStart { .. }
                | InlineEvent::ImageStartRef { .. }
                | InlineEvent::ImageEnd
                | InlineEvent::Autolink { .. }
                | InlineEvent::AutolinkLiteral { .. } => counters.inline_link_events += 1,
                InlineEvent::Html(_) => counters.inline_html_events += 1,
                _ => {}
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn record_inline_parse(
    input_bytes: usize,
    events: usize,
    marks: usize,
    mark_capacity: usize,
    emit_points: usize,
    emit_point_capacity: usize,
    fast_path: bool,
) {
    update(|counters| {
        counters.inline_parses += 1;
        counters.inline_input_bytes += input_bytes as u64;
        counters.inline_events += events as u64;
        counters.inline_marks += marks as u64;
        counters.inline_emit_points += emit_points as u64;
        counters.inline_fast_paths += u64::from(fast_path);
        counters.max_mark_capacity = counters.max_mark_capacity.max(mark_capacity as u64);
        counters.max_emit_point_capacity = counters
            .max_emit_point_capacity
            .max(emit_point_capacity as u64);
    });
}

pub(crate) fn record_paragraph_copy(bytes: usize) {
    update(|counters| {
        counters.paragraph_copied_bytes += bytes as u64;
    });
}

fn update(update_counters: impl FnOnce(&mut PipelineSnapshot)) {
    COUNTERS.with(|cell| {
        let mut counters = cell.get();
        update_counters(&mut counters);
        cell.set(counters);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_should_clear_recorded_work() {
        reset();
        record_paragraph_copy(12);
        reset();

        assert_eq!(snapshot().paragraph_copied_bytes, 0);
    }
}
