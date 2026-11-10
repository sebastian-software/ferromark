# Upgrading from Ferromark v4 to v5

Version 5 upgrades the optional Rust `ferrigura` feature to Ferrigura 0.7.
The published Ferromark 4.0.0 crate uses Ferrigura 0.4; unreleased main also
used 0.6. Those pre-1.0 engine versions define distinct Rust types.
`FerriguraHooks::new` accepts the renderer re-exported by the current Ferromark,
so a separately constructed 0.4 or 0.6 `Renderer` no longer fits that parameter.
The engine error type exposed through `FerriguraError` changes with it.

## Rust: align the renderer dependency

Prefer the re-export when the application does not need a separate engine pin:

```toml
[dependencies]
ferromark = { version = "5", features = ["ferrigura"] }
```

```rust
use ferromark::ferrigura::{Renderer, Value};
use ferromark::FerriguraHooks;

let renderer = Renderer::new(&Value::empty_object());
let mut hooks = FerriguraHooks::new(renderer, "page")?;
```

If the application already depends directly on Ferrigura, upgrade that
requirement too:

```toml
[dependencies]
ferromark = { version = "5", features = ["ferrigura"] }
ferrigura = "0.7"
```

A `ferrigura::Renderer` from this dependency and a
`ferromark::ferrigura::Renderer` are then the same type. Existing hooks,
diagnostics, deterministic IDs, and the static SVG policy keep their APIs.
The [Ferrigura guide](ferrigura.md) shows caller-owned custom font registration;
embedded font bytes and CSS pass through the existing HTML and JSX hooks.

## Default Rust and Node.js consumers

The default Rust feature set does not enable Ferrigura. Its API does not need a
renderer migration. The standard Node.js addon still excludes the native
Ferrigura plugin. Applications using the injected JavaScript renderer keep the
same callback API and choose their engine version independently; this Rust
upgrade adds no font options to Ferromark's Node.js API.

For earlier upgrades, apply the [v3-to-v4 guide](migration-v4.md) first.
