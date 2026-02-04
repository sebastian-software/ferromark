//! Comparison benchmarks: md-fast vs other Rust Markdown parsers
//!
//! Run with: cargo bench --bench comparison
//!
//! Parsers compared:
//! - md-fast (this crate)
//! - md4c (C)
//! - pulldown-cmark (most popular, used by rustdoc)
//! - comrak (100% CommonMark compliant, GFM support)
//! - markdown (markdown-rs, wooorm's parser)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memchr::memchr;
use std::os::raw::{c_char, c_int, c_uint, c_void};

/// Sample documents for benchmarking
mod samples {
    /// Tiny document - baseline measurement
    pub const TINY: &str = "Hello, **world**!";

    /// Small README-style document
    pub const SMALL: &str = r#"# Heading

This is a paragraph with *emphasis* and **strong** text.

- Item 1
- Item 2
- Item 3

`inline code` and [a link](https://example.com).
"#;

    /// Medium-sized README
    pub const MEDIUM: &str = r#"# Project README

This is a sample README file that demonstrates various Markdown features.

## Features

- Fast parsing
- Zero-copy design
- SIMD acceleration

### Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```

## Performance

The parser achieves **high throughput** on typical documents.

> This is a blockquote with some *emphasized* text.

### Links

- [GitHub](https://github.com)
- [Documentation](https://docs.rs)

## Conclusion

Thank you for reading!
"#;

    /// Simple document: headers, lists, paragraphs, basic inline formatting
    pub const SIMPLE: &str = r#"# Title

## Section A

This is a paragraph with *emphasis* and **strong**.

- Item one
- Item two
- Item three

Another paragraph.
"#;

    /// Link-heavy document: autolinks, inline links, entities, images
    pub const LINKS: &str = r#"# Links

Visit <https://example.com> or <mailto:test@example.com>.

Inline [link](https://example.com/path?query=1&x=2) with &amp; entity.

![Image alt](https://example.com/image.png "Title")

Text with `code` and [another link](https://example.com).
"#;

    /// Reference link definitions and uses
    pub const REFS: &str = r#"[ref-1]: https://example.com "Example"
[ref-2]: /relative/path 'Rel'

This uses [ref-1] and [ref-2].

[Another ref][ref-1] and [short][ref-2].
"#;

    /// Nested lists and mixed block elements
    pub const LISTS: &str = r#"# Lists

1. Ordered
   1. Nested ordered
   2. Nested ordered
2. Ordered
   - Nested unordered
     - Deep nested

> Blockquote
> - Quoted list item
>   - Nested in quote
"#;

    /// HTML blocks and inline HTML
    pub const HTML: &str = r#"<div class="note">
<p>Inline <em>HTML</em> inside a block.</p>
</div>

Paragraph with <span class="hi">inline HTML</span> and &amp; entity.

<script>
var x = 1;
</script>
"#;

    /// Mixed realistic document with multiple features
    pub const MIXED: &str = r#"# Mixed Sample

Intro paragraph with *emphasis*, **strong**, and `code`.

[ref]: https://example.com "Title"

## Section

> Blockquote with [link][ref] and <https://example.com>.

- List item with ![image](https://example.com/x.png)
- List item with `<code>` and <span>HTML</span>

```rust
fn example() {
    println!("Hello");
}
```

Paragraph after code.
"#;

    /// Generate a large document by repeating sections
    pub fn large() -> String {
        let section = r#"
## Section Title

This paragraph contains various inline elements like *emphasis*, **strong**,
`code`, and [links](https://example.com).

- First bullet point with **bold** text
- Second bullet point with *italic* text
- Third point with `code`

> A blockquote that spans
> multiple lines.

```rust
fn example() {
    let x = 42;
    println!("{}", x);
}
```

Another paragraph to add some content. This helps test the parser's ability
to handle longer documents efficiently.

"#;
        section.repeat(50)
    }
}

/// Parse with md-fast
fn parse_md_fast(input: &str) -> String {
    md_fast::to_html(input)
}

/// Parse with pulldown-cmark
fn parse_pulldown_cmark(input: &str) -> String {
    use pulldown_cmark::{html, Parser};
    let parser = Parser::new(input);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}

/// Parse with comrak
fn parse_comrak(input: &str) -> String {
    comrak::markdown_to_html(input, &comrak::Options::default())
}

/// Parse with markdown-rs
fn parse_markdown_rs(input: &str) -> String {
    markdown::to_html(input)
}

unsafe extern "C" {
    fn md_html(
        input: *const c_char,
        input_size: c_uint,
        process_output: extern "C" fn(*const c_char, c_uint, *mut c_void),
        userdata: *mut c_void,
        parser_flags: c_uint,
        renderer_flags: c_uint,
    ) -> c_int;
}

extern "C" fn md4c_output(data: *const c_char, size: c_uint, userdata: *mut c_void) {
    if data.is_null() || userdata.is_null() || size == 0 {
        return;
    }
    let buf = unsafe { &mut *(userdata as *mut Vec<u8>) };
    let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) };
    buf.extend_from_slice(bytes);
}

/// Parse with md4c (C) via md_html.
fn parse_md4c(input: &str) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(input.len() + input.len() / 4);
    let rc = unsafe {
        md_html(
            input.as_ptr() as *const c_char,
            input.len() as c_uint,
            md4c_output,
            &mut out as *mut Vec<u8> as *mut c_void,
            0,
            0,
        )
    };
    debug_assert_eq!(rc, 0, "md_html returned error");
    unsafe { String::from_utf8_unchecked(out) }
}

fn bench_tiny(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiny");
    let input = samples::TINY;
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("md-fast", |b| {
        b.iter(|| parse_md_fast(black_box(input)))
    });
    group.bench_function("md4c", |b| {
        b.iter(|| parse_md4c(black_box(input)))
    });
    group.bench_function("pulldown-cmark", |b| {
        b.iter(|| parse_pulldown_cmark(black_box(input)))
    });
    group.bench_function("comrak", |b| {
        b.iter(|| parse_comrak(black_box(input)))
    });
    group.bench_function("markdown-rs", |b| {
        b.iter(|| parse_markdown_rs(black_box(input)))
    });

    group.finish();
}

fn bench_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("small");
    let input = samples::SMALL;
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("md-fast", |b| {
        b.iter(|| parse_md_fast(black_box(input)))
    });
    group.bench_function("md4c", |b| {
        b.iter(|| parse_md4c(black_box(input)))
    });
    group.bench_function("pulldown-cmark", |b| {
        b.iter(|| parse_pulldown_cmark(black_box(input)))
    });
    group.bench_function("comrak", |b| {
        b.iter(|| parse_comrak(black_box(input)))
    });
    group.bench_function("markdown-rs", |b| {
        b.iter(|| parse_markdown_rs(black_box(input)))
    });

    group.finish();
}

fn bench_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple");
    let input = samples::SIMPLE;
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("md-fast", |b| {
        b.iter(|| parse_md_fast(black_box(input)))
    });
    group.bench_function("md4c", |b| {
        b.iter(|| parse_md4c(black_box(input)))
    });
    group.bench_function("pulldown-cmark", |b| {
        b.iter(|| parse_pulldown_cmark(black_box(input)))
    });
    group.bench_function("comrak", |b| {
        b.iter(|| parse_comrak(black_box(input)))
    });
    group.bench_function("markdown-rs", |b| {
        b.iter(|| parse_markdown_rs(black_box(input)))
    });

    group.finish();
}

fn bench_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium");
    let input = samples::MEDIUM;
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("md-fast", |b| {
        b.iter(|| parse_md_fast(black_box(input)))
    });
    group.bench_function("md4c", |b| {
        b.iter(|| parse_md4c(black_box(input)))
    });
    group.bench_function("pulldown-cmark", |b| {
        b.iter(|| parse_pulldown_cmark(black_box(input)))
    });
    group.bench_function("comrak", |b| {
        b.iter(|| parse_comrak(black_box(input)))
    });
    group.bench_function("markdown-rs", |b| {
        b.iter(|| parse_markdown_rs(black_box(input)))
    });

    group.finish();
}

fn bench_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("large");
    let input = samples::large();
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("md-fast", |b| {
        b.iter(|| parse_md_fast(black_box(&input)))
    });
    group.bench_function("md4c", |b| {
        b.iter(|| parse_md4c(black_box(&input)))
    });
    group.bench_function("pulldown-cmark", |b| {
        b.iter(|| parse_pulldown_cmark(black_box(&input)))
    });
    group.bench_function("comrak", |b| {
        b.iter(|| parse_comrak(black_box(&input)))
    });
    group.bench_function("markdown-rs", |b| {
        b.iter(|| parse_markdown_rs(black_box(&input)))
    });

    group.finish();
}

/// Complexity comparison across representative feature sets
fn bench_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("complexity");

    let cases: Vec<(&str, &str)> = vec![
        ("simple", samples::SIMPLE),
        ("links", samples::LINKS),
        ("refs", samples::REFS),
        ("lists", samples::LISTS),
        ("html", samples::HTML),
        ("mixed", samples::MIXED),
    ];

    for (name, input) in &cases {
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_with_input(BenchmarkId::new("md-fast", name), input, |b, s| {
            b.iter(|| parse_md_fast(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("md4c", name), input, |b, s| {
            b.iter(|| parse_md4c(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("pulldown-cmark", name), input, |b, s| {
            b.iter(|| parse_pulldown_cmark(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("comrak", name), input, |b, s| {
            b.iter(|| parse_comrak(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("markdown-rs", name), input, |b, s| {
            b.iter(|| parse_markdown_rs(black_box(s)))
        });
    }

    group.finish();
}

/// Throughput comparison across all document sizes
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    let sizes: Vec<(&str, String)> = vec![
        ("tiny", samples::TINY.to_string()),
        ("small", samples::SMALL.to_string()),
        ("medium", samples::MEDIUM.to_string()),
        ("large", samples::large()),
    ];

    for (name, input) in &sizes {
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_with_input(BenchmarkId::new("md-fast", name), input, |b, s| {
            b.iter(|| parse_md_fast(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("md4c", name), input, |b, s| {
            b.iter(|| parse_md4c(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("pulldown-cmark", name), input, |b, s| {
            b.iter(|| parse_pulldown_cmark(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("comrak", name), input, |b, s| {
            b.iter(|| parse_comrak(black_box(s)))
        });
        group.bench_with_input(BenchmarkId::new("markdown-rs", name), input, |b, s| {
            b.iter(|| parse_markdown_rs(black_box(s)))
        });
    }

    group.finish();
}

fn bench_experiments(c: &mut Criterion) {
    let mut group = c.benchmark_group("experiments");

    let docs = [
        ("simple", samples::SIMPLE),
        ("links", samples::LINKS),
        ("refs", samples::REFS),
        ("mixed", samples::MIXED),
    ];

    for (name, input) in docs {
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_with_input(BenchmarkId::new("baseline_to_html", name), input, |b, text| {
            b.iter(|| {
                let html = md_fast::to_html(black_box(text));
                black_box(html);
            })
        });

        group.bench_with_input(BenchmarkId::new("hybrid_paragraph_buffer", name), input, |b, text| {
            b.iter(|| {
                let html = hybrid_paragraph_buffer(black_box(text));
                black_box(html);
            })
        });

        group.bench_with_input(
            BenchmarkId::new("prescan_candidates_then_to_html", name),
            input,
            |b, text| {
                b.iter(|| {
                    let _c = prescan_candidates(black_box(text.as_bytes()));
                    let html = md_fast::to_html(black_box(text));
                    black_box(html);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("prescan_full_then_to_html", name),
            input,
            |b, text| {
                b.iter(|| {
                    let _c = prescan_full(black_box(text.as_bytes()));
                    let html = md_fast::to_html(black_box(text));
                    black_box(html);
                })
            },
        );
    }

    group.finish();
}

fn prescan_candidates(input: &[u8]) -> usize {
    let mut count = 0usize;
    let mut line_start = 0usize;
    while line_start <= input.len() {
        let line_end = match memchr(b'\n', &input[line_start..]) {
            Some(i) => line_start + i,
            None => input.len(),
        };
        let line = &input[line_start..line_end];
        let mut i = 0usize;
        let mut spaces = 0u8;
        while i < line.len() {
            match line[i] {
                b' ' => {
                    spaces += 1;
                    if spaces > 3 {
                        break;
                    }
                    i += 1;
                }
                b'\t' => break,
                b'[' => {
                    count += 1;
                    break;
                }
                _ => break,
            }
        }
        if line_end == input.len() {
            break;
        }
        line_start = line_end + 1;
    }
    count
}

fn prescan_full(input: &[u8]) -> usize {
    let mut count = 0usize;
    let mut pos = 0usize;
    while pos < input.len() {
        if pos == 0 || input[pos - 1] == b'\n' {
            if let Some((_def, end)) = parse_link_ref_def(input, pos) {
                count += 1;
                pos = end;
                continue;
            }
        }
        pos += 1;
    }
    count
}

fn hybrid_paragraph_buffer(input: &str) -> String {
    // Bench-only prototype: stream paragraph-by-paragraph with optional buffering
    // for ref-candidate paragraphs. Not CommonMark-correct; used to estimate overheads.
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len() + input.len() / 4);
    let mut pos = 0usize;
    let len = bytes.len();

    let mut buf = Vec::new();
    while pos <= len {
        let line_end = match memchr(b'\n', &bytes[pos..]) {
            Some(i) => pos + i,
            None => len,
        };
        let line = &bytes[pos..line_end];

        let is_blank = line.iter().all(|&b| b == b' ' || b == b'\t');

        if is_blank {
            if !buf.is_empty() {
                let para = std::str::from_utf8(&buf).unwrap_or("");
                if paragraph_has_ref_candidate(para) {
                    out.push_str(&md_fast::to_html(para));
                } else {
                    out.push_str(&md_fast::to_html(para));
                }
                buf.clear();
            }
            pos = if line_end == len { len + 1 } else { line_end + 1 };
            continue;
        }

        if !buf.is_empty() {
            buf.push(b'\n');
        }
        buf.extend_from_slice(line);

        if line_end == len {
            pos = len + 1;
        } else {
            pos = line_end + 1;
        }
    }

    if !buf.is_empty() {
        let para = std::str::from_utf8(&buf).unwrap_or("");
        if paragraph_has_ref_candidate(para) {
            out.push_str(&md_fast::to_html(para));
        } else {
            out.push_str(&md_fast::to_html(para));
        }
    }

    out
}

fn paragraph_has_ref_candidate(input: &str) -> bool {
    // Heuristic: any '[' without immediate ']' + '(' is a candidate
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            return true;
        }
        i += 1;
    }
    false
}

#[allow(dead_code)]
#[derive(Debug)]
struct ParsedLinkRefDef {
    label: Vec<u8>,
    url: Vec<u8>,
    title: Option<Vec<u8>>,
}

fn parse_link_ref_def(input: &[u8], start: usize) -> Option<(ParsedLinkRefDef, usize)> {
    let len = input.len();
    let mut i = start;

    // Up to 3 leading spaces
    let mut spaces = 0usize;
    while i < len && input[i] == b' ' && spaces < 3 {
        i += 1;
        spaces += 1;
    }

    if i >= len || input[i] != b'[' {
        return None;
    }
    i += 1;

    // Parse label
    let label_start = i;
    while i < len {
        match input[i] {
            b'\\' => {
                if i + 1 < len {
                    i += 2;
                } else {
                    return None;
                }
            }
            b'[' => return None,
            b']' => break,
            _ => i += 1,
        }
    }
    if i >= len || input[i] != b']' {
        return None;
    }
    let label_end = i;
    i += 1;

    if i >= len || input[i] != b':' {
        return None;
    }
    i += 1;

    // Skip whitespace (allow a single line break)
    let mut saw_newline = false;
    while i < len {
        match input[i] {
            b' ' | b'\t' => i += 1,
            b'\n' => {
                if saw_newline {
                    return None;
                }
                saw_newline = true;
                i += 1;
            }
            _ => break,
        }
    }
    if i >= len {
        return None;
    }

    // Parse destination
    let (url_bytes, mut i) = if input[i] == b'<' {
        i += 1;
        let url_start = i;
        while i < len && input[i] != b'>' && input[i] != b'\n' {
            i += 1;
        }
        if i >= len || input[i] != b'>' {
            return None;
        }
        let url_end = i;
        i += 1;
        if i < len && !matches!(input[i], b' ' | b'\t' | b'\n') {
            return None;
        }
        (input[url_start..url_end].to_vec(), i)
    } else {
        let url_start = i;
        let mut parens = 0i32;
        while i < len {
            let b = input[i];
            if b == b'\\' && i + 1 < len {
                i += 2;
                continue;
            }
            if b == b'(' {
                parens += 1;
                i += 1;
                continue;
            }
            if b == b')' {
                if parens == 0 {
                    break;
                }
                parens -= 1;
                i += 1;
                continue;
            }
            if matches!(b, b' ' | b'\t' | b'\n') {
                break;
            }
            i += 1;
        }
        if url_start == i {
            return None;
        }
        (input[url_start..i].to_vec(), i)
    };

    let mut line_end = i;
    while line_end < len && input[line_end] != b'\n' {
        line_end += 1;
    }

    // Skip whitespace before title
    let mut j = i;
    let mut had_title_sep = false;
    let mut title_on_newline = false;
    while j < len && (input[j] == b' ' || input[j] == b'\t') {
        j += 1;
        had_title_sep = true;
    }
    if j < len && input[j] == b'\n' {
        j += 1;
        had_title_sep = true;
        title_on_newline = true;
        while j < len && (input[j] == b' ' || input[j] == b'\t') {
            j += 1;
        }
    }

    let mut title_bytes = None;
    if had_title_sep && j < len {
        let opener = input[j];
        let closer = match opener {
            b'"' => b'"',
            b'\'' => b'\'',
            b'(' => b')',
            _ => 0,
        };

        if closer != 0 {
            j += 1;
            let title_start = j;
            while j < len {
                let b = input[j];
                if b == b'\\' && j + 1 < len {
                    j += 2;
                    continue;
                }
                if b == b'\n' && j + 1 < len && input[j + 1] == b'\n' {
                    if title_on_newline {
                        return Some((
                            ParsedLinkRefDef {
                                label: input[label_start..label_end].to_vec(),
                                url: url_bytes,
                                title: None,
                            },
                            if line_end < len { line_end + 1 } else { line_end },
                        ));
                    }
                    return None;
                }
                if b == closer {
                    break;
                }
                j += 1;
            }
            if j >= len || input[j] != closer {
                if title_on_newline {
                    return Some((
                        ParsedLinkRefDef {
                            label: input[label_start..label_end].to_vec(),
                            url: url_bytes,
                            title: None,
                        },
                        if line_end < len { line_end + 1 } else { line_end },
                    ));
                }
                return None;
            }
            let title_end = j;
            j += 1;
            title_bytes = Some(input[title_start..title_end].to_vec());

            while j < len && (input[j] == b' ' || input[j] == b'\t') {
                j += 1;
            }
            if j < len && input[j] != b'\n' {
                if title_on_newline {
                    return Some((
                        ParsedLinkRefDef {
                            label: input[label_start..label_end].to_vec(),
                            url: url_bytes,
                            title: None,
                        },
                        if line_end < len { line_end + 1 } else { line_end },
                    ));
                }
                return None;
            }
            i = j;
        }
    }

    if title_bytes.is_none() {
        i = line_end;
    }

    if i < len && input[i] == b'\n' {
        i += 1;
    }

    Some((
        ParsedLinkRefDef {
            label: input[label_start..label_end].to_vec(),
            url: url_bytes,
            title: title_bytes,
        },
        i,
    ))
}

criterion_group!(
    benches,
    bench_tiny,
    bench_small,
    bench_simple,
    bench_medium,
    bench_large,
    bench_complexity,
    bench_throughput,
    bench_experiments
);
criterion_main!(benches);
