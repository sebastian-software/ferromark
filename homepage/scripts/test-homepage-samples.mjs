/* eslint-disable security/detect-non-literal-fs-filename -- The fixture URL points to checked-in homepage samples. */
import { createElement, Fragment } from "ardo/react";
import { renderToStaticMarkup } from "ardo/react-dom/server";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createServer } from "vite";

import { verifyHomepageSamples } from "./homepage-samples.mjs";

const retained = {
  renderedWith: { ferromark: "4.0.0", "@ferriki/core": "1.3.0", "@ferrigura/core": "0.4.0" },
  code: { theme: "github-light-default" },
};
const attribution = (samples) =>
  `Real output of ferromark ${samples.renderedWith.ferromark}. ` +
  `@ferriki/core ${samples.renderedWith["@ferriki/core"]}, theme ${samples.code.theme} ` +
  `@ferrigura/core ${samples.renderedWith["@ferrigura/core"]}, static SVG`;

test("a later release candidate accepts honestly labelled retained samples", () => {
  assert.doesNotThrow(() =>
    verifyHomepageSamples(retained, `<span>4.1.0</span>${attribution(retained)}`),
  );
});

test("missing and malformed sample versions are rejected", () => {
  assert.throws(() => verifyHomepageSamples({}, ""), /release version/);
  for (const name of Object.keys(retained.renderedWith)) {
    for (const value of [undefined, null, "", "4.0", "04.0.0", "4.0.0-rc.01"]) {
      const bad = structuredClone(retained);
      bad.renderedWith[name] = value;
      assert.throws(() => verifyHomepageSamples(bad, attribution(retained)), /release version/);
    }
  }
});

test("recorded rc versions retain their exact visible attribution", () => {
  const samples = structuredClone(retained);
  samples.renderedWith.ferromark = "4.1.0-rc.2";
  assert.doesNotThrow(() => verifyHomepageSamples(samples, attribution(samples)));
});

test("each visible package attribution must match its recorded producer", () => {
  for (const version of Object.values(retained.renderedWith)) {
    const falseAttribution = attribution(retained).replace(version, "99.0.0");
    assert.throws(
      () => verifyHomepageSamples(retained, falseAttribution),
      /missing sample attribution/,
    );
  }
});

test("actual homepage sections retain producer labels without claiming the current release", async () => {
  const samples = JSON.parse(
    await readFile(new URL("../app/data/homepage-samples.json", import.meta.url), "utf8"),
  );
  const [major, minor] = samples.renderedWith.ferromark.split(".").map(Number);
  const laterTarget = `${major}.${minor + 1}.0`;
  const server = await createServer({
    configFile: false,
    server: { middlewareMode: true, hmr: false, ws: false },
    oxc: { jsx: { runtime: "automatic" } },
  });
  try {
    const { Ffm, Code, Mdx } = await server.ssrLoadModule("/app/components/feature-sections.tsx");
    const html = renderToStaticMarkup(
      createElement(Fragment, null, createElement(Ffm), createElement(Code), createElement(Mdx)),
    );
    assert.doesNotThrow(() => verifyHomepageSamples(samples, `<span>${laterTarget}</span>${html}`));
    assert.doesNotMatch(html, /the current(?: npm)? release/);
  } finally {
    await server.close();
  }
});
