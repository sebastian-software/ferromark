/* eslint-disable security/detect-non-literal-fs-filename -- Fixture URLs point to fixed checked-in benchmark data. */
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { createServer } from "vite";

const completed = JSON.parse(
  await readFile(new URL("../app/data/benchmark-platform-values.json", import.meta.url), "utf8"),
);
const projects = JSON.parse(
  await readFile(new URL("../app/data/benchmark-projects.json", import.meta.url), "utf8"),
);

async function renderComparison(figures, { claim = false } = {}) {
  const server = await createServer({
    configFile: false,
    server: { middlewareMode: true },
    oxc: { jsx: { runtime: "automatic" } },
    plugins: [
      {
        name: "comparison-test-data",
        load(id) {
          if (id.endsWith("/app/data/benchmark-platform-values.json")) {
            return JSON.stringify({ figures });
          }
          return null;
        },
      },
    ],
  });
  try {
    const { BenchmarkComparison, benchmarkLead } = await server.ssrLoadModule(
      "/app/components/benchmark-comparison.tsx",
    );
    return claim ? benchmarkLead() : renderToStaticMarkup(createElement(BenchmarkComparison));
  } finally {
    await server.close();
  }
}

test("a newly measured architecture adds one column and preserves existing cells", async () => {
  const baseline = await renderComparison(completed.figures);
  // Routing fixtures only: never published or presented as measured Intel data.
  const additional = projects.map((project) => ({
    ...completed.figures[0],
    ...project,
    platform: "macos-x86-64",
    platformLabel: "macOS x86-64",
    machine: "TEST FIXTURE",
    report: "docs/reports/test-fixture",
    overviewReport: "docs/reports/test-fixture",
  }));
  const extended = await renderComparison([...completed.figures, ...additional]);
  assert.match(extended, /<th scope="col">macOS x86-64<\/th>/);
  const before = [...baseline.matchAll(/<td>(.*?)<\/td>/gs)].map((match) => match[1]);
  const after = [...extended.matchAll(/<td>(.*?)<\/td>/gs)].map((match) => match[1]);
  assert.equal(before.length, projects.length * 2);
  assert.equal(after.length, projects.length * 3);
  for (let row = 0; row < projects.length; row += 1) {
    assert.deepEqual(after.slice(row * 3, row * 3 + 2), before.slice(row * 2, row * 2 + 2));
    assert.match(after[row * 3 + 2], /TEST FIXTURE/);
    assert.match(after[row * 3 + 2], /docs\/reports\/test-fixture/);
  }
});

test("all main candidates have GitHub links and measured cells; variants disclose their backend", async () => {
  const html = await renderComparison(completed.figures);
  for (const id of [
    "goldmark",
    "remarkable",
    "satteri",
    "md4x-napi",
    "ox-content-napi",
    "tanstack-markdown",
  ]) {
    const project = projects.find((row) => row.id === id);
    assert.ok(project);
    assert.ok(html.includes(`href="${project.github}"`));
    assert.ok(html.includes(`${project.label} · ${project.backend}`));
    assert.equal(completed.figures.filter((row) => row.id === id).length, 2);
  }
  assert.equal([...html.matchAll(/aria-label="Not measured"/g)].length, 0);
  assert.equal(projects.length, 20);
  for (const id of ["markdown-exit", "markdown-it-ts", "md4x-wasm"]) {
    assert.ok(!projects.some((project) => project.id === id));
  }
  assert.ok(!html.includes('class="ferromark-project-backend">WASM</span>'));
  assert.ok(html.includes('class="ferromark-project-backend">Native addon</span>'));
});

test("each runtime group is alphabetical and the syntax subset remains a separate note", async () => {
  const html = await renderComparison(completed.figures);
  assert.ok(html.includes('</a><span class="ferromark-project-backend">(syntax subset)</span>'));
  const names = [...html.matchAll(/class="ferromark-project-link"[^>]*>(.*?)<\/a>/gs)].map(
    (match) => match[1],
  );
  assert.deepEqual(names, [
    "Bun MD",
    "cmark",
    "cmark-gfm",
    "Comrak",
    "Goldmark",
    "markdown-rs",
    "md4c",
    "OX-Content",
    "pulldown-cmark",
    "commonmark.js",
    "markdown-it",
    "marked",
    "MD4X",
    "micromark",
    "OX-Content",
    "remark / unified",
    "Remarkable",
    "Sätteri",
    "Showdown",
    "TanStack Markdown",
  ]);
});

test("the overall speed claim requires every candidate on every displayed platform", async () => {
  assert.equal(
    await renderComparison(completed.figures, { claim: true }),
    "Ahead of every measured library.",
  );
  await assert.rejects(renderComparison(completed.figures.slice(1), { claim: true }));
  for (const changes of [
    { fresh: 1 },
    { fresh: Number.NaN },
    { documents: 56 },
    { corpusDocuments: 58 },
    { scoringScope: "equivalent-only" },
  ]) {
    const figures = completed.figures.map((figure) =>
      figure.id === "remarkable" && figure.platform === "linux-x86-64"
        ? { ...figure, ...changes }
        : figure,
    );
    await assert.rejects(renderComparison(figures, { claim: true }));
  }
});

test("full-corpus factors annotate output differences without changing legacy cells", async () => {
  const legacy = completed.figures.map((figure) => ({
    ...figure,
    scoringScope: undefined,
    agreeingDocuments: undefined,
    overviewReport: undefined,
  }));
  const baseline = await renderComparison(legacy);
  // Rendering fixture only; no new measurement or performance value is invented.
  const selected = legacy[0];
  const figures = legacy.map((figure) =>
    figure === selected
      ? {
          ...figure,
          scoringScope: "all-documents",
          documents: 57,
          agreeingDocuments: 31,
          overviewReport: "docs/reports/test-overview",
        }
      : figure,
  );
  const html = await renderComparison(figures);
  assert.equal([...html.matchAll(/<sup /g)].length, 1);
  assert.match(html, /<\/a><sup aria-label="Output differs for some inputs">\*<\/sup>/);
  assert.match(html, /57\/57 timed documents · 31 equivalent outputs/);
  assert.match(html, /All inputs contribute to the/);
  assert.match(
    html,
    /href="https:\/\/github.com\/sebastian-software\/ferromark\/tree\/main\/docs\/reports\/test-overview"/,
  );
  assert.ok(!baseline.includes("<tfoot>"));
  assertOnlyOneChangedCell(baseline, html);
});

function assertOnlyOneChangedCell(baseline, html) {
  const before = [...baseline.matchAll(/<td>(.*?)<\/td>/gs)].map((match) => match[1]);
  const after = [...html.matchAll(/<td>(.*?)<\/td>/gs)].map((match) => match[1]);
  assert.equal(before.length, after.length);
  let changed = 0;
  for (const [index, cell] of before.entries()) {
    if (cell !== after[index]) {
      changed += 1;
      assert.match(after[index], /Output differs for some inputs/);
    }
  }
  assert.equal(changed, 1);
}

test("full-corpus factors with equivalent outputs do not need an asterisk", async () => {
  const figures = completed.figures.map((figure) => ({
    ...figure,
    scoringScope: "all-documents",
    documents: 57,
    agreeingDocuments: 57,
  }));
  const html = await renderComparison(figures);
  assert.ok(!html.includes("<sup "));
  assert.ok(!html.includes("<tfoot>"));
});
