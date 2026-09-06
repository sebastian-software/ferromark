# Ferromark documentation

Project documentation that does not belong in the [README](../README.md) or in
the [API reference on docs.rs](https://docs.rs/ferromark). The rendered guide
for consumers lives at
<https://sebastian-software.github.io/ferromark/>.

## Migrating

Upgrade guides, oldest first. Each guide covers the breaking changes of the
release it names; when crossing several releases, apply the guides in order.

| Guide | Covers | Main changes |
| --- | --- | --- |
| [0.2 migration guide](migration-0.2.md) | to 0.2 | Untrusted rendering becomes the default; fallible UTF-8 and MDX entry points |
| [0.3 migration guide](migration-0.3.md) | to 0.3 | Removed `std`, `neon`, and `trace` Cargo features; new integration APIs |
| [0.4–0.7 migration guide](migration-0.4.md) | 0.3.3 → 0.7.0 | `Profile` removal, inline-parser argument, `Options` no longer `Copy`, non-exhaustive fenced-code events, Rust and Node.js floors |
| [0.8 migration guide](migration-0.8.md) | to 0.8 | Non-exhaustive `Options` and event enums, crate-root imports |

## Reference

| Area | Contents |
| --- | --- |
| [arch/](arch/) | Architecture decision records (`ADR-*`), the parser comparison matrix (`ARCH-COMP-*`), performance experiments (`ARCH-EXP-*`), and the optimization backlog (`ARCH-PLAN-001`) |
| [plans/](plans/) | Dated design and implementation plans for larger work packages |
| [reports/](reports/) | Dated measurement artifacts: profiling runs, audits, and their raw JSON |
| [releasing.md](releasing.md) | Release process for the crate and the npm package |

Contribution workflow and the required local checks live in
[CONTRIBUTING.md](../CONTRIBUTING.md); agent-specific guidance lives in
[AGENTS.md](../AGENTS.md).
