//! The parser and renderer configuration the addon runs with.
//!
//! Two crate roots compile this file: the `cdylib` in `lib.rs`, which builds
//! every entry point's configuration from [`addon_defaults`] plus the caller's
//! `Options`, and the profile-guided training driver in `bin/pgo_train.rs`,
//! which includes it with `#[path = "../options.rs"]`. A `cdylib` cannot be
//! used as a library target, so sharing the source file is what keeps the
//! trained configuration from drifting away from the shipped one.

use ferromark::{HtmlRendererOptions, ParserOptions};
use ferromark_transforms::TransformPipeline;

/// A resolved parser and renderer pair, as every addon entry point uses it.
pub struct CoreOptions {
    pub parser: ParserOptions,
    pub html: HtmlRendererOptions,
    pub heading_level_offset: i32,
    pub heading_id_prefix: String,
    pub pipeline: TransformPipeline,
}

/// The configuration the addon uses when the caller passes no options.
///
/// The GFM specification profile without autolink literals, rendered through
/// the Node package's safe output boundary with heading IDs and callouts on.
/// Every field of `Options` overrides one of these.
pub fn addon_defaults() -> CoreOptions {
    // Preserve the Node package's safe output boundary and common defaults.
    let mut parser = ParserOptions::gfm_spec();
    parser.autolinks = false;
    let mut html = HtmlRendererOptions::gfm_spec();
    html.sanitize = true;
    html.heading_ids = true;
    html.callouts = true;
    CoreOptions {
        parser,
        html,
        heading_level_offset: 0,
        heading_id_prefix: String::new(),
        pipeline: TransformPipeline::new(),
    }
}
