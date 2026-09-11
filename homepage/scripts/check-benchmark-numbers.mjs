// Contract check: the published benchmark figures exist exactly once.
//
// app/data/benchmarks.json is the single source. This script fails the homepage
// build when the repository README no longer agrees with it, or when a homepage
// source hard-codes a throughput figure instead of rendering it from the JSON.
import { spawnSync } from "node:child_process";
import { readFile, readdir } from "node:fs/promises";
import { join, relative } from "node:path";

const homepageRoot = new URL("../", import.meta.url);
const repositoryRoot = new URL("../../", import.meta.url);
const dataUrl = new URL("app/data/benchmarks.json", homepageRoot);
const readmeUrl = new URL("README.md", repositoryRoot);

const failures = [];

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}

/** Collapse whitespace so line wrapping never changes the meaning of a sentence. */
function normalize(text) {
  return text.replace(/\s+/g, " ").trim();
}

/** Casing is decided per document (prose vs. package name), so compare case-insensitively. */
function contains(haystack, needle) {
  return normalize(haystack).toLowerCase().includes(normalize(needle).toLowerCase());
}

function readmeSection(readme, heading) {
  const start = readme.indexOf(`## ${heading}\n`);
  if (start === -1) {
    return null;
  }
  const rest = readme.slice(start + heading.length + 4);
  const end = rest.search(/^## /m);
  return end === -1 ? rest : rest.slice(0, end);
}

/** Every pipe table in document order, cells stripped of Markdown emphasis. */
function pipeTables(markdown) {
  const tables = [];
  let current = null;
  for (const line of markdown.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed.startsWith("|")) {
      current = null;
      continue;
    }
    const rawCells = trimmed
      .slice(1, trimmed.endsWith("|") ? -1 : undefined)
      .split("|")
      .map((cell) => cell.trim());
    const cells = rawCells.map((cell) => cell.replaceAll("*", ""));
    if (cells.every((cell) => /^:?-{2,}:?$/.test(cell))) {
      continue;
    }
    if (!current) {
      current = { header: cells, rows: [], rawRows: [] };
      tables.push(current);
      continue;
    }
    current.rows.push(cells);
    current.rawRows.push(rawCells);
  }
  return tables;
}

const benchmarks = JSON.parse(await readFile(dataUrl, "utf8"));
const nativeBenchmarks = JSON.parse(
  await readFile(new URL("app/data/native-benchmarks.json", homepageRoot), "utf8"),
);
for (const table of [...benchmarks.tables, ...benchmarks.featureTables]) {
  const fastest = Math.min(...table.rows.map((row) => row.medianNs));
  check(
    table.rows[0]?.parser === "ferromark" && table.rows[1]?.parser === "pulldown-cmark",
    `${table.id}: pulldown-cmark must appear directly after ferromark`,
  );
  for (const row of table.rows) {
    check(
      row.winner === (row.medianNs === fastest),
      `${table.id}/${row.parser}: winner must follow the unrounded measured time`,
    );
  }
}
const readme = await readFile(readmeUrl, "utf8");
const section = readmeSection(readme, "Benchmarks");

if (!section) {
  failures.push("README.md has no '## Benchmarks' section to compare against");
} else {
  const { run, competitors, tables, summary, conditions, scope } = benchmarks;

  for (const fact of [run.machine, run.os, `rustc ${run.rustc}`, run.date]) {
    check(contains(section, fact), `README benchmarks section must state ${JSON.stringify(fact)}`);
  }
  for (const [name, version] of Object.entries(competitors)) {
    const expected = ["md4c", "bun"].includes(name) ? `${name} @ ${version}` : `${name} ${version}`;
    check(
      contains(section, expected),
      `README benchmarks section must state ${JSON.stringify(expected)}`,
    );
  }
  for (const [label, sentence] of [
    ["summary", summary],
    ["conditions", conditions],
    ["scope", scope],
  ]) {
    check(
      contains(section, sentence),
      `README benchmarks ${label} must read ${JSON.stringify(normalize(sentence))}`,
    );
  }

  const readmeTables = pipeTables(section).filter(
    (table) => table.header[0]?.toLowerCase() === "parser",
  );
  check(
    readmeTables.length === tables.length,
    `README benchmarks section has ${readmeTables.length} parser tables, benchmarks.json has ${tables.length}`,
  );
  tables.forEach((table, index) => {
    const readmeTable = readmeTables[index];
    if (!readmeTable) {
      return;
    }
    check(
      readmeTable.rows.length === table.rows.length,
      `README table ${index + 1} (${table.id}) has ${readmeTable.rows.length} rows, benchmarks.json has ${table.rows.length}`,
    );
    table.rows.forEach((row, rowIndex) => {
      const readmeRow = readmeTable.rows[rowIndex];
      if (!readmeRow) {
        return;
      }
      const expected = [row.parser, row.latency, row.throughput, row.ratio];
      const actual = readmeRow;
      check(
        readmeTable.rawRows[rowIndex].every((cell) => /^\*\*.*\*\*$/.test(cell) === row.winner),
        `README ${table.id}/${row.parser}: bold must identify the measured winner`,
      );
      check(
        expected.every((value, cell) => value.toLowerCase() === (actual[cell] ?? "").toLowerCase()),
        `README table ${index + 1} (${table.id}) row ${rowIndex + 1} is ${JSON.stringify(actual)}, benchmarks.json says ${JSON.stringify(expected)}`,
      );
    });
  });

  const featureTables = pipeTables(section).filter(
    (table) => table.header[0] === "Input / feature set",
  );
  const expectedFeatures = benchmarks.featureTables.map((table) => [
    table.label,
    String(table.inputBytes),
    ...table.rows.map((row) => row.latency),
  ]);
  check(
    featureTables.length === 1 &&
      JSON.stringify(featureTables[0].rows) === JSON.stringify(expectedFeatures),
    "README feature matrix must match every measured feature-set row",
  );

  const featureMatrix = featureTables[0];
  if (featureMatrix) {
    check(
      JSON.stringify(featureMatrix.header.slice(2)) ===
        JSON.stringify(benchmarks.featureTables[0].rows.map((row) => row.parser)),
      "README feature parser columns must follow the same order as the data",
    );
    benchmarks.featureTables.forEach((table, index) => {
      table.rows.forEach((row, column) => {
        const cell = featureMatrix.rawRows[index]?.[column + 2] ?? "";
        check(
          /^\*\*.*\*\*$/.test(cell) === row.winner,
          `README feature ${table.id}/${row.parser}: bold must identify the measured winner`,
        );
      });
    });
  }

  const known = new Set(
    [...tables, ...nativeBenchmarks.tables].flatMap((table) =>
      table.rows.map((row) => row.throughput),
    ),
  );
  for (const [figure] of section.matchAll(/\d+(?:\.\d+)?\s*MiB\/s/g)) {
    check(
      known.has(normalize(figure)),
      `README benchmarks section states ${JSON.stringify(figure)}, which is not a value in benchmarks.json`,
    );
  }
}

async function sources(directory) {
  const found = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      found.push(...(await sources(path)));
    } else if (/\.(tsx?|mdx|css)$/.test(entry.name)) {
      found.push(path);
    }
  }
  return found;
}

for (const path of await sources(new URL("app", homepageRoot).pathname)) {
  const contents = await readFile(path, "utf8");
  const figures = [...contents.matchAll(/\d+(?:\.\d+)?\s*MiB\/s/g)].map(([figure]) => figure);
  check(
    figures.length === 0,
    `${relative(new URL(".", homepageRoot).pathname, path)} hard-codes ${JSON.stringify(figures.join(", "))}; render it from app/data/benchmarks.json instead`,
  );
}

const guide = await readFile(new URL("app/routes/guide/benchmarks.mdx", homepageRoot), "utf8");
check(
  guide.includes(benchmarks.competitors.md4c),
  `the benchmarks guide must check out the pinned md4c revision ${benchmarks.competitors.md4c}`,
);

const provenance = spawnSync("python3", ["benchmarks/bun-comparison/publish.py", "--check"], {
  cwd: repositoryRoot,
  encoding: "utf8",
});
check(
  provenance.status === 0,
  `Published tables must agree with verified raw samples: ${provenance.stderr || provenance.error || provenance.stdout}`,
);

const nativeProvenance = spawnSync(
  "python3",
  ["benchmarks/native-pipeline-comparison/publish.py", "--check"],
  {
    cwd: repositoryRoot,
    encoding: "utf8",
  },
);
check(
  nativeProvenance.status === 0,
  `Native pairs must agree with verified raw samples: ${nativeProvenance.stderr || nativeProvenance.error || nativeProvenance.stdout}`,
);

if (failures.length > 0) {
  console.error("Benchmark figures are out of sync with homepage/app/data/benchmarks.json:");
  for (const failure of failures) {
    console.error(`  - ${failure}`);
  }
  process.exit(1);
}

console.log("Benchmark figures match README.md and are rendered from app/data/benchmarks.json");
