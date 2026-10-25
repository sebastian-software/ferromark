import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, readdir, rm, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const output = await mkdtemp(path.join(tmpdir(), "ferromark-ferrigura-"));
const root = fileURLToPath(new URL("../../../", import.meta.url));
const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8", ...options });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${command} failed:\n${result.stdout}\n${result.stderr}`);
};

try {
  for (const variant of ["combined", "diagrams"]) {
    const directory = path.join(output, variant);
    run(process.execPath, [fileURLToPath(new URL("build-native.mjs", import.meta.url))], {
      env: {
        ...process.env,
        FERROMARK_PGO: "0",
        FERROMARK_NATIVE_OUTPUT_DIR: directory,
        FERROMARK_NAPI_NO_DEFAULT_FEATURES: variant === "diagrams" ? "1" : "0",
        FERROMARK_NAPI_FEATURES: "ferrigura",
      },
    });
    const files = await readdir(directory);
    const addon = files.find((file) => file.endsWith(".node"));
    assert.ok(addon);
    run(process.execPath, [
      "--expose-gc",
      fileURLToPath(new URL("verify-ferrigura-worker.mjs", import.meta.url)),
      path.join(directory, addon),
      variant,
    ]);
    const { size } = await stat(path.join(directory, addon));
    process.stdout.write(`Ferrigura ${variant}: HTML/JSX checks passed; addon ${size} bytes\n`);
  }
} finally {
  await rm(output, { recursive: true, force: true });
}
