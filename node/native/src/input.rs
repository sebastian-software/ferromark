//! Conversion of a JavaScript Markdown argument to UTF-8 text.
//!
//! Every export that takes Markdown accepts a string or a `Uint8Array`, which
//! includes a Node.js `Buffer`, holding UTF-8:
//!
//! - A **string** converts in one N-API pass (see [Strings](#strings)).
//! - **Bytes** become the text `Buffer#toString('utf8')` makes of them (see
//!   [Bytes](#bytes)). Valid UTF-8 stays as it is, a leading byte order mark
//!   stays U+FEFF, and each maximal invalid subpart becomes one U+FFFD.
//! - **Anything else** fails with a `TypeError` whose `code` is
//!   `ERR_INVALID_ARG_TYPE`. The message names the kind of value it received,
//!   and describing it runs no JavaScript.
//!
//! [`Utf8Input`] reads a `Uint8Array` when the export first uses the text,
//! after napi-rs has converted every argument, and always copies the bytes
//! before it validates and decodes them. The text an export renders is
//! therefore always its own memory.
//!
//! This is the addon's only module with `unsafe` code. Every block states why
//! it is sound.
//!
//! # Strings
//!
//! napi-rs converts a `String` argument with two `napi_get_value_string_utf8`
//! calls. The first only measures the UTF-8 length, which V8 computes by
//! walking the whole string, and the second transcodes into a zero-filled
//! buffer of exactly that length. The conversion here asks for the UTF-16
//! length instead, which V8 stores, reserves the worst case without filling
//! it, and transcodes once. Both paths use N-API's transcoder, and with enough
//! room it writes the same bytes. The unused part of the reservation is never
//! written, and every export drops the string before it returns.
//!
//! N-API turns each UTF-16 code unit of the string into at most three UTF-8
//! bytes:
//!
//! - U+0000 to U+007F take one byte and U+0080 to U+07FF two. That covers
//!   every character of a string V8 stores as one-byte (Latin-1).
//! - Every other unit takes three, including a lone surrogate, which N-API
//!   replaces with U+FFFD (three bytes).
//! - A surrogate pair takes four bytes for its two units.
//!
//! So `3 * units` bytes hold any string. The largest V8 string has
//! `v8::String::kMaxLength` (2^29 - 24) units on 64-bit targets, so the
//! reservation stays below 1.7 GB. That also fits the `int` capacity Node.js
//! 22 passes on to V8.
//!
//! The reservation adds `MAX_UTF8_CHAR` spare bytes, so each conversion also
//! proves that it is complete. N-API stops before the end of a string only when
//! the next character does not fit into the room left, and no character takes
//! more than four bytes. A conversion that leaves four bytes unused has
//! therefore written the whole string. One that does not, which the bound rules
//! out for V8, is redone with napi-rs's exact two-pass conversion, so nothing is
//! truncated even on an N-API implementation that broke the bound.
//!
//! # Bytes
//!
//! A `Uint8Array` is accepted by its N-API type, `napi_uint8_array`, which a
//! `Buffer` and any other subclass report too. Other typed arrays, a
//! `DataView` and an `ArrayBuffer` are rejected.
//!
//! Rust's `String::from_utf8_lossy` replaces invalid UTF-8 the way the Unicode
//! standard recommends ("substitution of maximal subparts"), and so does V8's
//! decoder behind `Buffer#toString('utf8')`. `verify-input-conversion.mjs`
//! checks the two against each other on every sequence of up to three bytes,
//! on four-byte sequences across all byte classes and on seeded random input.
//! Neither strips a byte order mark; the parser treats one leading U+FEFF as
//! metadata, whichever path the text came from.
//!
//! A view of no bytes is empty input. That includes a detached buffer, whose
//! data pointer is null, and a view out of bounds of a shrunk resizable
//! buffer. Input above `MAX_BYTES` (`u32::MAX`) fails with a `RangeError`
//! (code `ERR_OUT_OF_RANGE`), because the parser stores offsets as `u32`; no
//! JavaScript string can be that long, so only bytes need the check.
//!
//! ## Reading and copying
//!
//! A `&str` promises valid UTF-8 for as long as it lives, and the parser and
//! renderer rely on that promise, including in `unsafe` code. JavaScript owns
//! a `Uint8Array`'s memory, and code outside this call can change it: a
//! highlighter the export calls, an option getter, a worker writing a
//! `SharedArrayBuffer`, or native code that still fills the buffer, such as an
//! unfinished asynchronous `fs.read`. So the text is never a borrow of that
//! memory:
//!
//! 1. **The bytes are read when the export first uses them**, after napi-rs has
//!    converted every argument. Converting an `Options` object runs its
//!    getters, which are JavaScript and can overwrite, detach or resize the
//!    buffer. The argument conversion only records the handle; the first
//!    dereference asks N-API for the view's current data and length, and
//!    caches the text for the rest of the call.
//! 2. **They are copied, then validated.** The copy is the call's own memory,
//!    so nothing that runs later, and nothing another thread does meanwhile,
//!    can change the text the parser sees. Bytes of a plain `ArrayBuffer`,
//!    which `napi_is_arraybuffer` confirms, are copied with one `memcpy`: only
//!    this thread's JavaScript can write them, and none runs during the copy.
//!    Native code that races on the same buffer can at most change which
//!    bytes are copied. Bytes of a `SharedArrayBuffer`, for which V8 answers
//!    false, and of anything else the check does not confirm, are copied with
//!    relaxed atomic loads, because Rust allows memory that another thread
//!    writes at the same time to be read only through atomics. A growable
//!    `SharedArrayBuffer` never shrinks, so its first `length` bytes stay
//!    readable for the copy. Either copy may mix bytes from before and after a
//!    concurrent write, and is then validated like any other input: valid
//!    UTF-8 becomes the text as it is, and anything else the text with its
//!    replacements.
//! 3. **The copy's memory is reserved fallibly**, with `try_reserve_exact`, as
//!    the string conversion reserves its buffer. If the allocation fails, the
//!    export throws an error instead of aborting the process. Decoding invalid
//!    input allocates as `String::from_utf8_lossy` does.
//! 4. **The handles stay inside the call.** `Utf8Input` carries the lifetime of
//!    the call's argument handles, so it cannot be moved into anything that
//!    outlives the export, where the recorded handles would dangle.

// The only module of the addon allowed to use `unsafe`; see above.
#![allow(unsafe_code)]

use std::cell::OnceCell;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;
use std::sync::atomic::{AtomicU8, Ordering};

use napi::bindgen_prelude::{Error, FromNapiValue, Result, Status};
use napi::sys;

/// UTF-8 bytes per UTF-16 code unit, at most. See the module documentation.
const MAX_UTF8_PER_UNIT: usize = 3;

/// UTF-8 bytes per character, at most: a code point above U+FFFF.
const MAX_UTF8_CHAR: usize = 4;

/// The largest buffer the single pass hands N-API. Node.js 22 passes
/// `bufsize - 1` to V8 as an `int`, and a larger size could wrap.
const MAX_CAPACITY: usize = i32::MAX as usize;

/// The most Markdown bytes an export accepts: the parser stores source offsets
/// as `u32`.
const MAX_BYTES: usize = u32::MAX as usize;

/// A Markdown argument: a string converted to UTF-8, or a `Uint8Array` whose
/// bytes the export copies when it first uses the text.
///
/// It holds exactly the text `Buffer#toString('utf8')` makes of the bytes, and
/// for a string exactly the bytes napi-rs's `String` conversion produces.
/// Exports declare it with `#[napi(ts_arg_type = "string | Uint8Array")]`.
pub struct Utf8Input<'call> {
    source: Source,
    /// Ties the value to the call whose handles `Source::Bytes` keeps.
    call: PhantomData<&'call ()>,
}

enum Source {
    /// Text the value owns: a converted string.
    Text(String),
    /// A `Uint8Array`, read at the first dereference.
    Bytes(Bytes),
}

/// The handles of a `Uint8Array` argument and, once read, its text.
struct Bytes {
    env: sys::napi_env,
    value: sys::napi_value,
    text: OnceCell<String>,
}

impl Utf8Input<'_> {
    /// The text, owned.
    pub fn into_string(self) -> String {
        match self.source {
            Source::Text(text) => text,
            Source::Bytes(bytes) => {
                bytes.text();
                bytes.text.into_inner().unwrap_or_default()
            }
        }
    }

    /// How the text is obtained: `"string"`, or for a `Uint8Array` `"empty"`
    /// (no bytes to read), `"copied"` (one `memcpy` from a plain
    /// `ArrayBuffer`) or `"copied atomically"` (relaxed atomic loads, for a
    /// `SharedArrayBuffer`). For the boundary diagnostics.
    #[cfg(feature = "boundary-bench")]
    pub fn origin(&self) -> &'static str {
        match &self.source {
            Source::Text(_) => "string",
            // SAFETY: the handles of the current call, as for `Bytes::text`.
            Source::Bytes(bytes) => match unsafe { view(bytes.env, bytes.value) } {
                View::Empty => "empty",
                View::Private(..) => "copied",
                View::Shared(..) => "copied atomically",
            },
        }
    }
}

impl Deref for Utf8Input<'_> {
    type Target = str;

    fn deref(&self) -> &str {
        match &self.source {
            Source::Text(text) => text,
            Source::Bytes(bytes) => bytes.text(),
        }
    }
}

impl Bytes {
    /// Reads and copies the bytes on first use, then returns the same text for
    /// the rest of the call.
    fn text(&self) -> &str {
        // SAFETY: `self.env` and `self.value` are the handles of the current
        // call: `Bytes` only comes from `Utf8Input::from_napi_value`, whose
        // lifetime keeps it inside the export. `value` is a `Uint8Array`.
        self.text
            .get_or_init(|| unsafe { read(self.env, self.value) })
    }
}

impl FromNapiValue for Utf8Input<'_> {
    unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> Result<Self> {
        // SAFETY: napi-rs passes the `env` and `value` handles of the current
        // call, and both stay valid for it.
        let source = match unsafe { classify(env, value) }? {
            // SAFETY: the same handles; `value` is a string of `units` units.
            Kind::String(units) => Source::Text(unsafe { convert_string(env, value, units) }?),
            // The bytes are read when the export first uses them.
            Kind::Bytes => Source::Bytes(Bytes {
                env,
                value,
                text: OnceCell::new(),
            }),
        };
        Ok(Self {
            source,
            call: PhantomData,
        })
    }
}

/// What a Markdown argument holds.
enum Kind {
    /// A string of this many UTF-16 code units.
    String(usize),
    /// A `Uint8Array` of at most `MAX_BYTES` bytes.
    Bytes,
}

/// Tells a string from a `Uint8Array`, and throws the argument's `TypeError`
/// or `RangeError` for anything else.
///
/// # Safety
///
/// `env` and `value` must be the valid handles of the current N-API call.
unsafe fn classify(env: sys::napi_env, value: sys::napi_value) -> Result<Kind> {
    let mut units = 0usize;
    // SAFETY: the caller passes the handles of the current call. A null
    // buffer asks only for the length, which N-API writes to `units`.
    let status =
        unsafe { sys::napi_get_value_string_utf16(env, value, ptr::null_mut(), 0, &raw mut units) };
    if status == sys::Status::napi_ok {
        return Ok(Kind::String(units));
    }

    let mut kind = -1;
    let mut length = 0usize;
    // SAFETY: the same handles. N-API answers `false` for any value that is
    // not a typed array.
    let typed = holds(|result| unsafe { sys::napi_is_typedarray(env, value, result) });
    // SAFETY: the same handles, and `value` is a typed array. With null data
    // and buffer pointers, N-API only reports the element type and length and
    // does not materialize the buffer.
    if typed
        && unsafe {
            sys::napi_get_typedarray_info(
                env,
                value,
                &raw mut kind,
                &raw mut length,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        } != sys::Status::napi_ok
    {
        kind = -1;
    }
    if kind != sys::TypedarrayType::uint8_array {
        // SAFETY: the same handles.
        let received = unsafe { describe(env, value) };
        let message = format!("markdown must be a string or a Uint8Array, received {received}");
        // SAFETY: the same call.
        return Err(unsafe { throw(env, Throw::TypeError, &message) });
    }
    if length > MAX_BYTES {
        // SAFETY: the same call.
        return Err(unsafe { throw(env, Throw::RangeError, &too_long(length)) });
    }
    Ok(Kind::Bytes)
}

/// Converts a string of `units` UTF-16 code units in one pass, as the module
/// documentation describes under "Strings".
///
/// # Safety
///
/// `env` and `value` must be the valid handles of the current N-API call, and
/// `value` must be a string of `units` code units.
unsafe fn convert_string(
    env: sys::napi_env,
    value: sys::napi_value,
    units: usize,
) -> Result<String> {
    if units == 0 {
        return Ok(String::new());
    }

    // The bound, then the spare bytes that prove completeness and the NUL
    // terminator N-API always writes. No V8 string needs more than
    // `MAX_CAPACITY`; past it, or when the worst case cannot be allocated but
    // the exact size still might be, the exact path converts.
    let bound = units.saturating_mul(MAX_UTF8_PER_UNIT);
    let capacity = bound.saturating_add(MAX_UTF8_CHAR + 1);
    let mut bytes = Vec::<u8>::new();
    if capacity > MAX_CAPACITY || bytes.try_reserve_exact(capacity).is_err() {
        // SAFETY: the same handles, still within the call.
        return unsafe { exact(env, value) };
    }

    let mut written = 0usize;
    // SAFETY: `bytes` owns at least `capacity` writable bytes, the size passed
    // as `bufsize`. N-API writes at most `bufsize - 1` UTF-8 bytes, then a NUL
    // terminator, and reports the UTF-8 byte count in `written`.
    let status = unsafe {
        sys::napi_get_value_string_utf8(
            env,
            value,
            bytes.as_mut_ptr().cast(),
            capacity,
            &raw mut written,
        )
    };
    if status != sys::Status::napi_ok || written > bound {
        // SAFETY: the same handles, still within the call.
        return unsafe { exact(env, value) };
    }

    // SAFETY: N-API initialized the first `written` bytes, and `written` is at
    // most `bound`, below the reserved capacity. At least `MAX_UTF8_CHAR` bytes
    // of room stayed unused, so N-API converted the whole string rather than
    // stopping at a character that did not fit. A whole conversion is valid
    // UTF-8, because N-API replaces lone surrogates with U+FFFD.
    unsafe {
        bytes.set_len(written);
        Ok(String::from_utf8_unchecked(bytes))
    }
}

/// napi-rs's own `String` conversion: a UTF-8 length pass, then the copy.
///
/// It handles every string the single pass leaves out.
///
/// # Safety
///
/// `env` and `value` must be the valid handles of the current N-API call, and
/// `value` must be a string.
#[cold]
#[inline(never)]
unsafe fn exact(env: sys::napi_env, value: sys::napi_value) -> Result<String> {
    // SAFETY: the caller passes the handles of the current call.
    unsafe { String::from_napi_value(env, value) }
}

/// A `Uint8Array`'s bytes as N-API reports them now.
enum View {
    /// No bytes: an empty, detached or out-of-bounds view.
    Empty,
    /// Bytes of a plain `ArrayBuffer`, which no other JavaScript thread can
    /// write.
    Private(*const u8, usize),
    /// Bytes another thread may write: a `SharedArrayBuffer`'s, or those of
    /// any buffer N-API does not confirm as a plain `ArrayBuffer`.
    Shared(*const u8, usize),
}

/// Asks N-API for a `Uint8Array`'s current data, length and buffer kind.
///
/// It panics if N-API fails for a typed array, or if the view has grown past
/// `MAX_BYTES` since its conversion, which only a resizable buffer resized by
/// an option getter can do. Every export catches panics and throws them as
/// JavaScript errors.
///
/// # Safety
///
/// `env` and `value` must be the valid handles of the current N-API call, and
/// `value` must be a typed array.
unsafe fn view(env: sys::napi_env, value: sys::napi_value) -> View {
    let mut length = 0usize;
    let mut data = ptr::null_mut();
    let mut buffer = ptr::null_mut();
    // SAFETY: the caller passes the handles of a typed array in the current
    // call. N-API writes its element count, its data pointer (the buffer's
    // data plus the view's offset, null once detached) and its buffer.
    let status = unsafe {
        sys::napi_get_typedarray_info(
            env,
            value,
            ptr::null_mut(),
            &raw mut length,
            &raw mut data,
            &raw mut buffer,
            ptr::null_mut(),
        )
    };
    assert!(
        status == sys::Status::napi_ok,
        "ferromark could not read the Markdown bytes (N-API status {status})"
    );
    if length == 0 || data.is_null() {
        return View::Empty;
    }
    assert!(length <= MAX_BYTES, "{}", too_long(length));
    // SAFETY: the same call; `buffer` is the handle N-API just returned. V8
    // reports a `SharedArrayBuffer` as not an `ArrayBuffer`.
    if holds(|result| unsafe { sys::napi_is_arraybuffer(env, buffer, result) }) {
        View::Private(data.cast_const().cast(), length)
    } else {
        View::Shared(data.cast_const().cast(), length)
    }
}

/// Reads a `Uint8Array`: copies its current bytes, then decodes the copy.
///
/// # Safety
///
/// `env` and `value` must be the valid handles of the current N-API call, and
/// `value` must be a typed array.
unsafe fn read(env: sys::napi_env, value: sys::napi_value) -> String {
    // SAFETY: the caller's handles.
    let bytes = match unsafe { view(env, value) } {
        View::Empty => return String::new(),
        // SAFETY: N-API reported `length` readable bytes at `data`.
        View::Private(data, length) => unsafe { copy_private(data, length) },
        // SAFETY: the same.
        View::Shared(data, length) => unsafe { copy_shared(data, length) },
    };
    String::from_utf8(bytes)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())
}

/// Reserves `length` bytes for a copy, or panics with a message the export
/// throws as a JavaScript error, rather than aborting the process.
fn reserve(length: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    let reserved = bytes.try_reserve_exact(length);
    assert!(
        reserved.is_ok(),
        "ferromark could not allocate {length} bytes to copy the Markdown"
    );
    bytes
}

/// Copies the bytes of a plain `ArrayBuffer` with one `memcpy`.
///
/// # Safety
///
/// `data` must point to `length` readable bytes that stay allocated for the
/// copy and that no JavaScript can write during it.
unsafe fn copy_private(data: *const u8, length: usize) -> Vec<u8> {
    let mut bytes = reserve(length);
    // SAFETY: `data` points to `length` readable bytes, and `bytes` owns at
    // least `length` bytes of fresh capacity, so the ranges cannot overlap.
    // The copy goes through raw pointers and never forms a reference to the
    // JavaScript memory. Once it has written them, the first `length` bytes
    // are initialized.
    unsafe {
        ptr::copy_nonoverlapping(data, bytes.as_mut_ptr(), length);
        bytes.set_len(length);
    }
    bytes
}

/// Copies bytes that another thread may be writing.
///
/// Rust allows memory that another thread writes at the same time to be read
/// only through atomics, so each byte is read with a relaxed atomic load. The
/// copy may mix bytes from before and after a concurrent write.
///
/// # Safety
///
/// `data` must point to `length` bytes that stay allocated for the copy.
unsafe fn copy_shared(data: *const u8, length: usize) -> Vec<u8> {
    let mut bytes = reserve(length);
    for index in 0..length {
        // SAFETY: `index` is below `length`, so the byte is allocated, and a
        // `u8` needs no alignment. The reference lives only for the load.
        let byte = unsafe { AtomicU8::from_ptr(data.add(index).cast_mut()) };
        bytes.push(byte.load(Ordering::Relaxed));
    }
    bytes
}

/// The message for Markdown bytes above `MAX_BYTES`.
fn too_long(length: usize) -> String {
    format!("markdown must be at most {MAX_BYTES} bytes long, received {length} bytes")
}

/// Whether an N-API predicate call answers `true`; a failed call counts as
/// `false`. `query` receives the out pointer for the answer.
fn holds(query: impl FnOnce(*mut bool) -> sys::napi_status) -> bool {
    let mut result = false;
    query(&raw mut result) == sys::Status::napi_ok && result
}

/// The kind of a rejected Markdown value, for its error message.
///
/// It only asks N-API about the value's type, so no getter, proxy trap or
/// `toJSON` runs. A `SharedArrayBuffer` reads as an object, because N-API on
/// Node.js 22 cannot tell it from one.
///
/// # Safety
///
/// `env` and `value` must be the valid handles of the current N-API call.
unsafe fn describe(env: sys::napi_env, value: sys::napi_value) -> &'static str {
    /// Indexed by `napi_valuetype`, whose values are part of the N-API ABI.
    const VALUE_TYPES: [&str; 10] = [
        "undefined",
        "null",
        "a boolean",
        "a number",
        "a string",
        "a symbol",
        "an object",
        "a function",
        "an external value",
        "a bigint",
    ];

    let mut value_type = -1;
    // SAFETY: the caller's handles.
    if unsafe { sys::napi_typeof(env, value, &raw mut value_type) } != sys::Status::napi_ok {
        return "an unsupported value";
    }
    if value_type == sys::ValueType::napi_object {
        // SAFETY: the caller's handles.
        return unsafe { describe_object(env, value) };
    }
    name(&VALUE_TYPES, value_type).unwrap_or("an unsupported value")
}

/// `describe` for an object.
///
/// # Safety
///
/// `env` and `value` must be the valid handles of the current N-API call.
unsafe fn describe_object(env: sys::napi_env, value: sys::napi_value) -> &'static str {
    /// Indexed by `napi_typedarray_type`, whose values are part of the N-API
    /// ABI.
    const TYPED_ARRAYS: [&str; 11] = [
        "an Int8Array",
        "a Uint8Array",
        "a Uint8ClampedArray",
        "an Int16Array",
        "a Uint16Array",
        "an Int32Array",
        "a Uint32Array",
        "a Float32Array",
        "a Float64Array",
        "a BigInt64Array",
        "a BigUint64Array",
    ];

    // SAFETY (every block below): the caller's handles, and N-API only writes
    // the out pointers it is given.
    if holds(|result| unsafe { sys::napi_is_typedarray(env, value, result) }) {
        let mut kind = -1;
        let status = unsafe {
            sys::napi_get_typedarray_info(
                env,
                value,
                &raw mut kind,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        return (status == sys::Status::napi_ok)
            .then(|| name(&TYPED_ARRAYS, kind))
            .flatten()
            .unwrap_or("a typed array");
    }
    if holds(|result| unsafe { sys::napi_is_dataview(env, value, result) }) {
        "a DataView"
    } else if holds(|result| unsafe { sys::napi_is_arraybuffer(env, value, result) }) {
        "an ArrayBuffer"
    } else if holds(|result| unsafe { sys::napi_is_array(env, value, result) }) {
        "an array"
    } else {
        "an object"
    }
}

/// The entry of `names` for an N-API enum value, if it has one.
fn name(names: &[&'static str], value: i32) -> Option<&'static str> {
    names.get(usize::try_from(value).ok()?).copied()
}

/// The JavaScript error class `throw` raises.
#[derive(Clone, Copy)]
enum Throw {
    TypeError,
    RangeError,
}

/// Throws a `TypeError` (code `ERR_INVALID_ARG_TYPE`) or a `RangeError` (code
/// `ERR_OUT_OF_RANGE`), and returns the error that makes napi-rs leave the
/// pending exception in place.
///
/// If throwing fails, the returned error makes napi-rs throw a plain `Error`
/// with the same message instead.
///
/// # Safety
///
/// `env` must be the valid handle of the current N-API call.
unsafe fn throw(env: sys::napi_env, kind: Throw, message: &str) -> Error {
    let Ok(text) = CString::new(message) else {
        return Error::new(Status::InvalidArg, message.to_owned());
    };
    // SAFETY: `env` is the handle of the current call, and both strings are
    // NUL-terminated and live until N-API returns.
    let status = unsafe {
        match kind {
            Throw::TypeError => {
                sys::napi_throw_type_error(env, c"ERR_INVALID_ARG_TYPE".as_ptr(), text.as_ptr())
            }
            Throw::RangeError => {
                sys::napi_throw_range_error(env, c"ERR_OUT_OF_RANGE".as_ptr(), text.as_ptr())
            }
        }
    };
    if status == sys::Status::napi_ok {
        Error::new(Status::PendingException, message.to_owned())
    } else {
        Error::new(Status::InvalidArg, message.to_owned())
    }
}
