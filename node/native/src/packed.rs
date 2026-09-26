//! Private entry points that take `Options` in packed form.
//!
//! napi-rs converts an `Options` argument field by field. For each of the 32
//! fields, present or not, it calls `napi_get_named_property`, which creates
//! the property key from a C string and runs an uncached property lookup, and
//! then `napi_typeof`. That costs more per call than rendering a small
//! document. The facade in `node/ferromark/index.mjs` makes the same reads in
//! JavaScript, where V8 caches them, and passes the result to these entries as
//! plain arguments:
//!
//! - `set` and `on` hold one bit each for `renderPolicy` and the 26 boolean
//!   fields, numbered in their declaration order in [`Options`]. A bit in `set`
//!   marks the field as present, and the same bit in `on` holds its value. For
//!   `renderPolicy`, a set value bit means `'trusted'`.
//! - `headingOffset`, `headingIdPrefix`, `linkBasePath`, `typography`, and
//!   `passes` pass through as read, and `undefined` stands for an absent field.
//!
//! [`unpack`] rebuilds the `Options` value napi-rs would have produced from
//! the object. Each entry then runs the same code as its public counterpart,
//! so every later value check, error message and output is shared. A value
//! napi-rs would reject never reaches these entries: the facade hands it to
//! the public entry, which throws napi-rs's own error for it.
//!
//! `Renderer.withPackedOptions`, the packed counterpart of the constructor,
//! sits with the other `Renderer` methods in `lib.rs`: napi-rs expands a
//! `#[napi] impl` block only after the struct it belongs to.
//!
//! The entries are private to the facade. napi-rs declares them in the
//! generated `native.d.ts`, but the package types in `index.d.mts` do not.

use napi::bindgen_prelude::{Buffer, FnArgs, Function, Result};
use napi::{Env, JsString};
use napi_derive::napi;

use crate::input::Utf8Input;
use crate::{
    NativePassConfig, Options, TransformResult, TypographyConfig, core_options, html_buffer,
    js_string, render_document, render_one_shot,
};

/// Rebuilds the `Options` napi-rs reads from the object the facade packed.
///
/// The bit numbers follow the declaration order of [`Options`], skipping the
/// three fields that pass through unpacked.
pub fn unpack(
    set: u32,
    on: u32,
    heading_offset: Option<f64>,
    heading_id_prefix: Option<String>,
    link_base_path: Option<String>,
    typography: Option<TypographyConfig>,
    passes: Option<Vec<NativePassConfig>>,
) -> Options {
    let flag = |bit: u32| (set & (1 << bit) != 0).then_some(on & (1 << bit) != 0);
    Options {
        render_policy: flag(0)
            .map(|trusted| String::from(if trusted { "trusted" } else { "untrusted" })),
        allow_html: flag(1),
        tables: flag(2),
        merged_table_cells: flag(3),
        table_colgroup: flag(4),
        table_column_names: flag(5),
        table_attributes: flag(6),
        strikethrough: flag(7),
        superscript: flag(8),
        subscript: flag(9),
        task_lists: flag(10),
        autolink_literals: flag(11),
        disallowed_raw_html: flag(12),
        footnotes: flag(13),
        highlight: flag(14),
        inline_footnotes: flag(15),
        allow_link_refs: flag(16),
        front_matter: flag(17),
        heading_ids: flag(18),
        heading_offset,
        heading_id_prefix,
        heading_attributes: flag(19),
        math: flag(20),
        callouts: flag(21),
        definition_lists: flag(22),
        line_comments: flag(23),
        wiki_links: flag(24),
        cjk_emphasis: flag(25),
        mdx: flag(26),
        link_base_path,
        typography,
        passes,
    }
}

/// [`unpack`], but `None` when no field is present, as napi-rs reads
/// `undefined` options.
///
/// The one-shot entries render without options on the thread's kept renderer
/// (see `default_renderer.rs`). Packed options with no field present are the
/// defaults as well: every bit of `set` is clear, so [`unpack`] ignores `on`
/// and leaves every field unset, and `core_options` changes nothing for
/// unset fields.
fn unpack_present(
    set: u32,
    on: u32,
    heading_offset: Option<f64>,
    heading_id_prefix: Option<String>,
    link_base_path: Option<String>,
    typography: Option<TypographyConfig>,
    passes: Option<Vec<NativePassConfig>>,
) -> Option<Options> {
    let absent = set == 0
        && heading_offset.is_none()
        && heading_id_prefix.is_none()
        && link_base_path.is_none()
        && typography.is_none()
        && passes.is_none();
    (!absent).then(|| {
        unpack(
            set,
            on,
            heading_offset,
            heading_id_prefix,
            link_base_path,
            typography,
            passes,
        )
    })
}

/// Internal to the `ferromark` facade: `toHtml` with packed options.
#[napi(catch_unwind, js_name = "toHtmlPacked")]
#[allow(clippy::too_many_arguments)]
pub fn to_html_packed<'env>(
    env: &'env Env,
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    set: u32,
    on: u32,
    heading_offset: Option<f64>,
    heading_id_prefix: Option<String>,
    link_base_path: Option<String>,
    typography: Option<TypographyConfig>,
    passes: Option<Vec<NativePassConfig>>,
) -> Result<JsString<'env>> {
    let options = unpack_present(
        set,
        on,
        heading_offset,
        heading_id_prefix,
        link_base_path,
        typography,
        passes,
    );
    render_one_shot(&markdown, options, |html| js_string(env, html))
}

/// Internal to the `ferromark` facade: `toHtmlBuffer` with packed options.
#[napi(catch_unwind, js_name = "toHtmlBufferPacked")]
#[allow(clippy::too_many_arguments)]
pub fn to_html_buffer_packed(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    set: u32,
    on: u32,
    heading_offset: Option<f64>,
    heading_id_prefix: Option<String>,
    link_base_path: Option<String>,
    typography: Option<TypographyConfig>,
    passes: Option<Vec<NativePassConfig>>,
) -> Result<Buffer> {
    let options = unpack_present(
        set,
        on,
        heading_offset,
        heading_id_prefix,
        link_base_path,
        typography,
        passes,
    );
    render_one_shot(&markdown, options, html_buffer)
}

/// Internal to the `ferromark` facade: `transform` with packed options.
#[napi(catch_unwind, js_name = "transformPacked")]
#[allow(clippy::too_many_arguments)]
pub fn transform_packed(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    set: u32,
    on: u32,
    heading_offset: Option<f64>,
    heading_id_prefix: Option<String>,
    link_base_path: Option<String>,
    typography: Option<TypographyConfig>,
    passes: Option<Vec<NativePassConfig>>,
) -> Result<TransformResult> {
    let options = unpack(
        set,
        on,
        heading_offset,
        heading_id_prefix,
        link_base_path,
        typography,
        passes,
    );
    render_document(&markdown, core_options(Some(options))?, None)
}

/// Internal to the `ferromark` facade: `toHtmlWithRenderer` with packed
/// options.
#[napi(catch_unwind, js_name = "toHtmlWithRendererPacked")]
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn to_html_with_renderer_packed(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    set: u32,
    on: u32,
    heading_offset: Option<f64>,
    heading_id_prefix: Option<String>,
    link_base_path: Option<String>,
    typography: Option<TypographyConfig>,
    passes: Option<Vec<NativePassConfig>>,
    renderer: Function<FnArgs<(String, Option<String>, Option<String>)>, Option<String>>,
) -> Result<String> {
    let options = unpack(
        set,
        on,
        heading_offset,
        heading_id_prefix,
        link_base_path,
        typography,
        passes,
    );
    Ok(render_document(&markdown, core_options(Some(options))?, Some(renderer))?.html)
}

/// Internal to the `ferromark` facade: `transformWithRenderer` with packed
/// options.
#[napi(catch_unwind, js_name = "transformWithRendererPacked")]
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn transform_with_renderer_packed(
    #[napi(ts_arg_type = "string | Uint8Array")] markdown: Utf8Input,
    set: u32,
    on: u32,
    heading_offset: Option<f64>,
    heading_id_prefix: Option<String>,
    link_base_path: Option<String>,
    typography: Option<TypographyConfig>,
    passes: Option<Vec<NativePassConfig>>,
    renderer: Function<FnArgs<(String, Option<String>, Option<String>)>, Option<String>>,
) -> Result<TransformResult> {
    let options = unpack(
        set,
        on,
        heading_offset,
        heading_id_prefix,
        link_base_path,
        typography,
        passes,
    );
    render_document(&markdown, core_options(Some(options))?, Some(renderer))
}
