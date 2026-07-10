# Migrating to ferromark 0.2

Version 0.2 tightens the trust boundary and removes unchecked UTF-8 and code
generation paths from the public API. Most callers only need to decide whether
their Markdown is trusted and handle a few new `Result` values.

## Untrusted rendering is the default

The standard rendering functions now use `RenderPolicy::Untrusted`. Raw HTML is
escaped, and unsafe URL schemes are rejected after entity and control-character
normalization.

No change is needed for user-supplied Markdown:

```rust
let html = ferromark::to_html(user_supplied_markdown);
```

Content you fully control can opt into raw HTML passthrough:

```rust
use ferromark::{Options, RenderPolicy};

let options = Options {
    render_policy: RenderPolicy::Trusted,
    ..Options::default()
};
let html = ferromark::to_html_with_options(trusted_markdown, &options);
```

`disallowed_raw_html` remains the narrower GFM tag filter. It is not a general
HTML sanitizer.

## UTF-8 conversions return `Result`

`Range::slice_str`, `HtmlWriter::as_str`, and `HtmlWriter::into_string` now
validate UTF-8 and return their standard-library error types:

```rust
let text = range.slice_str(input)?;
let html = writer.into_string()?;
```

`Range::try_slice_str` remains as an alias for `slice_str`. The mutable writer
buffer is no longer public; write through `HtmlWriter` methods instead.

## MDX component names are validated

`MdxOutput::to_component` now returns `Result<String, ComponentNameError>`.
Names must be valid JavaScript identifiers and cannot be reserved words. This
prevents a caller-controlled name from changing the generated module:

```rust
let module = output.to_component("GettingStarted")?;
```

## The md4c benchmark is isolated

The cross-parser benchmark no longer affects normal builds or packages. Run it
from its dedicated crate and point it at an explicit md4c checkout:

```bash
cd benchmarks/md4c-comparison
MD4C_DIR=/path/to/md4c cargo bench --bench comparison
```
