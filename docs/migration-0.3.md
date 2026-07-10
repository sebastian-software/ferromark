# Migrating to ferromark 0.3

ferromark 0.3 adds new integration APIs and removes Cargo features that did not enable supported behavior.

## Remove unused Cargo feature names

The `std`, `neon`, and `trace` Cargo features have been removed:

- `std` did not provide a `no_std` alternative.
- `neon` did not enable Node bindings. The 0.3 Node package uses napi-rs and is distributed through npm instead.
- `trace` did not enable tracing.

If your dependency declaration explicitly enables any of these names, remove them:

```toml
# Before
ferromark = { version = "0.2", features = ["std"] }

# After
ferromark = "0.3"
```

`mdx` remains the only opt-in Cargo feature:

```toml
ferromark = { version = "0.3", features = ["mdx"] }
```
