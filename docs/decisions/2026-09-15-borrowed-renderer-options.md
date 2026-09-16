# Borrowed renderer option strings

## Scope

`HtmlRendererOptions` owns five `String` fields (`soft_break`, `hard_break`,
`base_url`, `source_path`, `code_annotation_meta_key`) and one `Vec<String>`
(`autolink_patterns`). Every one of their defaults is a compile-time constant,
so `HtmlRendererOptions::new()` performs six heap allocations to reproduce
static data, `Clone` performs six more, and the internal `RendererOptions`
frees them when the renderer drops.

The internal type already worked around this with `Option<String>` fields plus
static defaults, and `HtmlRenderer::new()` avoids the allocations entirely. The
cost is paid only by callers that build a renderer per document from an
`HtmlRendererOptions` value — the ordinary pipeline shape, and the one the
benchmark worker uses. On a 957-byte GFM comment, the fresh-pipeline profile
attributes 8.8% of parse+render to `HtmlRendererOptions::clone` and 5.1% to the
`RendererOptions` drop.

This changes public field types, so it belongs in `2.0.0-rc` rather than after
the stable v2 release. Nothing here changes rendered HTML.

## Decision

Use `Cow<'static, str>` for the five string fields and
`Cow<'static, [Cow<'static, str>]>` for `autolink_patterns`. Defaults become
`Cow::Borrowed` over existing `const` data, so `HtmlRendererOptions::new()`,
`Default::default()`, `commonmark()`, `gfm()`, and `Clone` of any of them touch
the allocator zero times. Owned runtime values keep working through `.into()`,
which yields `Cow::Owned` for a `String` and `Cow::Borrowed` for a `&'static str`.

`autolink_patterns` must be the whole-list `Cow`, not `Vec<Cow<'static, str>>`.
A `Vec` cannot borrow: the default two-element list would still cost one
allocation in `new()` and one in every `Clone`, which defeats the point of the
change and would make the allocation-free guarantee untestable. The whole-list
`Cow` also stays ergonomic for the two realistic caller shapes — a static list
is `Cow::Borrowed(&[…])` and a runtime list is `vec![…].into()` — and
`Vec<Cow<'static, str>>` converts into it directly.

Simplify `RendererOptions` to hold the same `Cow`s. The `Option<String>` plus
static-default indirection existed only to keep `HtmlRenderer::new()` free of
allocations; borrowed defaults give that for free, so the `Option` wrappers, the
`unwrap_or` accessors and the duplicated internal default table are removed.
`HtmlRenderer::new()` now goes through `HtmlRendererOptions::new()`, leaving one
source of truth for the documented defaults.

Drop the internal `AutolinkPatterns` enum with it. It existed only to bridge
`&[&'static str]` defaults and `&[String]` custom lists; both are now
`&[Cow<'static, str>]`. Its semantics are unchanged and remain load-bearing: a
non-empty list autolinks, and an explicitly empty list disables autolinking just
as an empty `Vec` did.

Empty values stay meaningful. `source_path` defaults to `""`, an empty
`base_url` is not the same as `"/"`, and an empty pattern list is not the same
as the default list. Because the public defaults are the only defaults now, each
of those values survives the conversion into `RendererOptions` verbatim, exactly
as the previous `Some(…)` wrapping guaranteed.

## What breaks for callers

Public field types change, so any direct assignment or struct literal that
supplied a `String` or a `Vec<String>` stops compiling.

| Before | After |
| --- | --- |
| `base_url: "/docs/".to_string()` | `base_url: "/docs/".into()` |
| `options.base_url = path_from_config` | `options.base_url = path_from_config.into()` |
| `autolink_patterns: vec!["mailto:".to_string()]` | `autolink_patterns: vec!["mailto:".into()].into()` |
| `options.autolink_patterns.push(scheme)` | build a `Vec<Cow<'static, str>>`, then `options.autolink_patterns = list.into()` |

Reads are unaffected: `Cow<'static, str>` derefs to `str`, so `is_empty()`,
`trim_end_matches`, comparisons and `push_str` calls keep working. Call sites
that already wrote `.into()` — including
`benchmarks/optimization-rounds/worker.rs` with `hard_break: "<br />\n".into()`
— compile unchanged and now produce `Cow::Borrowed`.

The migration is mechanical: add `.into()` at the assignment, and for pattern
lists add `.into()` per element plus one for the list. Callers that need a
non-`'static` string must own it first (`value.to_string().into()`); the
`'static` bound is deliberate, because options are moved into a renderer that
outlives any borrowed configuration source.

## Alternatives considered

**Keep `String` and add a builder that yields `Arc<RendererOptions>`.** This
removes the per-document clone only for callers that restructure around a shared
handle, leaves the six default allocations in place for everyone else, and adds
atomic reference counting to a type that is read on the render hot path. It also
introduces a second way to configure the renderer while the direct struct
literal — the documented one — keeps its cost.

**Reference-counted options (`Rc`/`Arc` inside the options type).** Cloning gets
cheap, but constructing defaults still allocates, the renderer gains an
indirection on every `hard_break()`/`base_url()` read, and thread-safety becomes
a visible part of the public type. `Cow` reaches zero allocations for the
default case without either cost.

**Generic `Into<Cow<'static, str>>` setters over private fields.** This would
hide the type change, but the options type is deliberately a plain struct
literal with `..Default::default()`, which setters would replace with a builder.
The field types are the documented interface; changing them once during `rc` is
cheaper than changing the shape of the API.

## Verification

A dedicated integration test binary installs a counting `#[global_allocator]`
and asserts zero allocations for `HtmlRendererOptions::new()`, `default()`,
`commonmark()`, `gfm()` and their clones, and that
`HtmlRenderer::with_options(HtmlRendererOptions::new())` allocates exactly as
often as `HtmlRenderer::new()` — the renderer's own scratch buffers, with no
option contribution. Owned values are checked to stay owned and to survive a
clone.

HTML output must stay byte-identical for every option combination. Renderer
tests compare defaults, custom breaks, base URLs, source paths, annotation meta
keys, custom and empty autolink pattern lists, and the `commonmark`/`gfm`
profiles against their existing expectations, and the specification corpora and
snapshots are unchanged. No performance claim is recorded here; this decision
records the allocation counts, which are exact, not timings.
