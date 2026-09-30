import assert from "node:assert/strict";
import { copyFile, mkdir, mkdtemp, rm, symlink } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { benchmarkTarget } from "./benchmark-target.mjs";
import { nativeTarget } from "../../node/ferromark/native-target.mjs";

// CPU target routing is separate from whether a sidecar is published.
test("local benchmark targets support both CPU families on macOS and Linux", () => {
  for (const [platform, arch, libc, expected] of [
    ["darwin", "arm64", undefined, "darwin-arm64"],
    ["darwin", "x64", undefined, "darwin-x64"],
    ["linux", "x64", "gnu", "linux-x64-gnu"],
    ["linux", "x64", "musl", "linux-x64-musl"],
    ["linux", "arm64", "gnu", "linux-arm64-gnu"],
    ["linux", "arm64", "musl", "linux-arm64-musl"],
  ]) {
    assert.equal(benchmarkTarget(platform, arch, libc), expected);
  }
  assert.throws(() => nativeTarget("darwin", "x64"), /does not support/);
  assert.throws(() => benchmarkTarget("linux", "riscv64"), /does not support/);
});

test(
  "private Intel-Mac loader calls the unchanged public facade on the supplied local addon",
  { skip: !process.env.FERROMARK_BENCH_TEST_ADDON },
  async () => {
    // Exercise Intel-Mac filename selection in a real child process with a
    // native addon built for this test host. This verifies loader wiring, not
    // Intel machine performance or cross-architecture binary compatibility.
    const fixture = await mkdtemp(path.join(os.tmpdir(), "ferromark loader "));
    try {
      const benchmark = path.join(fixture, "benchmarks/markdown-ecosystem");
      const facade = path.join(fixture, "node/ferromark");
      await mkdir(benchmark, { recursive: true });
      await mkdir(facade, { recursive: true });
      for (const file of ["benchmark-target.mjs", "benchmark-facade.mjs"]) {
        await copyFile(new URL(file, import.meta.url), path.join(benchmark, file));
      }
      for (const file of ["index.mjs", "native-target.mjs"]) {
        await copyFile(new URL(`../../node/ferromark/${file}`, import.meta.url), path.join(facade, file));
      }
      await symlink(path.resolve(process.env.FERROMARK_BENCH_TEST_ADDON), path.join(facade, "ferromark.darwin-x64.node"));
      const result = spawnSync(
        process.execPath,
        ["--input-type=module", "-e", `
          import assert from 'node:assert/strict';
          import { readFileSync } from 'node:fs';
          import { loadBenchmarkFacade } from './benchmarks/markdown-ecosystem/benchmark-facade.mjs';
          Object.defineProperty(process, 'platform', { value: 'darwin' });
          Object.defineProperty(process, 'arch', { value: 'x64' });
          const facade = await loadBenchmarkFacade();
          assert.equal(facade.toHtml('**hello**'), '<p><strong>hello</strong></p>\\n');
          assert.equal(new facade.Renderer().toHtml('**hello**'), facade.toHtml('**hello**'));
          let stack;
          try { facade.toHtml('', { unexpectedOption: true }); } catch (error) { stack = error.stack; }
          const loaded = stack.match(new RegExp('file://[^()]+/index.mjs'))[0];
          assert.deepEqual(readFileSync(new URL(loaded)), readFileSync('./node/ferromark/index.mjs'));

        `],
        { cwd: fixture, encoding: "utf8" },
      );
      assert.equal(result.status, 0, result.stderr);
    } finally {
      await rm(fixture, { recursive: true, force: true });
    }
  },
);
