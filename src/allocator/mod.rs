//! Arena allocator for Ferromark.
//!
//! This crate provides a high-performance arena allocator based on bumpalo,
//! designed for efficient memory management during parsing operations.
//!
//! # bumpalo is part of the public API
//!
//! [`Allocator`] re-exports and dereferences to [`bumpalo::Bump`], and [`Box`],
//! [`Vec`] and [`String`] are bumpalo collections or thin wrappers around them.
//! A bumpalo major release is therefore a ferromark major release; see the
//! decision record `docs/decisions/2026-09-17-bumpalo-public-api.md`.
//!
//! # The arena and the AST are single-threaded
//!
//! [`Allocator`] holds a `Bump`, whose interior cells make it `!Sync`, and
//! [`Box`] holds a raw [`NonNull`](std::ptr::NonNull), which makes
//! [`crate::ast::Document`] and [`crate::ast::Node`] both `!Send` and `!Sync`.
//! An arena and the AST that lives in it stay on the thread that created them.
//! Parse and render per thread and move the rendered `String` — which owns
//! nothing in the arena — across threads instead.

#![deny(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::ops::Deref;

pub use bumpalo::Bump;

/// Arena allocator wrapper for Ferromark.
///
/// This type wraps bumpalo's `Bump` allocator to provide fast, arena-based
/// allocation for AST nodes and other parsing-related data structures.
#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

impl Allocator {
    /// Creates a new allocator with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Creates a new allocator with the specified capacity in bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
        }
    }

    /// Creates a new allocator pre-sized for parsing a Markdown source of
    /// the given length. The capacity is a heuristic (`source_len * 8`
    /// bytes, with a 2 KB floor) that covers the typical AST footprint
    /// for real-world Markdown without growing through bumpalo's
    /// chunk-doubling path — on a fresh [`Self::new`], that path accounts
    /// for ~10 global allocations on a 64 KB document.
    ///
    /// Callers that already know the input length should prefer this over
    /// [`Self::new`]: same fallible-only allocator API, but typically one
    /// arena chunk for the whole parse + render pipeline.
    #[must_use]
    pub fn for_source_len(source_len: usize) -> Self {
        Self::with_capacity(Self::capacity_for_source_len(source_len))
    }

    /// Returns the arena capacity [`Self::for_source_len`] would pick for a
    /// source of `source_len` bytes.
    ///
    /// Exposed for callers that reuse one arena across documents and need to
    /// decide whether the retained chunk is still big enough for the next one.
    #[must_use]
    pub const fn capacity_for_source_len(source_len: usize) -> usize {
        // The 8× factor is empirical: across the bundled corpora
        // (rust-book / vite / vue / typescript-handbook) the AST + render
        // output combined comes in between 5× and 7× of the source
        // length. 8× errs slightly on the over-allocation side so the
        // first chunk almost always suffices.
        const BYTES_PER_INPUT_BYTE: usize = 8;
        // Small documents break the ratio: below ~256 bytes the parse's fixed
        // and per-node arena costs, not the source, decide what the arena
        // holds, so some floor is unavoidable. It only has to cover that fixed
        // part, and the fixed part is small: measured with
        // `iter_allocated_chunks_raw` after parse and render, an empty
        // document occupies 128 bytes, the authored comment shapes of the
        // broad corpus 208–2,688, and the densest nested-emphasis CommonMark
        // example under 128 bytes 2,368.
        //
        // 2 KB covers that with room to spare. bumpalo rounds a sub-page
        // request up to the next power of two minus its chunk overhead, so
        // 2 KB becomes one chunk of 4,032 usable bytes, and none of 783
        // measured documents — every CommonMark and GFM specification
        // example, the repository's own Markdown, the broad corpus and its
        // authored comments — needed a second chunk under any profile. A 16 KB
        // request rounds up to a page multiple instead, so a 37-byte comment
        // that occupies 208 bytes reserved 20,416 bytes on every fresh parse;
        // an earlier 4 KB floor crossed the same page boundary and still
        // reserved 8,128. The floor binds only up to 256 bytes of source;
        // above that the 8× term is already larger and decides alone.
        const MIN_CAPACITY: usize = 2 * 1024;
        let capacity = source_len.saturating_mul(BYTES_PER_INPUT_BYTE);
        if capacity < MIN_CAPACITY {
            MIN_CAPACITY
        } else {
            capacity
        }
    }

    /// Returns the underlying bump allocator.
    #[must_use]
    pub fn bump(&self) -> &Bump {
        &self.bump
    }

    /// Allocates a value in the arena and returns a no-drop box to it.
    ///
    /// The box is a thin pointer: it does not run `T`'s destructor, for the
    /// same reason [`Vec`] does not. `T` must own nothing outside the arena.
    pub fn boxed<T>(&self, val: T) -> Box<'_, T> {
        Box::new_in(val, &self.bump)
    }

    /// Allocates a value in the arena and returns a reference to it.
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.bump.alloc(val)
    }

    /// Allocates a string in the arena.
    pub fn alloc_str(&self, s: &str) -> &str {
        self.bump.alloc_str(s)
    }

    /// Creates a new `Vec` in the arena.
    pub fn new_vec<T>(&self) -> Vec<'_, T> {
        Vec::new_in(&self.bump)
    }

    /// Creates a new `Vec` in the arena with the given capacity.
    pub fn new_vec_with_capacity<T>(&self, capacity: usize) -> Vec<'_, T> {
        Vec::with_capacity_in(capacity, &self.bump)
    }

    /// Creates a new `String` in the arena.
    pub fn new_string(&self) -> String<'_> {
        String::new_in(&self.bump)
    }

    /// Creates a new `String` in the arena from a `&str`.
    pub fn new_string_from(&self, s: &str) -> String<'_> {
        String::from_str_in(s, &self.bump)
    }

    /// Resets the allocator, freeing all allocated memory.
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Returns the total bytes allocated in this arena.
    #[must_use]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }
}

impl Deref for Allocator {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.bump
    }
}

/// A boxed value allocated in an arena.
///
/// Unlike [`bumpalo::boxed::Box`], this does **not** run `T`'s destructor.
/// The `Bump` reclaims the slot with everything else; a `Drop` impl here
/// would make any AST node that stored one need dropping, which is the
/// tree-walk [`Vec`] exists to avoid. `T` must own nothing outside the arena.
#[repr(transparent)]
pub struct Box<'a, T> {
    ptr: std::ptr::NonNull<T>,
    _lt: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> Box<'a, T> {
    /// Allocates `value` in `bump` and returns a pointer to it.
    pub fn new_in(value: T, bump: &'a Bump) -> Self {
        Self {
            ptr: std::ptr::NonNull::from(bump.alloc(value)),
            _lt: std::marker::PhantomData,
        }
    }
}

impl<T> std::ops::Deref for Box<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: `ptr` comes from `Bump::alloc` and lives for `'a`. The box
        // never deallocates or runs `T`'s destructor; the arena owns the slot.
        #[allow(unsafe_code)]
        unsafe {
            self.ptr.as_ref()
        }
    }
}

impl<T> std::ops::DerefMut for Box<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: same provenance as [`Deref::deref`], and `&mut self` is the
        // only live handle to this slot.
        #[allow(unsafe_code)]
        unsafe {
            self.ptr.as_mut()
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Box<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

impl<T: PartialEq> PartialEq for Box<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

/// A vector allocated in an arena.
///
/// Unlike [`bumpalo::collections::Vec`], this deliberately does **not** run its
/// elements' destructors. Everything it holds lives in the same arena, so the
/// `Bump` reclaims all of it at once; dropping the root of a parsed AST
/// normally would still walk every node in the tree to run empty drop glue,
/// which measured ~4% of a parse-and-render over the bundled corpora.
///
/// **Every element type must own nothing outside the arena.** Nothing here can
/// enforce that generically — a `needs_drop::<T>()` assertion in a generic
/// constructor is never forced — so each module that stores a type in one of
/// these asserts it concretely instead. See the `ast` module's
/// `AST_IS_ARENA_ONLY` for the AST's.
#[repr(transparent)]
pub struct Vec<'a, T>(std::mem::ManuallyDrop<bumpalo::collections::Vec<'a, T>>);

impl<'a, T> Vec<'a, T> {
    /// Constructs a new, empty vector in `bump`.
    pub fn new_in(bump: &'a Bump) -> Self {
        Self(std::mem::ManuallyDrop::new(
            bumpalo::collections::Vec::new_in(bump),
        ))
    }

    /// Constructs a new, empty vector in `bump` with room for `capacity`
    /// elements.
    pub fn with_capacity_in(capacity: usize, bump: &'a Bump) -> Self {
        Self(std::mem::ManuallyDrop::new(
            bumpalo::collections::Vec::with_capacity_in(capacity, bump),
        ))
    }

    /// Returns the elements as an arena slice, consuming the vector.
    pub fn into_bump_slice(self) -> &'a [T] {
        std::mem::ManuallyDrop::into_inner(self.0).into_bump_slice()
    }
}

impl<'a, T> std::ops::Deref for Vec<'a, T> {
    type Target = bumpalo::collections::Vec<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Vec<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Vec<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

impl<T: PartialEq> PartialEq for Vec<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<'a, T> IntoIterator for Vec<'a, T> {
    type Item = T;
    type IntoIter = bumpalo::collections::vec::IntoIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        std::mem::ManuallyDrop::into_inner(self.0).into_iter()
    }
}

impl<'v, T> IntoIterator for &'v Vec<'_, T> {
    type Item = &'v T;
    type IntoIter = std::slice::Iter<'v, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'v, T> IntoIterator for &'v mut Vec<'_, T> {
    type Item = &'v mut T;
    type IntoIter = std::slice::IterMut<'v, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A string allocated in an arena.
pub type String<'a> = bumpalo::collections::String<'a>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator_creation() {
        let allocator = Allocator::new();
        assert_eq!(allocator.allocated_bytes(), 0);
    }

    #[test]
    fn test_alloc_value() {
        let allocator = Allocator::new();
        let value = allocator.alloc(42);
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_alloc_str() {
        let allocator = Allocator::new();
        let s = allocator.alloc_str("hello");
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_arena_vec() {
        let allocator = Allocator::new();
        let mut vec = allocator.new_vec();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_arena_string() {
        let allocator = Allocator::new();
        let mut s = allocator.new_string();
        s.push_str("hello");
        s.push_str(" world");
        assert_eq!(s.as_str(), "hello world");
    }

    #[test]
    fn source_len_capacity_is_proportional_above_a_two_kilobyte_floor() {
        // Pinned so a reservation change is a deliberate edit: the floor holds
        // up to 256 bytes of source and the 8× term takes over above it.
        for (source_len, expected) in [
            (0, 2 * 1024),
            (37, 2 * 1024),
            (255, 2 * 1024),
            (256, 2 * 1024),
            (257, 257 * 8),
            (512, 512 * 8),
            (100 * 1024, 100 * 1024 * 8),
        ] {
            assert_eq!(
                Allocator::capacity_for_source_len(source_len),
                expected,
                "capacity for a {source_len}-byte source"
            );
            assert!(
                Allocator::for_source_len(source_len).allocated_bytes() >= expected,
                "arena for a {source_len}-byte source reserved less than {expected}"
            );
        }
        // The 8× term saturates rather than wrapping into a tiny reservation.
        assert_eq!(
            Allocator::capacity_for_source_len(usize::MAX),
            usize::MAX,
            "an implausible source length must not wrap"
        );
    }

    #[test]
    fn comment_sized_documents_parse_and_render_in_the_first_chunk() {
        // The floor exists for exactly this: the fixed cost of a parse, which
        // dwarfs the source at these sizes, must not force a second chunk.
        for source in [
            "",
            "# Title\n",
            "Thanks, this fixes the issue for me.\n",
            "[The guide](<https://example.org/guide>)\n",
            // The densest nested-emphasis specification examples, which need
            // more arena than any other document under 128 bytes.
            "*foo __bar *baz bim__ bam*\n",
            "[foo *[bar [baz](/uri)](/uri)*](/uri)\n",
            "Das Verhalten lässt sich auch mit Umlauten reproduzieren: **Änderungen**, *Größe* und `straße.md`.\n",
        ] {
            let arena = Allocator::for_source_len(source.len());
            let reserved = arena.allocated_bytes();
            let document = crate::Parser::new(&arena, source)
                .parse()
                .expect("the sample should parse");
            let html = crate::HtmlRenderer::new().render(&document);
            assert_eq!(
                arena.allocated_bytes(),
                reserved,
                "{source:?} rendered to {} bytes and grew the arena past its first chunk",
                html.len()
            );
        }
    }

    #[test]
    fn boxed_value_is_derefable_and_does_not_need_drop() {
        let allocator = Allocator::new();
        let boxed = allocator.boxed(7u32);
        assert_eq!(*boxed, 7);
        assert!(!std::mem::needs_drop::<Box<'static, u32>>());
    }
}
