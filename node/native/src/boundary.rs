//! N-API boundary diagnostics, compiled only with the `boundary-bench` feature.
//!
//! `node/ferromark/bench/boundary.mjs` times these exports next to the public
//! ones to split a `toHtml` call into input conversion, the Rust core, output
//! conversion and fixed per-call costs. Like `panic-test`, the feature is never
//! part of a published addon: `build-native.mjs` only builds it into an
//! isolated output directory, and `verify-package.mjs` rejects a package
//! declaration file that mentions it.
//!
//! Every export does one isolated piece of a real call, and the core loops call
//! the same helpers as the public exports (`render_one_shot`,
//! `Renderer::render_reused`), so the parts add up to the real thing. Markdown
//! arguments convert through [`Utf8Input`], as the public exports' do. The
//! `candidate` exports prototype reductions; they are measurements, not API.
//! The `NapiString` exports keep napi-rs's own `String` conversion, which the
//! public exports used before `Utf8Input`, as a reference.

use std::cell::{Cell, RefCell};
use std::hint::black_box;

use napi::bindgen_prelude::{Buffer, BufferSlice, Error, Result, Status, Uint8ArraySlice};
use napi::{Env, JsString, JsStringLatin1};
use napi_derive::napi;

use crate::input::Utf8Input;
use crate::packed::unpack;
use crate::{Options, Renderer, core_options, render_one_shot};

/// Returns nothing: the floor of a free-function N-API call.
#[napi(catch_unwind, js_name = "boundaryNoop")]
pub fn noop() {}

/// Input conversion only, as the public exports convert Markdown: one
/// `napi_get_value_string_utf8` pass into a reserved buffer. The string is
/// dropped again.
#[napi(catch_unwind, js_name = "boundaryLen")]
pub fn len(#[napi(ts_arg_type = "string")] markdown: Utf8Input) -> u32 {
    markdown.len() as u32
}

/// Input conversion only, with napi-rs's `String` conversion
/// (`napi_get_value_string_utf8` twice: a UTF-8 length pass, then the copy
/// into a zero-filled buffer), which the public exports used before.
#[napi(catch_unwind, js_name = "boundaryLenNapiString")]
pub fn len_napi_string(markdown: String) -> u32 {
    markdown.len() as u32
}

/// The UTF-8 bytes the public exports convert a string to.
#[napi(catch_unwind, js_name = "boundaryInputBytes")]
pub fn input_bytes(#[napi(ts_arg_type = "string")] markdown: Utf8Input) -> Buffer {
    markdown.into_string().into_bytes().into()
}

/// The UTF-8 bytes napi-rs's `String` conversion produces, to compare with
/// `boundaryInputBytes`.
#[napi(catch_unwind, js_name = "boundaryNapiStringBytes")]
pub fn napi_string_bytes(markdown: String) -> Buffer {
    markdown.into_bytes().into()
}

/// Input and output conversion of the same string, without the core.
#[napi(catch_unwind, js_name = "boundaryEcho")]
pub fn echo(#[napi(ts_arg_type = "string")] markdown: Utf8Input) -> String {
    markdown.into_string()
}

thread_local! {
    /// A grow-only ASCII buffer, so `boundaryMake` pays for output conversion
    /// only, not for building the string it converts.
    static ASCII: Cell<&'static str> = const { Cell::new("") };
    /// The `reuse` lifecycle's renderer. It outlives a call, as a long-lived
    /// `Renderer` does, so no batch pays for first-use arena or buffer growth.
    static REUSED: RefCell<Option<Renderer>> = const { RefCell::new(None) };
}

/// Output conversion only: an `n`-byte ASCII string, borrowed from a cached
/// buffer and copied into a JavaScript string by `napi_create_string_utf8`.
#[napi(catch_unwind, js_name = "boundaryMake")]
pub fn make(n: u32) -> &'static str {
    let n = n as usize;
    ASCII.with(|cached| {
        if cached.get().len() < n {
            // The superseded buffer stays allocated: a bounded geometric
            // series in a diagnostic build, in exchange for a `'static` borrow.
            cached.set(Box::leak(
                "x".repeat(n.next_power_of_two()).into_boxed_str(),
            ));
        }
        &cached.get()[..n]
    })
}

/// Runs the Rust core `iterations` times on one converted string.
///
/// The per-iteration core time carries no boundary cost. Returns a checksum of
/// the output lengths (wrapping), which the script compares with the real output.
///
/// - `reuse`: `Renderer.toHtml`'s Rust side (arena reset, parse, borrowed
///   render) on a renderer kept across calls.
/// - `fresh`: `toHtml`'s Rust side (`Renderer::new`, parse, owned render, drops).
/// - `setup`: `Renderer::new` with default options and its drop, nothing else.
#[napi(catch_unwind, js_name = "boundaryCoreOnly")]
pub fn core_only(
    #[napi(ts_arg_type = "string")] markdown: Utf8Input,
    iterations: u32,
    lifecycle: String,
) -> Result<u32> {
    let mut checksum = 0u32;
    match lifecycle.as_str() {
        "reuse" => REUSED.with_borrow_mut(|reused| -> Result<()> {
            let renderer = match reused {
                Some(renderer) => renderer,
                None => reused.insert(Renderer::new(None)?),
            };
            for _ in 0..iterations {
                let html = renderer.render_reused(black_box(&markdown))?;
                checksum = checksum.wrapping_add(black_box(html).len() as u32);
            }
            Ok(())
        })?,
        "fresh" => {
            for _ in 0..iterations {
                let html = render_one_shot(black_box(&markdown), None)?;
                checksum = checksum.wrapping_add(black_box(&html).len() as u32);
            }
        }
        "setup" => {
            for _ in 0..iterations {
                drop(black_box(Renderer::new(None)?));
                checksum = checksum.wrapping_add(1);
            }
        }
        _ => {
            return Err(Error::new(
                Status::InvalidArg,
                "lifecycle must be 'reuse', 'fresh' or 'setup'",
            ));
        }
    }
    Ok(checksum)
}

/// Option handling only: napi-rs reads every `Options` field from the object
/// (one `napi_get_named_property` each), then the addon resolves them.
#[napi(catch_unwind, js_name = "boundaryOptions")]
pub fn options(options: Option<Options>) -> Result<u32> {
    let options = core_options(options)?;
    Ok(u32::from(black_box(options).html.sanitize))
}

/// Option handling as the facade passes options now: packed into plain
/// arguments (see `packed.rs`), unpacked and resolved like `boundaryOptions`.
#[napi(catch_unwind, js_name = "boundaryOptionsPacked")]
pub fn options_packed(
    set: u32,
    on: u32,
    heading_offset: Option<f64>,
    heading_id_prefix: Option<String>,
    link_base_path: Option<String>,
) -> Result<u32> {
    let options = unpack(set, on, heading_offset, heading_id_prefix, link_base_path);
    let options = core_options(Some(options))?;
    Ok(u32::from(black_box(options).html.sanitize))
}

/// Candidate input path: UTF-8 bytes borrowed from a `Uint8Array` or `Buffer`
/// (no transcoding, no copy, no allocation), validated as a `&str` would be.
#[napi(catch_unwind, js_name = "boundaryBytesLen")]
pub fn bytes_len(bytes: Uint8ArraySlice) -> Result<u32> {
    let markdown = std::str::from_utf8(&bytes)
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
    Ok(markdown.len() as u32)
}

/// Candidate one-shot output path: an external Latin-1 string for ASCII HTML.
///
/// The owned HTML is handed to V8 without a copy and freed by the garbage
/// collector; non-ASCII HTML is converted like `toHtml`. Everything else is
/// `toHtml`.
#[napi(catch_unwind, js_name = "boundaryToHtmlExternal")]
pub fn to_html_external<'env>(
    env: &'env Env,
    #[napi(ts_arg_type = "string")] markdown: Utf8Input,
) -> Result<JsString<'env>> {
    let html = render_one_shot(&markdown, None)?;
    if !html.is_empty() && html.is_ascii() {
        Ok(JsStringLatin1::from_data(env, html.into_bytes())?.into_value())
    } else {
        env.create_string(&html)
    }
}

/// Holds one document's rendered HTML, so each output path can be timed on the
/// exact HTML the public exports return.
#[napi]
pub struct BoundaryProbe {
    html: String,
}

#[napi]
impl BoundaryProbe {
    /// Renders `markdown` once, as `new Renderer().toHtml(markdown)` does.
    #[napi(constructor, catch_unwind)]
    pub fn new(#[napi(ts_arg_type = "string")] markdown: Utf8Input) -> Result<Self> {
        let mut renderer = Renderer::new(None)?;
        let html = renderer.render_reused(&markdown)?.to_owned();
        Ok(Self { html })
    }

    /// Returns nothing: the floor of a method call on a wrapped object. It
    /// takes `&self` so the call unwraps the object, as `Renderer.toHtml` does.
    #[allow(clippy::unused_self)]
    #[napi(catch_unwind)]
    pub fn noop(&self) {}

    /// Output conversion exactly as `Renderer.toHtml` returns it: a borrowed
    /// `&str` through `napi_create_string_utf8`.
    #[napi(catch_unwind)]
    pub fn html(&self) -> &str {
        &self.html
    }

    /// Output exactly as `toHtmlBuffer` returns it: a `Vec` copy handed to
    /// `napi_create_external_buffer` with a finalizer.
    #[napi(catch_unwind)]
    pub fn html_buffer(&self) -> Buffer {
        self.html.as_bytes().to_vec().into()
    }

    /// Candidate `Buffer` output: one `napi_create_buffer_copy` from the
    /// borrowed HTML, with no Rust-side copy and no finalizer.
    #[napi(catch_unwind)]
    pub fn html_buffer_copy<'env>(&self, env: &'env Env) -> Result<BufferSlice<'env>> {
        BufferSlice::copy_from(env, self.html.as_bytes())
    }

    /// Candidate string output: `napi_create_string_latin1` (a plain copy with
    /// no UTF-8 decoding), valid only for ASCII HTML. The ASCII check runs on
    /// every call, as it would in a real implementation.
    #[napi(catch_unwind)]
    pub fn html_latin1<'env>(&self, env: &'env Env) -> Result<JsString<'env>> {
        if !self.html.is_ascii() {
            return Err(Error::new(
                Status::InvalidArg,
                "htmlLatin1 applies to ASCII HTML only",
            ));
        }
        env.create_string_latin1(self.html.as_bytes())
    }

    #[napi(getter)]
    pub fn html_byte_length(&self) -> u32 {
        self.html.len() as u32
    }

    #[napi(getter)]
    pub fn html_is_ascii(&self) -> bool {
        self.html.is_ascii()
    }
}
