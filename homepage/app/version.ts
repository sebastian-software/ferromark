// The documented version is the published one. Release Please updates
// `node/ferromark/package.json` in the release pull request, and the site reads
// it here at build time, so a release changes the rendered pages without a
// single homepage edit. `scripts/verify-build.mjs` fails the build when the
// prerendered output does not carry this version. See docs/releasing.md.
import packageJson from "../../node/ferromark/package.json";

export const version: string = packageJson.version;

export const docsRsUrl = `https://docs.rs/ferromark/${version}/ferromark/`;
