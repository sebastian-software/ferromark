# Homepage dependency audit policy

The homepage is a statically prerendered GitHub Pages site. Its production dependency graph is gated with:

```bash
pnpm audit --prod --audit-level high
```

High and critical advisories fail the build. Lower-severity findings are reviewed and patched when a compatible release exists.

As of 2026-07-24, the production audit reports no known vulnerabilities. The
homepage lockfile pins DOMPurify 3.4.12 or newer within Mermaid's compatible
range to include the fix for `GHSA-c2j3-45gr-mqc4`.
