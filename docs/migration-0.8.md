# Migrating to ferromark 0.8

Version 0.8 makes ferromark's public `Options` and event/type enums
forward-compatible before 1.0 and moves parser implementation modules behind
the stable crate-root facade. These are intentional pre-1.0 breaking API
changes: update option construction, exhaustive enum matches, and
module-qualified imports as below.

## Configure non-exhaustive options from a preset

`Options` is now `#[non_exhaustive]`. Rust consequently rejects all external
struct literals, including the former update form:

```rust,ignore
let options = Options {
    heading_ids: false,
    ..Options::default()
};
```

Start from a preset and mutate its public fields instead:

```rust
use ferromark::Options;

let mut options = Options::default();
options.heading_ids = false;
```

For several updates, `ferromark::options!` is a compact equivalent:

```rust
use ferromark::Options;

let options = ferromark::options!(Options::gfm();
    front_matter: true,
    allow_html: false,
);
```

## Add fallback arms to public enum matches

Public parser, rendering, autolink, and MDX enums are also
`#[non_exhaustive]`. Add a wildcard arm so future variants can be handled
without another source-breaking release:

```rust
use ferromark::RenderPolicy;

let label = match RenderPolicy::Untrusted {
    RenderPolicy::Untrusted => "untrusted",
    _ => "future policy",
};
assert_eq!(label, "untrusted");
```

## Import public types and helpers from the crate root

The `block`, `cursor`, `escape`, `footnote`, `inline`, `link_ref`, `range`, and
`render` modules are now private implementation details. Public parsers,
events, stores, ranges, the HTML writer, and the supported escaping helpers are
available directly from `ferromark`.

Replace module-qualified imports:

```rust,ignore
use ferromark::block::{ListKind, TaskState};
use ferromark::inline::AutolinkLiteralKind;
use ferromark::escape::escape_text_into;
```

with crate-root imports:

```rust
use ferromark::{AutolinkLiteralKind, ListKind, TaskState, escape_text_into};

let mut escaped = Vec::new();
escape_text_into(&mut escaped, b"<code>");
assert_eq!(escaped, b"&lt;code&gt;");
```

`Cursor`, mark-resolution internals, and other low-level helpers no longer form
part of the public API. Keep custom integrations on the root-level
`BlockParser`, `InlineParser`, event, store, `Range`, and `HtmlWriter` types.
