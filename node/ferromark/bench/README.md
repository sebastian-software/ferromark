# N-API boundary measurement

`boundary.mjs` measures how much of a Node.js call goes to the N-API boundary
and how much goes to the Rust core. It covers `toHtml` (one-shot),
`Renderer.toHtml` (reused), `toHtmlBuffer` and `Renderer.toHtmlBuffer`, and
splits each call into four parts:

- **input**: the JavaScript string becomes a Rust `String`. napi-rs calls
  `napi_get_value_string_utf8` twice: first a UTF-8 length pass, then a copy
  into a zero-filled buffer. V8 transcodes its Latin-1 or UTF-16 storage on
  the way.
- **core**: arena, parse and render in Rust.
- **output**: `napi_create_string_utf8` decodes the HTML into a V8 string.
  `toHtmlBuffer` copies the HTML into a `Vec` and wraps it in an external
  `Buffer`.
- **fixed**: the bare N-API call. One-shot calls also pay for
  `Renderer::new`. Option handling is reported separately because the
  default call passes no options.

The benchmark times the four public exports with the same addon that carries
the diagnostic exports, so all lanes run in one binary. It does not time the
JavaScript facade in `index.mjs`. With no options, the facade adds only
`validateOptions(undefined)` and a cached `loadNative()`.

## Build

The diagnostic exports come from the `boundary-bench` Cargo feature. Like
`panic-test`, it never reaches a package. `build-native.mjs` refuses to build a
diagnostic feature into the package directory, and `verify-package.mjs`
rejects a package `native.d.ts` that mentions one. Build into
`target/boundary-bench/` at the repository root, which is where the script
looks by default. From `node/`:

```sh
pnpm install --frozen-lockfile
FERROMARK_NAPI_FEATURES=boundary-bench \
  FERROMARK_NATIVE_OUTPUT_DIR="$PWD/../target/boundary-bench" \
  pnpm build:native
```

Add `FERROMARK_PGO=1` to profile the addon as published builds are profiled
([ADR-0019](../../../docs/arch/ADR-0019-profile-guided-native-addon.md)). The
feature adds exports but leaves the profiled core code unchanged.

## Run

From `node/ferromark/`:

```sh
node --expose-gc bench/boundary.mjs --json ../../target/boundary-bench/boundary.json
```

| Flag               | Default                                                            | Meaning                                                               |
| ------------------ | ------------------------------------------------------------------ | --------------------------------------------------------------------- |
| `--addon <path>`   | `target/boundary-bench/`                                           | The diagnostic addon, or a directory holding exactly one `.node` file |
| `--corpus <path>`  | `docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz` | The 57 broad documents                                                |
| `--filter <regex>` | all documents                                                      | Documents whose name matches                                          |
| `--rounds <n>`     | 15                                                                 | Paired rounds per document                                            |
| `--batch-ms <ms>`  | 2                                                                  | Target duration of one timed batch                                    |
| `--warmup-ms <ms>` | 20                                                                 | Warmup per lane and document                                          |
| `--two-byte`       | off                                                                | Store every input as UTF-16, keeping the content (see below)          |
| `--json <path>`    | none                                                               | Also write every round, median and derived share                      |
| `--smoke`          | off                                                                | Five documents, 3 rounds, 0.25 ms batches: a check that it works      |

The defaults should take one to two minutes. `--expose-gc` lets the script collect
garbage between documents, and the script runs without it too.

## Method

Before timing, the script checks every lane's output for each document. The
public exports, the probe's HTML and `Buffer`, and every candidate must return
the same HTML. The core loops must match the HTML length, and each input path
must match `Buffer.byteLength`.

Each lane is one loop over `k` calls, timed with `process.hrtime.bigint()`.
`k` is calibrated per lane so that a batch lasts `--batch-ms`. The core lanes
run `k` iterations inside one native call instead. In each round, every lane
runs once. The order rotates from round to round, and every other round runs
it backwards. Each part is computed from the lanes of the same round, and the
script reports the median over the rounds. The event loop turns after each
batch, outside the timed region, so N-API buffer finalizers run and memory
stays bounded. Because of that, deferred finalizer work is not charged to any
lane.

| Lane                                     | What it runs                                                                          |
| ---------------------------------------- | ------------------------------------------------------------------------------------- |
| `toHtml`, `toHtmlBuffer`                 | the public one-shot exports                                                           |
| `rendererToHtml`, `rendererToHtmlBuffer` | the public `Renderer` methods on one long-lived renderer                              |
| `noop`                                   | `boundaryNoop()`: the free-function call floor                                        |
| `methodNoop`                             | `BoundaryProbe#noop()`: the method call floor, including the unwrap of `this`         |
| `len`                                    | `boundaryLen(markdown)`: the call plus input conversion                               |
| `echo`                                   | `boundaryEcho(markdown)`: input plus output conversion of the Markdown                |
| `make`                                   | `boundaryMake(n)`: output conversion of `n` ASCII bytes, `n` = HTML length            |
| `html`, `htmlBuffer`                     | the exact HTML as `Renderer.toHtml` and `toHtmlBuffer` return it, rendered beforehand |
| `coreFresh`, `coreReuse`, `coreSetup`    | `boundaryCoreOnly(markdown, k, lifecycle)`: the export's own Rust code, `k` times     |

`boundaryCoreOnly` calls the same `render_one_shot` and
`Renderer::render_reused` helpers as the exports. `fresh` is the one-shot
lifecycle and `reuse` is a renderer kept across calls. `setup` is only
`Renderer::new` and its drop. The script subtracts one `len` (or `noop`) call
of the same round before dividing by `k`.

Each API is split as follows. The `total` is the public lane.

| Part     | `toHtml`, `toHtmlBuffer`                         | `Renderer.toHtml`, `Renderer.toHtmlBuffer` |
| -------- | ------------------------------------------------ | ------------------------------------------ |
| input    | `len - noop`                                     | `len - noop`                               |
| core     | `coreFresh - coreSetup`                          | `coreReuse`                                |
| output   | `html - methodNoop` or `htmlBuffer - methodNoop` | the same                                   |
| fixed    | `noop + coreSetup`                               | `methodNoop`                               |
| residual | `total - input - core - output - fixed`          | the same                                   |

The residual is what the model does not explain: `catch_unwind`, the
`undefined` options check, cache effects between the parts, and noise. If it
stays large, a part is missing from the model. The **N-API** column adds up
input, output and the bare call floor.

## Reading the output

1. **Fixed per-call costs**: the call floors, option handling (the cost of an
   `Options` object is independent of the document), `new Renderer()` from
   JavaScript and `Renderer::new` inside Rust.
2. **Per document: time per call**: the median public lanes and parts in
   nanoseconds. `input` shows the characters the Markdown contains and how V8
   stores it: `ascii`, `latin1` (up to U+00FF) or `wide`, and `one-byte` or
   `two-byte`. A `*` marks non-ASCII HTML.
3. **Per document: share of each call**: input/core/output/fixed/residual as
   percentages of each API's median time.
4. **Median share per size bin** for each API, and per input representation
   for the string APIs. Documents have equal weight.
5. **Candidates**: the median saving per call of each reduction prototype per
   size bin. Each saving is paired per round. In parentheses is the saving's
   share of the call named in the table below.

V8 stores a string as one-byte (Latin-1) when every character fits, otherwise
as two-byte (UTF-16). Conversion cost follows the storage, not just the
length. In the broad corpus, 30 inputs are ASCII, 4 are Latin-1 and 23 need
two-byte storage. `--two-byte` stores every input as UTF-16 with the content
unchanged, so running with and without it compares the two storage paths on
identical text.

## Candidates

The `boundary-bench` feature also prototypes reductions. They are
measurements, not API.

| Candidate          | Lane                             | Compared with | Share of          | Public API change                                               |
| ------------------ | -------------------------------- | ------------- | ----------------- | --------------------------------------------------------------- |
| `singlePassInput`  | `boundaryLenSinglePass`          | `len`         | `toHtml`          | none: an internal conversion                                    |
| `bytesInput`       | `boundaryBytesLen(Buffer)`       | `len`         | `toHtml`          | new: accept UTF-8 bytes (`Buffer`/`Uint8Array`)                 |
| `encodeIntoInput`  | `TextEncoder.encodeInto` + bytes | `len`         | `toHtml`          | none if the facade does it with a kept scratch buffer           |
| `latin1Output`     | `BoundaryProbe#htmlLatin1`       | `html`        | `Renderer.toHtml` | none: an internal conversion for ASCII HTML                     |
| `externalOutput`   | `boundaryToHtmlExternal`         | `toHtml`      | `toHtml`          | none: an internal conversion for ASCII HTML                     |
| `bufferCopyOutput` | `BoundaryProbe#htmlBufferCopy`   | `htmlBuffer`  | `toHtmlBuffer`    | none: an internal conversion                                    |
| `cachedRenderer`   | `Renderer.toHtml`                | `toHtml`      | `toHtml`          | none if `toHtml` without options reuses a module-level renderer |

- `singlePassInput` reads V8's string length in O(1), allocates the UTF-8
  worst case (three bytes per UTF-16 unit) without zero-filling it, and
  converts in one `napi_get_value_string_utf8` pass.
- `bytesInput` borrows the bytes and validates them as UTF-8, and it copies
  nothing. A caller who reads files as `Buffer` never builds a JavaScript
  string at all.
- `encodeIntoInput` shows the same path for a caller who already holds a
  string.
- `latin1Output` uses `napi_create_string_latin1`, a plain copy with no UTF-8
  decoding. It applies to ASCII HTML only, and it checks for ASCII on every
  call.
- `externalOutput` hands the owned one-shot HTML to V8 as an external Latin-1
  string, which avoids the copy. The garbage collector frees it later. The
  cost of that finalization is charged only when a collection happens during
  a batch.
- `bufferCopyOutput` uses `napi_create_buffer_copy` straight from the borrowed
  HTML. This drops the extra `Vec` copy and the external-buffer finalizer.
- `cachedRenderer` is an upper bound. The reused lane already shows what
  `toHtml` would cost if it rendered with a kept default renderer.

Per [AGENTS.md](../../../AGENTS.md), a change to the public API needs its own
decision record under [docs/decisions/](../../../docs/decisions/).
