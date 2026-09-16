# Homepage dependency audit policy

The homepage is a statically prerendered GitHub Pages site. Its production dependency graph is gated with:

The `homepage` job in [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)
runs the package script below:

```bash
pnpm run audit
```

That script expands to the production-only gate below:

```bash
pnpm audit --prod --audit-level high
```

High and critical advisories fail the build. Lower-severity findings are reviewed and patched when a compatible release exists.

Run the command before release and record its result with the release
validation; this policy intentionally avoids pinning advisory counts or
dependency versions that change as fixes are released.

This document covers homepage dependency auditing. Vulnerability reports for
the project belong in the [root security policy](../SECURITY.md).
