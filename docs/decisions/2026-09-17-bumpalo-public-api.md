# bumpalo is part of the public API

## Scope

The API-freeze review of 2026-09-17 asked which dependencies a caller can see
through `ferromark`'s public surface, because from 2.0.0 that answer decides
when a dependency upgrade forces a major release. This record documents the
answer and the decision to accept it. No code changes with it; the
`docs/rust-api.md` paragraph and the `allocator` module documentation that state
it do.

## What leaks

bumpalo is not an implementation detail of the `allocator` module. It is in the
signatures:

| Item | Exposure |
| --- | --- |
| `allocator::Bump` | `pub use bumpalo::Bump` |
| `Allocator::bump()` | Returns `&Bump` |
| `impl Deref for Allocator` | `Target = Bump` |
| `allocator::Box::new_in` | Takes `&'a Bump` |
| `allocator::Vec::new_in` | Takes `&'a Bump` |
| `impl Deref for allocator::Vec` | `Target = bumpalo::collections::Vec` |
| `IntoIterator for allocator::Vec` | `IntoIter = bumpalo::collections::vec::IntoIter` |
| `allocator::String` | `pub type String<'a> = bumpalo::collections::String<'a>` |

A caller who builds an AST node, walks an arena vector, or hands an arena to a
parser names a bumpalo type, directly or through a deref. So a breaking bumpalo
release is a breaking ferromark release: **bumpalo 4.x requires ferromark
3.0.0.** A bumpalo minor or patch release is a ferromark minor or patch release,
as usual.

The other dependencies do not leak. `compact_str`, `smallvec`, `serde_json` and
`thiserror` appear only in private fields, internal collections and derived
trait impls; `thiserror` generates `Display` and `Error` impls that are part of
the standard library's vocabulary, not `thiserror`'s.

## Decision

Accept the coupling.

Encapsulating bumpalo means wrapping `Bump`, both collection types and the
string type in newtypes that reimplement the parts of their APIs the parser and
every AST consumer use, and dropping the `Deref` impls that make an `Allocator`
usable as a `Bump` today. That is a refactor of the crate's most performance
sensitive layer, it removes capability from callers who legitimately want the
underlying arena, and it buys one thing: the freedom to absorb a bumpalo major
release in a ferromark minor one. bumpalo 3.x has been stable for years, the
arena API it exposes is small and settled, and the price of being wrong is a
major version bump that is cheap to perform and easy to explain.

The coupling is therefore documented rather than removed, so that a caller
pinning `ferromark` knows what a bumpalo upgrade implies, and so that a future
maintainer reading a bumpalo 4.0 release note knows this is a major release
here.

## The arena and the AST are single-threaded

`allocator::Box` holds a `std::ptr::NonNull<T>`, which is `!Send` and `!Sync`,
so `Document` and `Node` are neither; `Bump` holds interior cells, so
`Allocator` is `!Sync`. This is the expected shape for an arena and is not a defect,
but it was undocumented, and a caller who discovers it from a compiler error
while moving a parsed document onto a worker thread has already written the
wrong code.

The supported pattern is one arena per thread: parse and render on the thread
that owns the allocator and move the rendered `String`, which owns nothing in
the arena, wherever it is needed. `docs/rust-api.md` and the `allocator` module
documentation both state this.
