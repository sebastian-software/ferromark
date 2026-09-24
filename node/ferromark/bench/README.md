# N-API boundary measurement

`boundary.mjs` measures how much of a Node.js call goes to the N-API boundary
and how much goes to the Rust core. It covers `toHtml` (one-shot),
`Renderer.toHtml` (reused), `toHtmlBuffer` and `Renderer.toHtmlBuffer`, and
splits each call into four parts:

- **input**: the JavaScript string becomes a Rust `String`. The exports read
  V8's stored UTF-16 length, reserve three UTF-8 bytes per unit and call
  `napi_get_value_string_utf8` once (`Utf8Input` in
  [`node/native/src/input.rs`](../../native/src/input.rs)). V8 transcodes its
  Latin-1 or UTF-16 storage on the way. Before that, napi-rs's `String`
  conversion called it twice: a UTF-8 length pass, then a copy into a
  zero-filled buffer. Markdown passed as UTF-8 bytes (a `Buffer` or other
  `Uint8Array`) skips the transcoding: the exports copy the bytes into memory
  of their own and validate the copy. The [bytes lanes](#bytes-input) time
  `toHtml`, `Renderer.toHtml` and `toHtmlBuffer` that way next to the string
  calls.
- **core**: arena, parse and render in Rust.
- **output**: `napi_create_string_utf8` decodes the HTML into a V8 string.
  `toHtmlBuffer` copies the HTML into a `Vec` and wraps it in an external
  `Buffer`.
- **fixed**: the bare N-API call. Without options, a one-shot call renders
  on a renderer its thread keeps (see
  [`node/native/src/default_renderer.rs`](../../native/src/default_renderer.rs)),
  and taking that renderer out and back counts as core. With options, or
  with Markdown over 64 KiB, a one-shot call also pays for `Renderer::new`,
  as every one-shot call did before. Option handling is reported separately
  because the default call passes no options.

The benchmark times the four public exports with the same addon that carries
the diagnostic exports, so all lanes run in one binary. The per-document lanes
call the exports directly, without the JavaScript facade in `index.mjs`. With
no options, the facade adds only `validateOptions(undefined)` and a cached
`loadNative()`. With options, it reads and packs them itself (see
[Options](#options)), so the fixed table also times the facade, loaded against
the same addon.

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

To compare two revisions, build one addon from each checkout into its own
directory and pass each directory with `--addon`, together with that
checkout's `node/ferromark/` as `--facade`. The script skips diagnostic
exports that an older addon lacks, so one checkout's script can time both.

## Run

From `node/ferromark/`:

```sh
node --expose-gc bench/boundary.mjs --json ../../target/boundary-bench/boundary.json
```

| Flag               | Default                                                            | Meaning                                                               |
| ------------------ | ------------------------------------------------------------------ | --------------------------------------------------------------------- |
| `--addon <path>`   | `target/boundary-bench/`                                           | The diagnostic addon, or a directory holding exactly one `.node` file |
| `--facade <dir>`   | `node/ferromark/`                                                  | The directory whose `index.mjs` the facade lanes run                  |
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
public exports, with the string and with its bytes, the probe's HTML and
`Buffer`, and every candidate must return the same HTML. The core loops must
match the HTML length, and each input path must match `Buffer.byteLength`. The
exports' input conversion must return the same bytes as napi-rs's `String`
conversion and `Buffer.from`.

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
| `toHtmlBytes`, `toHtmlBufferBytes`       | `toHtml` and `toHtmlBuffer` with the document as a UTF-8 `Buffer`                     |
| `rendererToHtmlBytes`                    | `Renderer.toHtml` with the document as a UTF-8 `Buffer`                               |
| `noop`                                   | `boundaryNoop()`: the free-function call floor                                        |
| `methodNoop`                             | `BoundaryProbe#noop()`: the method call floor, including the unwrap of `this`         |
| `len`                                    | `boundaryLen(markdown)`: the call plus input conversion                               |
| `lenNapiString`                          | `boundaryLenNapiString(markdown)`: the same with napi-rs's `String` conversion        |
| `echo`                                   | `boundaryEcho(markdown)`: input plus output conversion of the Markdown                |
| `make`                                   | `boundaryMake(n)`: output conversion of `n` ASCII bytes, `n` = HTML length            |
| `html`, `htmlBuffer`                     | the exact HTML as `Renderer.toHtml` and `toHtmlBuffer` return it, rendered beforehand |
| `coreDefault`, `coreFresh`               | `boundaryCoreOnly(markdown, k, lifecycle)`: the export's own Rust code, `k` times     |
| `coreReuse`, `coreSetup`                 | the same for the other lifecycles                                                     |

`boundaryCoreOnly` calls the same `render_one_shot`, `render_fresh` and
`Renderer::render_reused` helpers as the exports. `default` is the one-shot
lifecycle without options: the thread's kept renderer, or a renderer of its own
for Markdown over 64 KiB. `fresh` is the one-shot lifecycle with a renderer of
its own, which calls with options take and every one-shot call took before.
`reuse` is a renderer kept across calls, and `setup` is only `Renderer::new`
and its drop. The script subtracts one `len` (or `noop`) call of the same round
before dividing by `k`. An addon built before the kept renderer has no
`default` lifecycle; the script then leaves out `coreDefault` and splits
`toHtml` as that addon runs it.

Each API is split as follows. The `total` is the public lane.

| Part     | `toHtml`, `toHtmlBuffer`                         | `Renderer.toHtml`, `Renderer.toHtmlBuffer` |
| -------- | ------------------------------------------------ | ------------------------------------------ |
| input    | `len - noop`                                     | `len - noop`                               |
| core     | `coreDefault` (before: `coreFresh - coreSetup`)  | `coreReuse`                                |
| output   | `html - methodNoop` or `htmlBuffer - methodNoop` | the same                                   |
| fixed    | `noop` (before: `noop + coreSetup`)              | `methodNoop`                               |
| residual | `total - input - core - output - fixed`          | the same                                   |

The residual is what the model does not explain: `catch_unwind`, the
`undefined` options check, cache effects between the parts, and noise. If it
stays large, a part is missing from the model. The **N-API** column adds up
input, output and the bare call floor.

### Options

napi-rs converts an `Options` object field by field: one
`napi_get_named_property` and one `napi_typeof` for each of its 30 fields,
present or not. The facade therefore reads the object itself, in the same order
and with the same property gets, and passes the private `…Packed` exports a
bitmask for `renderPolicy` and the boolean fields plus the three other values
([`node/native/src/packed.rs`](../../native/src/packed.rs)). The fixed table
times both paths:

| Lane                                | What it runs                                                          |
| ----------------------------------- | --------------------------------------------------------------------- |
| `optionsNone`, `optionsEmpty`       | `boundaryOptions()` and `boundaryOptions({})`: the object path        |
| `optionsTrusted`                    | `boundaryOptions({ renderPolicy: 'trusted' })`                        |
| `optionsPackedNone`                 | `boundaryOptionsPacked(0, 0)`: the packed path, native side only      |
| `optionsPackedTrusted`              | `boundaryOptionsPacked(1, 1)`, the packed `{ renderPolicy }`          |
| `toHtmlEmpty`, `toHtmlEmptyOptions` | the `toHtml` export with `''`, without and with `{}`                  |
| `toHtmlTrusted`                     | the same with `{ renderPolicy: 'trusted' }`                           |
| `rendererConstruct`                 | `new Renderer()` on the addon's class                                 |
| `rendererConstructTrusted`          | the same with `{ renderPolicy: 'trusted' }`                           |
| `facadeToHtmlEmpty`                 | the facade's `toHtml('')`                                             |
| `facadeToHtmlEmptyOptions`          | the facade's `toHtml('', {})`: validation, packing and `toHtmlPacked` |
| `facadeToHtmlTrusted`               | the same with `{ renderPolicy: 'trusted' }`                           |
| `facadeRendererTrusted`             | the facade's `new Renderer({ renderPolicy: 'trusted' })`              |

The facade lanes copy `index.mjs` and `native-target.mjs` from `--facade` into
a temporary directory next to a link to the addon, which the facade then loads
as its local binary. Before timing, the script checks that the facade calls
into that same addon and renders as the object-taking export does, and that
packed and object options resolve alike. An addon from before the packed
exports lacks the packed lanes; its facade still has the facade lanes.

The second table pairs each object-path lane with its packed counterpart per
round. The object side of the `toHtml` and `Renderer` rows is what the facade
called before it packed options, minus its `validateOptions`, which both
versions run and the packed side includes. Packed options with no field
present are the defaults, so the packed side of `toHtml('', {})` also renders
on the kept renderer, while the object side builds a renderer of its own.

## Reading the output

1. **Fixed per-call costs**: the call floors, option handling (the cost of an
   `Options` object is independent of the document), `new Renderer()` from
   JavaScript and `Renderer::new` inside Rust. A second table sets the object
   path of options (before) against the packed path (after), with the paired
   saving.
2. **Per document: time per call**: the median public lanes and parts in
   nanoseconds. `input` shows the characters the Markdown contains and how V8
   stores it: `ascii`, `latin1` (up to U+00FF) or `wide`, and `one-byte` or
   `two-byte`. A `*` marks non-ASCII HTML. `bytes` is `toHtml` with the
   document as a `Buffer`.
3. **Per document: share of each call**: input/core/output/fixed/residual as
   percentages of each API's median time.
4. **Median share per size bin** for each API, and per input representation
   for the string APIs. Documents have equal weight.
5. **Bytes input**: see [below](#bytes-input).
6. **Candidates**: the median saving per call of each reduction prototype per
   size bin. Each saving is paired per round. In parentheses is the saving's
   share of the call named in the table below.

V8 stores a string as one-byte (Latin-1) when every character fits, otherwise
as two-byte (UTF-16). Conversion cost follows the storage, not just the
length. In the broad corpus, 30 inputs are ASCII, 4 are Latin-1 and 23 need
two-byte storage. `--two-byte` stores every input as UTF-16 with the content
unchanged, so running with and without it compares the two storage paths on
identical text.

### Bytes input

Every export that takes Markdown also takes it as UTF-8 bytes
([decision record](../../../docs/decisions/2026-09-24-node-bytes-input.md)).
The bytes lanes call `toHtml`, `Renderer.toHtml` and `toHtmlBuffer` with the
document as a `Buffer`, encoded once before timing, so one build times a caller
that holds the string against one that holds the bytes, for example from
`fs.readFile` without an encoding. Every bytes call copies the bytes before it
validates and renders them, so the bytes lanes include that copy. The table
pairs each API's string lane with its bytes lane per round and reports the
median saving per size bin, and its share of the string call. `--two-byte`
changes only the string lanes; the bytes are the same.

An addon built before the exports took bytes has no bytes lanes, and the table
shows `n/a`.

## Candidates

The `boundary-bench` feature also prototypes reductions. They are
measurements, not API.

| Candidate          | Lane                             | Compared with   | Share of          | Public API change                                     |
| ------------------ | -------------------------------- | --------------- | ----------------- | ----------------------------------------------------- |
| `singlePassInput`  | `boundaryLen`                    | `lenNapiString` | `toHtml`          | none: shipped as the exports' conversion              |
| `bytesInput`       | `boundaryLen(Buffer)`            | `len`           | `toHtml`          | shipped: every export takes UTF-8 bytes               |
| `encodeIntoInput`  | `TextEncoder.encodeInto` + bytes | `len`           | `toHtml`          | none if the facade does it with a kept scratch buffer |
| `latin1Output`     | `BoundaryProbe#htmlLatin1`       | `html`          | `Renderer.toHtml` | none: an internal conversion for ASCII HTML           |
| `externalOutput`   | `boundaryToHtmlExternal`         | `toHtmlFresh`   | `toHtml`          | none: an internal conversion for ASCII HTML           |
| `bufferCopyOutput` | `BoundaryProbe#htmlBufferCopy`   | `htmlBuffer`    | `toHtmlBuffer`    | none: an internal conversion                          |
| `cachedRenderer`   | `toHtml`                         | `toHtmlFresh`   | `toHtml`          | none: shipped as the one-shot calls' kept renderer    |

- `singlePassInput` has shipped. The exports read V8's string length in O(1),
  reserve the UTF-8 worst case (three bytes per UTF-16 unit) without
  zero-filling it, and convert in one `napi_get_value_string_utf8` pass. Its
  column now pairs napi-rs's `String` conversion (`boundaryLenNapiString`,
  the path before) with the exports' own (`len`), so it reports the saving of
  the switch within one build. An addon built before the switch lacks
  `boundaryLenNapiString`, and the column shows `n/a` for it.
- `bytesInput` has shipped as public API: every export takes UTF-8 bytes,
  copies them and validates the copy. The `boundaryBytesLen` prototype
  borrowed them instead, which the shipped conversion does not. A caller who
  reads files as `Buffer` never builds a JavaScript string at all. Its column pairs
  the string conversion (`len`) with the exports' own conversion of the bytes,
  `boundaryLen(Buffer)`, which replaced the `boundaryBytesLen` prototype. On an
  addon from before, which still has the prototype, the column times that
  instead. The [bytes lanes](#bytes-input) show the whole calls.
- `encodeIntoInput` shows the same path for a caller who already holds a
  string: `TextEncoder.encodeInto` into a kept scratch buffer, then the bytes
  conversion.
- `latin1Output` uses `napi_create_string_latin1`, a plain copy with no UTF-8
  decoding. It applies to ASCII HTML only, and it checks for ASCII on every
  call.
- `externalOutput` hands the owned one-shot HTML to V8 as an external Latin-1
  string, which avoids the copy. The garbage collector frees it later. The
  cost of that finalization is charged only when a collection happens during
  a batch. Only a renderer of its own can hand its buffer away, so the column
  pairs it with `toHtmlFresh` (`boundaryToHtmlFresh`, below); with an addon
  from before the kept renderer, it pairs it with `toHtml`, which was that
  path.
- `bufferCopyOutput` uses `napi_create_buffer_copy` straight from the borrowed
  HTML. This drops the extra `Vec` copy and the external-buffer finalizer.
- `cachedRenderer` has shipped. One-shot calls without options render on a
  renderer their thread keeps
  ([`node/native/src/default_renderer.rs`](../../native/src/default_renderer.rs)).
  Its column now pairs `toHtml` as it ran before (`boundaryToHtmlFresh`: a
  renderer of its own, whose buffer `HtmlRenderer::render` hands over) with
  the export, so it reports the saving of the switch within one build. An
  addon built before the switch lacks `boundaryToHtmlFresh`, and the column
  shows `n/a` for it. Before the switch, the column compared `toHtml` with
  `Renderer.toHtml`, which also counted the method call floor against the
  kept renderer.

Per [AGENTS.md](../../../AGENTS.md), a change to the public API needs its own
decision record under [docs/decisions/](../../../docs/decisions/).
