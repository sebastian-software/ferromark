# Code-block hook characterization

This local diagnostic compares three render entry points after adding whole-block
HTML replacements and ordered hook composition. It measures that hook
candidate's rendering cost, not the size or speed of a diagram engine.

These measurements and `validation.json` predate the native `CodeCompiler`
host added later on the same branch. The retained hashes identify that earlier
candidate, not the final branch state. No native plugin host, addon, or engine
performance claim follows from this report. The earlier manifest is retained as
[`candidate-Cargo.toml`](candidate-Cargo.toml); the measured Rust source files
still match their recorded hashes.

The [benchmark](../../../benches/code_block_hooks.rs) parses each fixture once,
reuses one renderer, and borrows the output buffer. Before timing it checks exact
output equality between default rendering, general node hooks, and code-only
hooks. The hooks decline every block; no external engine is loaded or timed.

Command:

```sh
cargo bench --locked --bench code_block_hooks -- \
  --sample-size 30 --warm-up-time 1 --measurement-time 2 --noplot
```

The repository bench profile uses opt-level 3, fat LTO, one codegen unit and the
generic CPU target. Host: macOS 27.0.1, arm64, Rust 1.95. Each lane gets one second
of warmup and two seconds of sampling. Two runs are retained. The initial run
may overlap Node formatting/typecheck/lint subprocesses; the confirmation ran
after the Node tests had ended. Host activity and thermal state are not controlled.

Median rendering time from the confirmation run:

| Fixture | Default | General hooks | Code-only hooks |
| --- | ---: | ---: | ---: |
| Short prose | 89.45 ns | 95.10 ns | 87.14 ns |
| Long prose, 80 repeated sections | 17.57 us | 19.27 us | 19.04 us |
| Forty ordinary code fences | 3.22 us | 3.99 us | 3.94 us |

The short-prose difference is a few nanoseconds and is not a general speed claim.
The long-prose fixture retains about 8% rendering overhead with code-only hooks;
the many-fence fixture retains about 23%. This entry point avoids per-inline-node
callbacks, but still uses the hook-aware block traversal and code-block fallback.
It does not establish performance neutrality. Parsing, JS callbacks, native-addon
boundaries, actual highlighting and diagram rendering are excluded.

Applications that do not install plugins continue to use the existing no-hook
render path. There is no Ferrigura dependency in the core or Node package.

[Environment and source hashes](environment.json), [summary](summary.json), and
both raw Criterion runs ([initial](initial/), [confirmation](confirmation/)) are
retained. The source revision is the branch's base; the source hashes identify
the modified candidate. This is a local characterization, not a cross-platform
or end-to-end benchmark.
