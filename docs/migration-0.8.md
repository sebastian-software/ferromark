# Migrating to ferromark 0.8

Version 0.8 makes ferromark's public `Options` and event/type enums
forward-compatible before 1.0. This is an intentional pre-1.0 breaking API
change: update option construction and exhaustive enum matches as below.

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
