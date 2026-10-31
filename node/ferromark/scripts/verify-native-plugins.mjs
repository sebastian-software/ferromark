import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const output = await mkdtemp(path.join(tmpdir(), "ferromark-native-plugins-"));
const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8", ...options });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${command} failed:\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
};

try {
  // This is the release dependency graph, not the all-features workspace graph.
  const graph = run("cargo", [
    "tree",
    "--locked",
    "-p",
    "ferromark-node",
    "--edges",
    "normal,build",
    "--prefix",
    "none",
  ]);
  const packages = graph.split("\n").map((line) => line.split(" ", 1)[0]);
  assert.ok(
    packages.every(
      (name) =>
        !name.startsWith("ferriki") &&
        !name.startsWith("ferrigura") &&
        !["ferromark-plugin-fixture", "cssparser", "rustls", "ureq", "onig"].includes(name),
    ),
    "the engine-free build must omit engines, their parsers and assets, and diagnostic plugins",
  );
  run(process.execPath, [fileURLToPath(new URL("build-native.mjs", import.meta.url))], {
    env: {
      ...process.env,
      FERROMARK_PGO: "0",
      FERROMARK_NATIVE_OUTPUT_DIR: output,
      FERROMARK_NAPI_FEATURES: "plugin-test",
    },
  });
  const files = await readdir(output);
  const addon = files.find((file) => file.endsWith(".node"));
  assert.ok(addon);
  // Load the isolated binary directly: the published package remains untouched.
  run(process.execPath, [
    "--expose-gc",
    fileURLToPath(new URL("verify-native-plugins-worker.mjs", import.meta.url)),
    path.join(output, addon),
  ]);
} finally {
  await rm(output, { recursive: true, force: true });
}
