# Current Ferriki release checks

Both CI lanes test the current Ferromark source with published Ferriki 1.3.0.
The shared runner, authored cases, Rust example, and exact Rust/Node expectations
come from the existing [1.1.2 reference contract](../ferriki-compatibility/README.md).
Its source metadata, npm lock, cases, snapshots, and example contents remain unchanged.
The runner writes a temporary case document with only the tested release identity
changed, then executes every original assertion against the same expected output.

```sh
cargo fetch --locked
python3 scripts/ferriki-compatibility/run.py /tmp/ferriki-current-rust --release scripts/ferriki-current
npm ci --prefix scripts/ferriki-current --ignore-scripts --no-audit --no-fund
python3 scripts/ferriki-compatibility/run.py /tmp/ferriki-current-peers --node --release scripts/ferriki-current
```

Use fresh output directories. The Rust-only command does not install or invoke
Node. The peer command builds the current Ferromark addon and selects the exact
npm peer installed here. Both package and unpack the current Cargo archive,
retain locked registry identities, and check all 14 standalone and 14 Markdown
cases, UTF-8 offsets, token scopes/types, errors, asset failures, reuse, and
allocator resets. The peer lane also retains raw HTML snapshots, visible-text
and error parity, dual-theme output, and missing/corrupt Node cache checks.

`asset-source.json` pins the official npm archive integrity and asset commit;
every downloaded asset retains its size and SHA-256 check. Ferriki 1.3 adds
prepared `.fkbc` sidecars alongside raw `.fkgram` grammars and `.fktheme` themes.
The runner accepts those language sidecars at their immutable hash paths and
makes all verified release assets available to both peers. It does not bypass
asset verification or disable prepared loading.

The evidence summary distinguishes `fixtureBaselineFerriki: 1.1.2` from the
actually tested `ferriki: 1.3.0`, asset commit, and npm archive integrity.
Current release selection cannot update the historical snapshots. Without
`--release`, the original runner still selects the 1.1.2 reference and requires
matching locked crates; it is not the current-product CI lane. No old Ferromark
source checkout substitutes for current-product coverage.
