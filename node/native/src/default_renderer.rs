//! The renderer each thread keeps for one-shot calls without options.
//!
//! `toHtml` and `toHtmlBuffer` used to build a `Renderer` per call and drop
//! it when the call returned. Each call paid for `Renderer::new`, which
//! resolves the options and moves a renderer of about 1 KB into place, and
//! then for allocating and freeing what a kept renderer reuses: the arena's
//! chunks, the output buffer, and the heading ID storage.
//!
//! A call without options (`undefined` or `null`, or packed options with no
//! field present, see `packed.rs`) now renders with a renderer its thread
//! keeps, through `Renderer::render_reused`, exactly as a long-lived
//! `Renderer` object renders every document after its first: the arena is
//! reset, and the HTML renderer clears its output, heading IDs and footnote
//! state before it writes. Link reference definitions and every other bit
//! of parse state live in the `Parser`, which each call builds anew. A call
//! with options still builds a renderer of its own.
//!
//! # Checkout
//!
//! A call takes the renderer out of its slot and puts it back only once the
//! call has succeeded:
//!
//! - A call that finds the slot empty builds a renderer of its own. That is
//!   the first call on a thread, a call made while another call on the same
//!   thread holds the renderer, and the first call after a failed one. No
//!   call waits for the renderer or shares it, so a reentrant call cannot
//!   observe a document in progress. (Today no JavaScript runs while the
//!   renderer is out: code highlighters run inside the entries that render
//!   with `render_document`, which keep no renderer.)
//! - An error drops the renderer, and so does a panic, while it unwinds. A
//!   renderer that stopped in the middle of a document is never used again,
//!   and nothing stays borrowed or poisoned.
//!
//! # Retention
//!
//! A kept renderer holds its arena's last chunk, its output buffer, and its
//! heading ID and footnote storage until the thread's next call. Two limits
//! keep that from following the largest document the thread ever rendered:
//!
//! - Markdown longer than `SOURCE_LIMIT` renders with a renderer of its
//!   own, as every call did before, and leaves the kept renderer alone. This
//!   bounds the heading ID and footnote storage, which grows with the
//!   Markdown.
//! - After a call, the arena is reset, which frees every chunk but the last.
//!   If that chunk or the HTML is longer than `RETAINED_LIMIT`, the
//!   renderer is dropped instead of kept. Reference links let short Markdown
//!   expand into long HTML.
//!
//! A thread therefore keeps at most one arena chunk of 1 MiB, an output
//! buffer of about twice that (a `String` at most doubles its content when
//! it grows), and the heading ID and footnote storage for 64 KiB of
//! Markdown. Typical documents keep far less: across the broad benchmark
//! corpus, the kept arena chunk is 448 to 1,984 bytes for documents under
//! 512 bytes and at most 393,152 bytes for those up to 64 KiB.
//!
//! # Threads
//!
//! The slot is thread-local. Every Node.js worker runs on a thread of its
//! own, with its own slot, and the standard library drops a thread's kept
//! renderer when the thread exits.

use std::cell::Cell;

use napi::bindgen_prelude::Result;

use crate::{Renderer, render_fresh};

/// Markdown longer than this, in bytes, renders with a renderer of its own.
pub const SOURCE_LIMIT: usize = 64 * 1024;

/// A renderer whose arena chunk or HTML is longer than this, in bytes, after
/// a call is dropped rather than kept.
pub const RETAINED_LIMIT: usize = 1024 * 1024;

thread_local! {
    /// The thread's kept renderer; empty while a call holds it.
    static KEPT: Cell<Option<Box<Renderer>>> = const { Cell::new(None) };
}

/// Renders `markdown` with the default options and hands the HTML to `emit`
/// while the renderer still holds it.
pub fn render<T>(markdown: &str, emit: impl FnOnce(&str) -> Result<T>) -> Result<T> {
    if markdown.len() > SOURCE_LIMIT {
        return render_fresh(markdown, None, emit);
    }
    // `try_with` fails only while the thread is exiting.
    let mut renderer = match KEPT.try_with(Cell::take) {
        Ok(Some(renderer)) => renderer,
        _ => Box::new(Renderer::new(None)?),
    };
    let html = renderer.render_reused(markdown)?;
    let html_len = html.len();
    let result = emit(html)?;
    renderer.allocator.reset();
    if html_len <= RETAINED_LIMIT && renderer.allocator.allocated_bytes() <= RETAINED_LIMIT {
        // A renderer that a nested call put back meanwhile is dropped.
        let _ = KEPT.try_with(|slot| slot.set(Some(renderer)));
    }
    Ok(result)
}
