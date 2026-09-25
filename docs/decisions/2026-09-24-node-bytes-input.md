# Markdown as UTF-8 bytes in the Node package

**Status:** Accepted for the Node package

## Scope

Every Node entry that takes Markdown took a JavaScript string, and the addon
transcoded it to UTF-8 on every call. A caller that already holds UTF-8 — a
file read without an encoding, an HTTP body, a stream chunk — first had to
decode it into a string, which the addon then encoded straight back. The N-API
boundary measurement from #435 prototyped the alternative as its `bytesInput`
candidate: borrow the caller's bytes, validate them and render from them. It
was the one reduction on that list that changes the public API, so it needed
this record. The shipped version copies the bytes before it validates them;
[Safety](#safety-always-copied-then-validated) explains why, with the cost
of the copy. Other measured savings belong to the benchmark reports.

## Decision

### Entries

`toHtml`, `toHtmlBuffer`, `Renderer#toHtml`, `Renderer#toHtmlBuffer`,
`transform`, `toHtmlWithHighlighter` and `transformWithHighlighter` accept
`markdown: string | Uint8Array`. A `Buffer` is a `Uint8Array`, and so is any
subclass or a `Uint8Array` from another realm. Nothing else changes: options,
results and errors of valid calls are those of the string path.

Both the facade and the addon recognize a `Uint8Array` by its brand, not its
prototype: the facade with `util.types.isUint8Array` (Node's own check, which
no getter, proxy or patched prototype can change), the addon with
`napi_get_typedarray_info`, which reports `napi_uint8_array`.

### The equivalence rule

For any `Uint8Array` `bytes`, every entry returns exactly what it returns for
the string

```js
Buffer.from(bytes.buffer, bytes.byteOffset, bytes.byteLength).toString("utf8");
```

A file read as a `Buffer` therefore renders exactly like the same file read
with `'utf8'`. This one rule fixes every edge case below, and the tests check
the rule itself rather than a list of expected outputs.

### Invalid UTF-8

Invalid sequences become U+FFFD the way Node's decoder replaces them. The
addon uses Rust's `String::from_utf8_lossy`, which implements the Unicode
standard's "substitution of maximal subparts": each maximal prefix of a
well-formed sequence, or each lone byte that cannot start one, becomes one
U+FFFD. V8's decoder behind `Buffer#toString('utf8')` follows the same
practice (the WHATWG decoder's). The two were compared, not assumed:

- on every sequence of one, two and three bytes, on four-byte sequences with
  every lead byte from C0 to FF, every second byte and 20 classes for the rest,
  on those sequences back to back without a separator, on every sequence cut
  short at the end of the input, and on seeded random input;
- this covers overlongs (C0 80, E0 80 80, F0 80 80 80), encoded surrogates
  (ED A0 80), code points above U+10FFFF (F4 90 80 80), the bytes F5 to FF,
  lone continuation bytes and truncated sequences before other bytes and at
  the end of the input.

The results were identical on Node.js 24 and 26 before this change was made,
so there was no case in which the addon had to fall back to decoding in
JavaScript, and it has no hand-written decoder. The comparison stays in the
test suite: `scripts/verify-input-conversion.mjs` runs it on the addon's own
conversion in every `pnpm test` (CI: Node.js 22, 24 and 26), and
`test/bytes.test.mjs` compares the rendered output of a smaller sweep on the
Node.js 22.12 floor too. If a Node.js release ever decoded differently, those
checks fail rather than the output drifting.

### Byte order mark

Neither decoder strips a byte order mark: EF BB BF becomes U+FEFF, as
`Buffer#toString` keeps it and unlike `TextDecoder`, which removes one. The
parser already treats one leading U+FEFF as metadata, from a string as from
bytes, so a single mark renders as before and a second one stays content on
both paths. Stripping it in the addon would have broken the equivalence for a
document with two marks.

### Empty, detached and oversized input

A view of no bytes is empty input and renders like `""`: an empty
`Uint8Array`, an empty subarray, a detached buffer (its data pointer is null)
and a view out of bounds of a shrunk resizable buffer. Reading one never
dereferences its pointer.

The parser stores source offsets as `u32`. No JavaScript string reaches 4 GiB,
but a `Uint8Array` can, so input of more than `u32::MAX` bytes throws a
`RangeError` with the code `ERR_OUT_OF_RANGE` before any byte is read.

### Other values, and the changed error

Every other value is rejected: other typed arrays (including
`Uint8ClampedArray`), `DataView`, `ArrayBuffer`, `SharedArrayBuffer`, arrays,
String objects, numbers and the rest. `ArrayBuffer` is not accepted, see the
alternatives below.

The error is a `TypeError` with the code `ERR_INVALID_ARG_TYPE`, the code Node
uses for the same mistake in its own APIs, and the message

```text
markdown must be a string or a Uint8Array, received a Float64Array
```

The message follows the package's own "`x` must be …" style and names the
value's kind: `undefined`, `null`, a boolean, number, symbol, bigint, function,
array, `ArrayBuffer`, `DataView`, each typed array by name, or an object. The
addon asks N-API only for the value's type, so building the message runs no
JavaScript. A `SharedArrayBuffer` reads as "an object", because N-API on
Node.js 22 cannot tell it from one.

This is a deliberate behavior change for every value that is neither a string
nor a `Uint8Array`. Before, napi-rs's `String` conversion threw a plain `Error`
with the code `StringExpected` and a message such as ``Failed to convert
JavaScript value `Number 123 ` into rust type `String` ``. That message now
says the wrong thing, since strings are no longer the only type, and building
it ran user code: `JSON.stringify` for objects, including any `toJSON`, and the
`name` getter of a function. A cyclic object even surfaced `JSON.stringify`'s
own `TypeError` instead, and a large typed array produced a message with every
element in it. Code that matched `/string/i` still matches; code that checked
`error.code === 'StringExpected'` or the exact message does not.

The error order is unchanged: an unknown option key (the facade's
`TypeError`) wins over the Markdown type, which wins over an option value.
The addon converts the Markdown before napi-rs reads any option, and the
facade sends a value it does not recognize as Markdown to the object-taking
export with the options untouched, so no getter runs for a rejected call.

### Options

Options for bytes take the packed path of #439 like options for a string: the
facade reads them in JavaScript and calls the private `…Packed` exports. Only
Markdown that is neither a string nor a `Uint8Array` takes the object path, as
non-string Markdown did before.

### Types

`index.d.mts` declares `markdown: string | Uint8Array` on all seven entries,
and the generated `native.d.ts` does the same for the native exports. Passing
an `ArrayBuffer` is a type error.

## Safety: always copied, then validated

A Rust `&str` promises valid UTF-8 for as long as it lives, and the parser and
renderer rely on that promise, including in `unsafe` code. The bytes of a
`Uint8Array` belong to JavaScript, and code outside the call can change them
while it renders: a highlighter the call invokes, an option getter, a worker
writing a `SharedArrayBuffer`, or native code that still fills the buffer,
such as an unfinished asynchronous `fs.read()` into the same `Buffer`. The
addon therefore never renders from that memory. Every entry copies the bytes
into memory of its own, then validates and decodes the copy. All of the
addon's `unsafe` code sits in `node/native/src/input.rs`, the only module
allowed to use it, with a `SAFETY` comment on every block.

**The bytes are read after the option getters.** napi-rs converts the Markdown
argument first and an `Options` object after it, and converting options runs
their getters, which are JavaScript that can overwrite, detach, transfer or
resize the buffer. The argument conversion therefore only records the typed
array's handle. The first use of the text in the export body asks N-API for
the view's current data pointer and length, copies the bytes and caches the
text for the rest of the call. Through the facade, the options have already
been read in JavaScript by then. Tests drive both the facade and the
object-taking exports with getters that overwrite, detach, shrink and grow the
input, and check that the render matches the bytes as the getter left them.

**The copy fits the memory it comes from.** For a plain `ArrayBuffer`, which
`napi_is_arraybuffer` confirms, the copy is one `memcpy` through raw
pointers. No other JavaScript thread can write that memory, and no JavaScript
runs on this one during the copy. Native code racing on the same buffer can at
most change which bytes are copied, and it cannot make the parser see invalid
UTF-8. For a `SharedArrayBuffer`, for which V8 answers `false`, and for
anything else the check does not confirm, the copy reads each byte with a
relaxed atomic load, since Rust allows memory that another thread writes to be
read only through atomics. A growable `SharedArrayBuffer` never shrinks, so
the length read before the copy stays readable. Either copy may mix bytes from
before and after a concurrent write, and it is validated like any other input.
The check sits in the addon, not the facade. A facade check
(`bytes.buffer instanceof SharedArrayBuffer`) would go through a patchable
`buffer` getter and a realm-specific constructor, and the native exports can
be called without the facade.

**Then it is validated.** `String::from_utf8` takes a valid copy as it is,
and anything else goes through `String::from_utf8_lossy`, so the parser only
ever sees valid UTF-8. The copy's memory is reserved with `try_reserve_exact`,
as the string conversion reserves its buffer. If that allocation fails, the
call throws an error instead of aborting the process.

**The handles stay inside the call.** The argument type carries the lifetime
of the call's handles, so no code can move it into anything that outlives the
export, where the recorded handles would dangle.

Copying costs little next to what the bytes path saves. The owner measured
borrowing against always-copy on Apple M1 Pro with PGO `boundary-bench`
addons, pairing the string and bytes lanes per round and alternating the two
builds over two passes. The machine was under load (15 to 40), so the numbers
are indicative. As the geomean over the 57 broad-corpus documents of the
bytes call's speed over the string call's:

| Entry | Borrow | Copy |
| --- | ---: | ---: |
| `toHtml` | 1.53× | 1.47× |
| `Renderer.toHtml` | 1.53× | 1.49× |
| `toHtmlBuffer` | 1.69× | 1.57× |

The copy costs 3 to 8% of the bytes call, and up to about 10 to 19% of it
below 512 B, where that is tens of nanoseconds. In exchange, no input and no
caller behavior can break the `&str` invariant.

## Alternatives considered

**Borrowing when no JavaScript runs.** The first version of this change
borrowed the bytes of a plain `ArrayBuffer` as a `&str` for the render in the
entries that call no JavaScript while they render, and copied only for the
highlighter entries and shared memory. That is sound against everything
JavaScript can do. It is not sound against native code that holds the
buffer's data pointer and writes from another thread, such as an unfinished
asynchronous `fs.read()` into the same `Buffer`. Such a write during the
render would break the `&str` invariant, which is undefined behavior, not
just wrong output. No addon can detect that case, so the borrow needed a
documented caller contract. With a copy, the same race can only produce odd
text. The measurement above puts the price of dropping the contract at a few
percent of the bytes call, so the borrow was rejected.

**Separate entries (`toHtmlFromBytes` and friends).** Seven more functions and
two more `Renderer` methods, each needing its own packed native export, type
declarations and documentation, for a choice the argument's type already
makes. Node APIs such as `fs.writeFile`, `crypto.hash` and
`Buffer.byteLength` take a string or bytes in one parameter, so a union is the
idiom callers expect, and it keeps "same output" visibly one contract.

**`TextEncoder.encodeInto` in the facade.** Encoding strings into a kept
scratch buffer and handing the addon bytes changes no API, but a caller that
holds bytes gains nothing from it: it still decodes to a string first. The
boundary benchmark keeps this as the `encodeIntoInput` candidate for strings.

**Accepting `ArrayBuffer` (and `DataView`, other typed arrays).** An
`ArrayBuffer` carries no view, so the addon would have to define its offset and
length, and a `SharedArrayBuffer` would need the same treatment through a
separate code path. `new Uint8Array(buffer)` makes the intent explicit at no
cost, and rejecting everything but `Uint8Array` keeps the type check a single
brand test. Other typed arrays are not byte sequences in any useful sense.

**Decoding invalid input in JavaScript.** Falling back to
`Buffer#toString('utf8')` for invalid bytes would guarantee Node's decoder by
construction, at the price of a second path through the facade and a string
round trip for exactly the inputs that need care. The comparison above showed
Rust's decoder to be identical, and the verifier keeps checking it.

## Verification

Output equality is the binding constraint, and every test compares a bytes
call with the string call of the equivalence rule:

- **Corpus and fixtures:** all seven entries, with and without options, on the
  57 broad-corpus documents and the documents of the other Node tests, each as
  a `Buffer`, a plain `Uint8Array`, a subarray at a non-zero offset and a view
  of a `SharedArrayBuffer` (`test/bytes.test.mjs`). The kept default renderer's
  A-B-A sequences run with bytes as well (`test/default-renderer.test.mjs`).
- **Invalid UTF-8:** the rendered sweep in the test suite and the exact
  conversion sweep in `scripts/verify-input-conversion.mjs`, described above,
  for every form of `Uint8Array` on the random input and every eighth block of
  the exhaustive sweep.
- **Copy paths:** the verifier checks through a `boundary-bench` diagnostic
  that plain buffers are copied with `memcpy`, shared and growable shared ones
  with atomic loads, and that empty and detached views read no bytes.
- **Mutation:** highlighters that overwrite, detach or shrink the input, a
  worker writing a `SharedArrayBuffer` while it is rendered, and option getters
  that change the input through the facade and through the object-taking
  exports.
- **Rejected values:** the class, code and message for every kind of value,
  that describing it runs none of its code, and the error order.
- **Mutation checks** (not committed): reading the bytes before the option
  getters, copying shared memory with `memcpy` and a per-byte replacement
  decoder each fail the suite.

The existing Node tests pass unchanged; they match the Markdown type error by
`/string/i`, which the new message still satisfies.
