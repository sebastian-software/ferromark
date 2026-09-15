# V2 repository integration validation

Validated locally on macOS arm64 with Rust 1.95.0, Node.js 24.15.0, and pnpm
11.17.0. The imported source is the clean sibling checkout at
`e9dd98bbd29c458dd18c48f80d839d94b3fcc77f`; the Ferromark parent is
`523e77e6f873a29cc7a5d2ce08006cb73489ad45`.

## Results

| Check | Result |
| --- | --- |
| Rust all-feature tests, including doctests | 791 passed |
| Workspace Clippy with warnings denied | Passed |
| Workspace rustfmt | Passed with the inherited managed configuration |
| Workspace benchmark compilation | Passed |
| Rustdoc with warnings denied | Passed |
| Rust line coverage | 90.13%; inherited 90% gate retained |
| Node tests | 27 passed, plus native panic-unwind verification |
| Node TypeScript, lint, package contents | Passed |
| Clean consumer installation from local tarballs | Passed for 2.0.0-dev.0 on macOS arm64 |
| Website TypeScript and static build | Passed; all six routes verified |
| Website browser check | Desktop and 390px mobile layout, guide navigation, no console errors |
| Python benchmark harness tests | 76 passed |
| Core benchmark snapshot | Resolves all workspace members, including node/native |
| Repository contracts | 10 passed, plus workflow-pin fixture checks |
| README themes and Ferramenta block | Match pinned generators |
| Benchmark README source | Matches the frozen report publisher |
| Standards 0.11.1 | Passed |
| cargo-deny 0.20.2 | Advisories, bans, licenses, and sources passed |
| Node dependency audit | No known vulnerabilities |
| Website production audit | High-severity gate passed; 1 low and 6 moderate findings remain |

Coverage was measured with cargo-llvm-cov 0.9.0 and all tests enabled. CI runs
instrumented tests serially: a parallel local build caused a timing-ratio test
to fail under instrumentation. The complete serial run passed without skips.
New corpus-wide hook equivalence tests verify that installing a Node highlighter
cannot change unrelated Markdown output, including reused renderer state.

## Preservation checks

- All 1,275 imported report, snapshot, and specification-fixture files match
  the sibling checkout byte for byte.
- Fifty older CSVs required restoring their original CRLF bytes because the
  source repository's Git attributes had normalized their stored text. All
  report CSVs now opt out of text conversion; numerical values are unchanged.
- Every existing registry dependency version and checksum in the v2 lockfile
  remains unchanged. The Node binding adds 17 registry packages.
- Original upstream MIT attribution and separately licensed specification
  fixtures remain intact; npm archives include upstream and local MIT notices.
- The sibling repository and the original Ferromark checkout remain unchanged.

## CI and release boundaries

The six-configuration Rust OS/toolchain matrix, eight native package targets,
and Node.js 22.12.0 consumer floor are configured in CI; only the local arm64
platform and Node.js 24.15.0 were executed here. Cross-platform CI has not run
as part of this local import. No branch was pushed and no packages or website
were published. Release enablement remains a separate step described in
[the release guide](releasing.md).
