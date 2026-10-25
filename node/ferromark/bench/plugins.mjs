// Characterize the installed custom addon; process RSS includes Node/V8 and
// native allocations. Plain-code output and SVG rendering are different work.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { cpus, release, tmpdir, totalmem } from "node:os";
import path from "node:path";

import { fixtures } from "./plugins/fixtures.mjs";

if (process.argv.length !== 4)
  throw new Error("Usage: plugins.mjs <bundle.json> <new-output-directory>");
const manifestPath = path.resolve(process.argv[2]);
const output = path.resolve(process.argv[3]);
const bundle = JSON.parse(await readFile(manifestPath, "utf8"));
const hasFerrigura = bundle.plugins.includes("ferrigura");
assert.equal(path.basename(bundle.archive.file), bundle.archive.file);
const archive = path.join(path.dirname(manifestPath), bundle.archive.file);
const hash = (data) => createHash("sha256").update(data).digest("hex");
const median = (values) => values.toSorted((a, b) => a - b)[Math.floor(values.length / 2)];
const warmMedian = (run) => median(run.samples.map((sample) => sample.millisecondsPerRender));
assert.equal(hash(await readFile(archive)), bundle.archive.sha256);
await mkdir(output);
const directory = await mkdtemp(path.join(tmpdir(), "ferromark-plugin-bench-"));
const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, { cwd: directory, encoding: "utf8", ...options });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  return result.stdout;
};
try {
  await writeFile(path.join(directory, "package.json"), '{"private":true,"type":"module"}');
  run(
    process.platform === "win32" ? "npm.cmd" : "npm",
    ["install", "--offline", "--ignore-scripts", "--no-audit", "--no-fund", archive],
    {
      shell: process.platform === "win32",
      env: { ...process.env, npm_config_cache: path.join(directory, "npm-cache") },
    },
  );
  for (const file of ["worker.mjs", "fixtures.mjs"])
    await copyFile(new URL(`plugins/${file}`, import.meta.url), path.join(directory, file));
  const cases = ["html", "jsx"].flatMap((format) => [
    ...["plain", "empty", ...(hasFerrigura ? ["ferrigura"] : [])].map((lane) => [
      "ordinary",
      format,
      lane,
    ]),
    ...(hasFerrigura
      ? ["small", "medium", "mixed"].map((fixture) => [fixture, format, "ferrigura"])
      : []),
  ]);
  const runs = [];
  // Rotate and reverse the order across independent fresh processes.
  for (let repetition = 0; repetition < 3; repetition++) {
    const offset = repetition * 4;
    const order = [...cases.slice(offset), ...cases.slice(0, offset)];
    if (repetition % 2) order.reverse();
    for (const args of order)
      runs.push({
        repetition,
        ...JSON.parse(run(process.execPath, ["--expose-gc", "worker.mjs", ...args])),
      });
  }
  const summary = cases.map(([fixture, format, lane]) => {
    const group = runs.filter(
      (r) => r.fixture === fixture && r.format === format && r.lane === lane,
    );
    const latencies = group.map((run) => warmMedian(run));
    return {
      fixture,
      format,
      lane,
      inputBytes: group[0].inputBytes,
      outputBytes: group[0].outputBytes,
      warmMs: {
        median: median(latencies),
        min: Math.min(...latencies),
        max: Math.max(...latencies),
      },
      loadMs: median(group.map((r) => r.loadMs)),
      initMs: median(group.map((r) => r.initMs)),
      firstRenderMs: median(group.map((r) => r.firstRenderMs)),
      retainedRssBytes: median(group.map((r) => r.memory.retained.rss)),
      peakRssBytes: median(group.map((r) => r.memory.maxRssBytes)),
      rssIncreaseBytes: median(group.map((r) => r.memory.retained.rss - r.memory.before.rss)),
    };
  });
  const result = {
    dateUtc: new Date().toISOString(),
    environment: {
      node: process.version,
      platform: process.platform,
      arch: process.arch,
      os: release(),
      cpu: cpus()[0]?.model,
      logicalCpus: cpus().length,
      totalMemoryBytes: totalmem(),
      rust:
        bundle.rustVersion ??
        run("rustc", ["-V"], { cwd: path.resolve(import.meta.dirname, "../../..") }).trim(),
      profile: "release-node; PGO disabled",
    },
    method: {
      independentProcessesPerCase: 3,
      warmupRenders: 5,
      batchesPerProcess: 11,
      minimumBatchMs: 20,
      gc: "before load and after warmup/measurement, never inside timed batches",
      memory:
        "whole-process RSS and high-water RSS, includes Node/V8, output allocation and native caches",
      limits:
        "Local shared developer host; no controlled thermals, Node startup timing, allocator attribution, cross-platform or PGO claim. Ordinary output is checked equal; diagram cases characterize different work.",
    },
    bundle,
    fixtureSha256: Object.fromEntries(
      Object.entries(fixtures).map(([name, fixture]) => [name, hash(fixture.source)]),
    ),
    summary,
    runs,
  };
  await writeFile(path.join(output, "results.json"), `${JSON.stringify(result, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
} finally {
  await rm(directory, { recursive: true, force: true });
}
