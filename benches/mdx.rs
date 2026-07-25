//! Bench-only screening for MDX segmentation and container-flow recognition.
//!
//! Run with:
//!
//! ```sh
//! cargo bench --bench mdx --features mdx
//! ```
//!
//! `logical_flow_candidates` is deliberately not a production parser. It is a
//! zero-allocation, line-local pre-scan that approximates the extra work a
//! container-aware flow recognizer would have to do before it can hand a tag to
//! the existing JSX parser. In particular, it does not track list continuation
//! indentation, multiline tags, ESM, expressions, or container state across
//! lines. It must therefore not be read as a compatibility implementation.
//!
//! Its purpose is narrower: bound the cost of a universal extra recognition
//! pass on documents that do and do not contain container-local JSX. A future
//! implementation may fuse this work with `mdx::segment`, or gate it behind an
//! opt-in API, and must be measured independently.

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use ferromark::mdx::{self, jsx_tag::parse_jsx_tag};

const PLAIN_MARKDOWN: &str = include_str!("fixtures/commonmark-20k.md");

fn root_flow_mdx() -> String {
    r#"<Callout kind="note">

## Root-level MDX

- First item
- Second item with **strong** text

</Callout>

"#
    .repeat(240)
}

fn container_flow_mdx() -> String {
    r#"> <Callout kind="note">
>
> ## Blockquote-local MDX
>
> - First item
> - Second item with **strong** text
>
> </Callout>

"#
    .repeat(240)
}

fn bench_mdx(c: &mut Criterion) {
    let root_flow = root_flow_mdx();
    let container_flow = container_flow_mdx();

    // Keep the experiment honest: the probe must actually find the JSX lines
    // present in the fixture before it is used as a benchmarked operation.
    assert_eq!(logical_flow_candidates(&root_flow), 480);
    assert_eq!(logical_flow_candidates(&container_flow), 480);

    let mut group = c.benchmark_group("mdx");
    group.sample_size(80);
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.measurement_time(std::time::Duration::from_secs(5));

    for (name, input) in [
        ("plain_markdown", PLAIN_MARKDOWN),
        ("root_flow", root_flow.as_str()),
        ("container_flow", container_flow.as_str()),
    ] {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(format!("segment/{name}"), input, |b, input| {
            b.iter(|| mdx::segment(black_box(input)))
        });
        group.bench_with_input(format!("render/{name}"), input, |b, input| {
            b.iter(|| mdx::render(black_box(input)))
        });
        group.bench_with_input(format!("render_plus_probe/{name}"), input, |b, input| {
            b.iter(|| {
                black_box(logical_flow_candidates(black_box(input)));
                mdx::render(black_box(input))
            })
        });
    }

    // `parse_events` has a different (semantic-event) output contract than
    // `render`; report it separately rather than treating it as a like-for-like
    // HTML-rendering replacement.
    for (name, input) in [
        ("plain_markdown", PLAIN_MARKDOWN),
        ("container_flow", container_flow.as_str()),
    ] {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(format!("parse_events/{name}"), input, |b, input| {
            b.iter(|| mdx::parse_events(black_box(input)))
        });
    }

    group.finish();
}

/// Count single-line, flow-position JSX tags after lightweight blockquote/list
/// prefixes. This deliberately gives no semantics to the surrounding
/// container: it is a cost probe, not a parser.
fn logical_flow_candidates(input: &str) -> usize {
    input
        .as_bytes()
        .split(|byte| *byte == b'\n')
        .filter(|line| is_logical_flow_jsx(trim_carriage_return(line)))
        .count()
}

fn trim_carriage_return(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\r").unwrap_or(line)
}

fn is_logical_flow_jsx(line: &[u8]) -> bool {
    let logical_start = after_container_prefixes(line);
    if !logical_start.starts_with(b"<") {
        return false;
    }

    parse_jsx_tag(logical_start).is_some_and(|tag| {
        logical_start[tag.end_offset..]
            .iter()
            .all(|byte| matches!(byte, b' ' | b'\t'))
    })
}

/// Strip a repeated blockquote/list marker sequence from a physical line.
///
/// This intentionally does not attempt to retain container state across
/// lines. Lines that are merely list continuations are still scanned after
/// their indentation, which is appropriate for the cost screening but not a
/// substitute for block parsing.
fn after_container_prefixes(line: &[u8]) -> &[u8] {
    let mut pos = 0;

    loop {
        pos = skip_horizontal_whitespace(line, pos);
        if line.get(pos) == Some(&b'>') {
            pos += 1;
            if matches!(line.get(pos), Some(b' ' | b'\t')) {
                pos += 1;
            }
            continue;
        }
        if let Some(after_marker) = list_marker_end(line, pos) {
            pos = after_marker;
            continue;
        }
        return &line[pos..];
    }
}

fn skip_horizontal_whitespace(line: &[u8], mut pos: usize) -> usize {
    while matches!(line.get(pos), Some(b' ' | b'\t')) {
        pos += 1;
    }
    pos
}

fn list_marker_end(line: &[u8], pos: usize) -> Option<usize> {
    match line.get(pos) {
        Some(b'-' | b'+' | b'*') if matches!(line.get(pos + 1), Some(b' ' | b'\t')) => {
            Some(pos + 2)
        }
        Some(byte) if byte.is_ascii_digit() => {
            let mut end = pos;
            while end < line.len() && line[end].is_ascii_digit() && end - pos < 9 {
                end += 1;
            }
            if end > pos
                && matches!(line.get(end), Some(b'.' | b')'))
                && matches!(line.get(end + 1), Some(b' ' | b'\t'))
            {
                Some(end + 2)
            } else {
                None
            }
        }
        _ => None,
    }
}

criterion_group!(benches, bench_mdx);
criterion_main!(benches);
