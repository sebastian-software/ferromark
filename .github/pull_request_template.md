## Summary

<!-- What changed and why, in one or two sentences. -->

## Changes

<!-- The notable changes, one bullet each. Call out user-visible or breaking behavior. -->

## Validation

<!-- The commands you ran and their result, or a note that this change is documentation-only. -->

## Issue

<!-- Closes #123, Refs #123, or a short note on why no issue exists. -->

## Checklist

- [ ] The change is focused and preserves existing compatibility unless documented.
- [ ] Tests cover behavior changes and regressions.
- [ ] Public APIs and user-facing behavior are documented.
- [ ] The [required local checks](../CONTRIBUTING.md#required-local-checks) pass:
      `cargo test --locked --all-features`,
      `cargo clippy --all-targets --all-features --locked -- -D warnings` and
      `cargo fmt --check`.
- [ ] Touched repository contracts under `scripts/` were run, and the Node
      workspace checks (`pnpm build`, `pnpm test`) when `node/` changed.
