import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFile, cp, mkdtemp, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageDir = fileURLToPath(new URL("../", import.meta.url));
const output = await mkdtemp(path.join(tmpdir(), "ferromark-ferriki-"));
const run = (args, options = {}) => {
  const result = spawnSync(process.execPath, args, { encoding: "utf8", ...options });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  process.stdout.write(result.stdout);
};

try {
  run([path.join(packageDir, "scripts/build-native.mjs")], {
    cwd: packageDir,
    env: {
      ...process.env,
      FERROMARK_PGO: "0",
      FERROMARK_NATIVE_OUTPUT_DIR: output,
      FERROMARK_NAPI_FEATURES: "ferriki",
    },
  });
  // Exercise the public facade against the custom binary without replacing
  // the engine-free addon or declarations used by package and release checks.
  for (const file of ["index.mjs", "native-target.mjs"]) {
    await copyFile(path.join(packageDir, file), path.join(output, file));
  }
  await cp(path.join(packageDir, "test/plugins"), path.join(output, "test/plugins"), {
    recursive: true,
  });
  const entries = await readdir(path.join(output, "test/plugins"));
  const files = entries
    .filter((file) => file.endsWith(".test.mjs"))
    .map((file) => path.join(output, "test/plugins", file));
  run(["--test", ...files], { cwd: output, env: { ...process.env, FERRIKI_REMOTE: "0" } });
} finally {
  await rm(output, { recursive: true, force: true });
}
