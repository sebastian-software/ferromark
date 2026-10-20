// The documented version is the published one. Release Please updates
// `node/ferromark/package.json` in the release pull request, and the site reads
// it through vite.config.ts at build time, so a release changes the pages without a
// single homepage edit. `scripts/verify-build.mjs` fails the build when the
// prerendered output does not carry this version. See docs/releasing.md.
declare const __FERROMARK_VERSION__: string;

export const version: string = __FERROMARK_VERSION__;

export const docsRsUrl = `https://docs.rs/ferromark/${version}/ferromark/`;
