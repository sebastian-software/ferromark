import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFile, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

if (process.argv.length !== 3) throw new Error("Usage: verify-plugin-bundle.mjs <bundle.json>");
const manifestPath = path.resolve(process.argv[2]);
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
assert.equal(path.basename(manifest.archive.file), manifest.archive.file);
const archive = path.join(path.dirname(manifestPath), manifest.archive.file);
const hash = (value) => createHash("sha256").update(value).digest("hex");
assert.equal(hash(await readFile(archive)), manifest.archive.sha256);
const directory = await mkdtemp(path.join(tmpdir(), "ferromark-plugin-consumer-"));
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
  const installed = path.join(directory, "node_modules/ferromark");
  assert.equal(
    hash(await readFile(path.join(installed, manifest.binary.file))),
    manifest.binary.sha256,
  );
  await copyFile(
    new URL("verify-plugin-bundle-worker.mjs", import.meta.url),
    path.join(directory, "verify.mjs"),
  );
  await copyFile(
    new URL("../../plugins/ferrigura/tests/fixtures/diagrams.json", import.meta.url),
    path.join(directory, "mermaid-fixtures.json"),
  );
  process.stdout.write(run(process.execPath, ["--expose-gc", "verify.mjs"]));
  await writeFile(
    path.join(path.dirname(manifestPath), "installation-test.json"),
    `${JSON.stringify(
      {
        target: manifest.target,
        plugins: manifest.plugins,
        archiveSha256: manifest.archive.sha256,
        status: "passed",
      },
      null,
      2,
    )}\n`,
  );
} finally {
  await rm(directory, { recursive: true, force: true });
}
