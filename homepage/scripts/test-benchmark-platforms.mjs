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

async function renderComparison(figures) {
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
    const { BenchmarkComparison } = await server.ssrLoadModule(
      "/app/components/benchmark-comparison.tsx",
    );
    return renderToStaticMarkup(createElement(BenchmarkComparison));
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

test("new candidates have GitHub links and unmeasured cells; variants disclose their backend", async () => {
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
    assert.ok(!completed.figures.some((row) => row.id === id));
  }
  assert.equal([...html.matchAll(/aria-label="Not measured"/g)].length, 12);
  assert.equal(projects.length, 20);
  for (const id of ["markdown-exit", "markdown-it-ts", "md4x-wasm"]) {
    assert.ok(!projects.some((project) => project.id === id));
  }
  assert.ok(!html.includes('class="ferromark-project-backend">WASM</span>'));
  assert.ok(html.includes('class="ferromark-project-backend">Native addon</span>'));
});
