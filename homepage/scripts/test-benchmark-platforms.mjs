/* eslint-disable security/detect-non-literal-fs-filename -- Fixture URLs point to fixed checked-in benchmark data. */
import { createElement } from "ardo/react";
import { renderToStaticMarkup } from "ardo/react-dom/server";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createServer } from "vite";

const completed = JSON.parse(
  await readFile(new URL("../app/data/benchmark-platform-values.json", import.meta.url), "utf8"),
);
const projects = JSON.parse(
  await readFile(new URL("../app/data/benchmark-projects.json", import.meta.url), "utf8"),
);

async function renderComparison(figures, { claim = false, conformance = false, rangeId } = {}) {
  const server = await createServer({
    configFile: false,
    server: { middlewareMode: true, hmr: false, ws: false },
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
    const { BenchmarkComparison, ConformanceComparison, benchmarkLead, benchmarkRange } =
      await server.ssrLoadModule("/app/components/benchmark-comparison.tsx");
    if (claim) return benchmarkLead();
    if (rangeId) return benchmarkRange(rangeId);
    return renderToStaticMarkup(
      createElement(conformance ? ConformanceComparison : BenchmarkComparison),
    );
  } finally {
    await server.close();
  }
}

test("a newly measured architecture adds a selectable panel and preserves existing measurements", async () => {
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
  assert.match(extended, /<option value="macos-x86-64">macOS x86-64<\/option>/);
  const before = [...baseline.matchAll(/<td>(.*?)<\/td>/gs)].map((match) => match[1]);
  const after = [...extended.matchAll(/<td>(.*?)<\/td>/gs)].map((match) => match[1]);
  assert.equal(before.length, projects.length * 2);
  assert.equal(after.length, projects.length * 3);
  assert.deepEqual(after.slice(0, before.length), before);
  for (const cell of after.slice(before.length)) {
    assert.match(cell, /TEST FIXTURE/);
    assert.match(cell, /docs\/reports\/test-fixture/);
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
    assert.ok(
      html.includes(`${project.label} · ${project.backend.replace(" (syntax subset)", "")}`),
    );
    assert.equal(completed.figures.filter((row) => row.id === id).length, 2);
  }
  assert.equal([...html.matchAll(/aria-label="Not measured"/g)].length, 0);
  assert.equal(projects.length, 20);
  for (const id of ["markdown-exit", "markdown-it-ts", "md4x-wasm"]) {
    assert.ok(!projects.some((project) => project.id === id));
  }
  assert.ok(!html.includes('class="ferra-benchmark-detail">WASM</span>'));
  assert.ok(html.includes('class="ferra-benchmark-detail">Native addon</span>'));
});

test("each runtime group is alphabetical and implementation details remain visible", async () => {
  const html = await renderComparison(completed.figures);
  for (const backend of ["Rust", "C", "Go", "JavaScript"]) {
    assert.ok(html.includes(`class="ferra-benchmark-detail">${backend}</span>`));
  }
  assert.ok(!html.includes("(syntax subset)"));
  const names = [...html.matchAll(/class="ferra-benchmark-source"[^>]*>(.*?)<\/a>/gs)].map(
    (match) => match[1],
  );
  const expected = [
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
  ];
  assert.deepEqual(names, [...expected, ...expected]);
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

test("one differing platform annotates the library name once and preserves other cells", async () => {
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
  assert.match(html, /pulldown-cmark<\/a><sup /);
  assert.ok(![...html.matchAll(/<td>(.*?)<\/td>/gs)].some((match) => match[1].includes("<sup ")));
  assert.match(html, /57\/57 timed documents · 31 equivalent outputs/);
  assert.match(html, /All inputs contribute to the/);
  assert.match(
    html,
    /href="https:\/\/github.com\/sebastian-software\/ferromark\/tree\/main\/docs\/reports\/test-overview"/,
  );
  assert.ok(!baseline.includes("* Output differs"));
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
      assert.match(after[index], /31 equivalent outputs/);
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
  assert.ok(!html.includes("* Output differs"));
});

test("spec agreement covers every variant with separate references and evidence links", async () => {
  const html = await renderComparison(completed.figures, { conformance: true });
  const conformance = JSON.parse(
    await readFile(new URL("../app/data/comparison-conformance.json", import.meta.url), "utf8"),
  );
  assert.equal(conformance.rows.length, projects.length + 2);
  assert.equal(
    [...html.matchAll(/class="ferra-benchmark-conformance"/g)].length,
    (projects.length + 2) * 2,
  );
  assert.equal([...html.matchAll(/class="ferromark-reference-row"/g)].length, 2);
  assert.match(html, /28 extension examples, not full GFM/);
  assert.match(html, /GFM Unsupported/);
  assert.ok(!html.includes("GFM 0.0%"));
  for (const row of conformance.rows) {
    assert.ok(html.includes(`${conformance.report}/${row.evidence}`));
    for (const suite of ["commonmark", "gfm"]) {
      const result = row.suites[suite];
      if (result.status === "measured") {
        assert.ok(
          html.includes(
            `${result.passed}/${result.total} examples (${result.percent.toFixed(2)}%)`,
          ),
        );
      }
    }
  }
});

test("speed and conformance are separate tables", async () => {
  const speed = await renderComparison(completed.figures);
  const spec = await renderComparison(completed.figures, { conformance: true });
  assert.ok(!speed.includes("ferra-benchmark-conformance"));
  assert.ok(!spec.includes("times the throughput"));
  assert.match(speed, /aria-controls="markdown-ecosystem-results"/);
  assert.equal([...speed.matchAll(/<div hidden=""/g)].length, 1);
});

test("homepage ranges use the measured hosts and reject incomplete examples", async () => {
  assert.equal(
    await renderComparison(completed.figures, { rangeId: "pulldown-cmark" }),
    "2.3–2.7×",
  );
  assert.equal(await renderComparison(completed.figures, { rangeId: "marked" }), "9.5–9.7×");
  await assert.rejects(renderComparison(completed.figures.slice(1), { rangeId: "pulldown-cmark" }));
});
