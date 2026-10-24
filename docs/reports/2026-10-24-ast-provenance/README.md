# AST provenance cost characterization

Date: 2026-10-24. Issues: #563 (image alt content), #564 (MDX diagnostics),
#565 (callout inspection). This report measures only the image-alt retention
option, comparing off and on in the same implementation. It is not a
before/after throughput comparison with the base revision.

## Setup and correctness

The [harness](../../../benches/image_alt_content.rs) repeats each input 64 times:
ordinary prose with emphasis, plain image labels, or image labels containing
inline code, strong emphasis, Unicode, and an entity. Each image lane has 64
images. HTML equality is asserted before timing. The focused integration tests
also compare HTML/JSX output, retained spans, reference images, nested images,
and MDX containers.

The timed boundary includes creating/dropping a fresh arena, constructing a GFM
parser, and parsing; it excludes fixture generation, rendering and reporting.
There are no Cargo features enabled. Criterion 0.8.2 uses a one-second warmup,
three-second measurement and 30 samples per lane, with off followed by on.
The bench profile uses opt-level 3, fat LTO and one codegen unit.

Host: macOS 27.0.1, aarch64-apple-darwin, Rust 1.95.0 / LLVM 22.1.2.
The CPU model is unavailable in the sandbox. This is one local characterization
run with no controlled power/thermal state. It supports an opt-in cost decision,
not a cross-platform performance promise or a statistically paired speed claim.
[Environment](environment.json) and [source hashes](source-sha256.json) identify
the local candidate on top of the recorded base commit.

```sh
cargo bench -p ferromark --bench image_alt_content --locked -- \
  --warm-up-time 1 --measurement-time 3 --sample-size 30 --noplot
```

## Results

Times are Criterion slope estimates in microseconds, with 95% confidence
intervals. The delta compares central estimates only. Arena bytes are measured
outside timing in one sufficiently large chunk: initial capacity minus remaining
capacity, with an assertion that no second chunk was allocated. They include
alignment and parser scratch allocations, exclude arena metadata and heap-owned
parser memo tables, and are not process RSS or total allocator traffic.
The table is generated from [retained estimates](raw/), [measurements](measurements.json)
and the [complete run log](criterion.log); outliers remain in those artifacts.

| Workload | Input bytes | Off µs [95% CI] | On µs [95% CI] | Time delta | Off arena bytes | On arena bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| prose | 3,264 | 6.697 [6.681, 6.718] | 6.698 [6.680, 6.717] | +0.0% | 40,480 | 40,480 |
| plain | 2,624 | 4.846 [4.835, 4.860] | 5.440 [5.429, 5.457] | +12.3% | 17,440 | 22,048 |
| rich | 3,328 | 16.911 [16.858, 16.976] | 17.125 [17.023, 17.217] | +1.3% | 65,632 | 68,192 |

Prose without images consumes the same arena storage in both modes. Retaining
plain alt content adds an inline node and its metadata; formatted alt already
needs inline parsing for flattening, so retention reuses those parsed nodes.
The extra analysis is off in every preset. Independently of that switch,
`Image` gains one optional pointer (64 to 72 bytes on this 64-bit target).
Ordinary images do not retain an `ImageAlt` or its children. Formatted alt
still needs transient inline parsing for its flattened value.

Callout inspection borrows the existing AST without additional parser storage.
Typed MDX details live in the existing boxed error path; normal fail-fast parsing
does not allocate a diagnostic list. These two APIs were validated functionally,
not timed by this harness.

A subsequent [before/after investigation](../2026-10-24-ast-performance/README.md)
compares the public defaults with the prior revision and records the accepted
MDX storage and parser-path changes. The off/on measurements here describe the
initial API implementation.
