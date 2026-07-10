# Homepage dependency audit policy

The homepage is a statically prerendered GitHub Pages site. Its production dependency graph is gated with:

```bash
pnpm audit --prod --audit-level high
```

High and critical advisories fail the build. Lower-severity findings are reviewed and patched when a compatible release exists.

As of 2026-07-10, the audit reports one low-severity build-tool finding: `GHSA-4x5r-pxfx-6jf8` in `@babel/core 7.29.0`, reached through Ardo's React Router toolchain. The advisory names `7.29.1` as the patched release, but that version is not published; the next published major is Babel 8. This exception does not ship a server or development endpoint on the deployed static site and should be removed when the upstream toolchain adopts a compatible patched release.
