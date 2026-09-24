//! Single-pass conversion of a JavaScript Markdown argument to UTF-8.
//!
//! napi-rs converts a `String` argument with two `napi_get_value_string_utf8`
//! calls. The first only measures the UTF-8 length, which V8 computes by
//! walking the whole string, and the second transcodes into a zero-filled
//! buffer of exactly that length. [`Utf8Input`] asks for the UTF-16 length
//! instead, which V8 stores, reserves the worst case without filling it, and
//! transcodes once. Both paths use N-API's transcoder, and with enough room it
//! writes the same bytes. The unused part of the reservation is never written,
//! and every export drops the string before it returns.
//!
//! # Why the reservation is always large enough
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

use std::ops::Deref;
use std::ptr;

use napi::bindgen_prelude::{FromNapiValue, Result};
use napi::sys;

/// UTF-8 bytes per UTF-16 code unit, at most. See the module documentation.
const MAX_UTF8_PER_UNIT: usize = 3;

/// UTF-8 bytes per character, at most: a code point above U+FFFF.
const MAX_UTF8_CHAR: usize = 4;

/// The largest buffer the single pass hands N-API. Node.js 22 passes
/// `bufsize - 1` to V8 as an `int`, and a larger size could wrap.
const MAX_CAPACITY: usize = i32::MAX as usize;

/// A Markdown argument, converted to UTF-8 in one pass.
///
/// It holds exactly the bytes napi-rs's `String` conversion produces, and a
/// value that is not a string fails with napi-rs's own error. Exports declare
/// it with `#[napi(ts_arg_type = "string")]`, so their TypeScript declarations
/// stay those of a `String` argument.
pub struct Utf8Input(String);

impl Utf8Input {
    /// The converted string.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Deref for Utf8Input {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

// `FromNapiValue::from_napi_value` is an unsafe trait method, and N-API is a C
// interface. The conversion's safety argument sits next to each call.
#[allow(unsafe_code)]
impl FromNapiValue for Utf8Input {
    unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> Result<Self> {
        let mut units = 0usize;
        // SAFETY: napi-rs passes the `env` and `value` handles of the current
        // call, and both stay valid for it. A null buffer asks only for the
        // length, which N-API writes to `units`.
        let status = unsafe {
            sys::napi_get_value_string_utf16(env, value, ptr::null_mut(), 0, &raw mut units)
        };
        if status != sys::Status::napi_ok {
            // Not a string. napi-rs's conversion fails the same way and builds
            // its usual message.
            // SAFETY: the same handles, still within the call.
            return unsafe { exact(env, value) };
        }
        if units == 0 {
            return Ok(Self(String::new()));
        }

        // The bound, then the spare bytes that prove completeness and the NUL
        // terminator N-API always writes. No V8 string needs more than
        // `MAX_CAPACITY`; past it, or when the worst case cannot be allocated
        // but the exact size still might be, the exact path converts.
        let bound = units.saturating_mul(MAX_UTF8_PER_UNIT);
        let capacity = bound.saturating_add(MAX_UTF8_CHAR + 1);
        let mut bytes = Vec::<u8>::new();
        if capacity > MAX_CAPACITY || bytes.try_reserve_exact(capacity).is_err() {
            // SAFETY: the same handles, still within the call.
            return unsafe { exact(env, value) };
        }

        let mut written = 0usize;
        // SAFETY: `bytes` owns at least `capacity` writable bytes, the size
        // passed as `bufsize`. N-API writes at most `bufsize - 1` UTF-8 bytes,
        // then a NUL terminator, and reports the UTF-8 byte count in `written`.
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

        // SAFETY: N-API initialized the first `written` bytes, and `written`
        // is at most `bound`, below the reserved capacity. At least
        // `MAX_UTF8_CHAR` bytes of room stayed unused, so N-API converted the
        // whole string rather than stopping at a character that did not fit.
        // A whole conversion is valid UTF-8, because N-API replaces lone
        // surrogates with U+FFFD.
        unsafe {
            bytes.set_len(written);
            Ok(Self(String::from_utf8_unchecked(bytes)))
        }
    }
}

/// napi-rs's own `String` conversion: a UTF-8 length pass, then the copy.
///
/// It handles every case the single pass leaves out, and it produces napi-rs's
/// error for a value that is not a string.
///
/// # Safety
///
/// `env` and `value` must be the valid handles of the current N-API call.
#[allow(unsafe_code)]
#[cold]
#[inline(never)]
unsafe fn exact(env: sys::napi_env, value: sys::napi_value) -> Result<Utf8Input> {
    // SAFETY: the caller passes the handles of the current call.
    unsafe { String::from_napi_value(env, value) }.map(Utf8Input)
}
