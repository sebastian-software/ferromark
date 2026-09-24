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
this record. Measured savings belong to the benchmark reports, not here.

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

## Safety: when the addon borrows

A Rust `&str` promises valid UTF-8 for as long as it lives, and the parser and
renderer rely on that promise, including in `unsafe` code. Borrowing a
JavaScript-owned byte range as a `&str` is only sound while nothing can change
those bytes. All of the addon's `unsafe` code sits in
`node/native/src/input.rs`, the only module allowed to use it, with a `SAFETY`
comment on every block. It borrows the bytes of a `Uint8Array` only when all
of the following hold, and copies them otherwise.

**The bytes are read after the last JavaScript before the render.** napi-rs
converts the Markdown argument first and an `Options` object after it, and
converting options runs their getters — JavaScript that could overwrite,
detach, transfer or resize the buffer. The argument conversion therefore only
records the typed array's handle. The first use of the text in the export body
asks N-API for the view's current data pointer and length, validates the
bytes, and caches the result for the rest of the call. Through the facade, the
options have already been read in JavaScript by then; the rule matters for the
object-taking exports, and a test drives them with getters that overwrite,
detach, shrink and grow the input.

**No JavaScript runs while the borrow lives.** `toHtml`, `toHtmlBuffer`,
`transform`, the `Renderer` methods and their packed forms parse and render in
Rust only. The highlighter entries call the highlighter for every code block,
and that JavaScript could change the input mid-render, so they take a second
argument type, `OwnedUtf8Input`: it reads the bytes at the same point but
copies them before decoding. A highlighter that overwrites, detaches or shrinks
the input changes nothing in that call, which the tests check. Without
JavaScript nothing on the thread can detach or resize the buffer either,
garbage collection does not move an `ArrayBuffer`'s backing store, and N-API
moves a small on-heap typed array's bytes into a backing store before it
returns their address.

**No other thread can write the bytes.** A `Uint8Array` over a
`SharedArrayBuffer` can be written by a worker at any moment. The addon asks
`napi_is_arraybuffer` about the view's buffer, for which V8 answers `true` only
for a non-shared `ArrayBuffer`, and borrows only then. Everything else is
copied first, byte by byte with relaxed atomic loads, since Rust allows memory
that another thread writes to be read only through atomics. A copy can mix old
and new bytes, but it is the call's own memory and is decoded like any other
input. A growable `SharedArrayBuffer` never shrinks, so the length read before
the copy stays readable. The check sits in the addon, not the facade: a
facade check (`bytes.buffer instanceof SharedArrayBuffer`) would go through a
patchable `buffer` getter and a realm-specific constructor, and the native
exports can be called without the facade.

**The borrow cannot outlive the call.** The argument types carry the lifetime
of the call's handles, so no code can move them into anything that outlives
the export.

**Invalid bytes are never a `&str`.** Validation with `str::from_utf8` comes
before the `&str` exists; where it fails, the addon builds an owned string
with the replacements instead.

One case is outside what any addon can detect. JavaScript cannot reach a
non-shared `ArrayBuffer` from another thread, but native code holding its data
pointer can: an unfinished asynchronous `fs.read()` into the same `Buffer`
writes from the thread pool. Rendering a buffer that is still being filled was
never meaningful, and the package README states that it is not supported. The
borrow-or-copy decision is one condition in `input.rs`, so copying every input
instead is a small change should that contract prove too weak.

## Alternatives considered

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

**Copying always.** One copy per call is simpler to argue about and still
avoids the string round trip, but the borrow is what the owner approved and
what the measurement in #435 prototyped. The conditions above make it sound
for everything JavaScript can do; the copy stays the path for shared memory
and for the highlighter entries.

## Verification

Output equality is the binding constraint, and every test compares a bytes
call with the string call of the equivalence rule:

- **Corpus and fixtures:** all seven entries, with and without options, on the
  57 broad-corpus documents and the documents of the other Node tests, each as
  a `Buffer`, a plain `Uint8Array`, a subarray at a non-zero offset and a view
  of a `SharedArrayBuffer` (`test/bytes.test.mjs`).
- **Invalid UTF-8:** the rendered sweep in the test suite and the exact
  conversion sweep in `scripts/verify-input-conversion.mjs`, described above,
  for both argument types, and for every form of `Uint8Array` on the random
  input and every eighth block of the exhaustive sweep.
- **Borrow and copy:** the verifier checks through a `boundary-bench`
  diagnostic that valid bytes of a plain `ArrayBuffer` are borrowed, and that
  shared, growable shared, invalid, empty and detached input is not.
- **Mutation:** highlighters that overwrite, detach or shrink the input, a
  worker writing a `SharedArrayBuffer` while it is rendered, and option getters
  that change the input of the object-taking exports.
- **Rejected values:** the class, code and message for every kind of value,
  that describing it runs none of its code, and the error order.
- **Mutation checks** (not committed): making the highlighter entries borrow,
  reading the bytes before the option getters, borrowing shared memory and a
  per-byte replacement decoder each fail the suite — the first two by crashing
  the test process with a memory fault, which is what the rules above prevent.

The existing Node tests pass unchanged; they match the Markdown type error by
`/string/i`, which the new message still satisfies.
