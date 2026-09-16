import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";

import { releaseChannel } from "./release-channel.mjs";
const workspace = path.resolve(import.meta.dirname, "..");
const packageDir = path.join(workspace, "ferromark");
const packageJson = JSON.parse(await readFile(path.join(packageDir, "package.json"), "utf8"));
const prefix = `${packageJson.name}-`;

for (const [name, version] of Object.entries(packageJson.optionalDependencies).sort()) {
  if (!name.startsWith(prefix) || version !== packageJson.version) {
    throw new Error(`Invalid platform dependency: ${name}@${version}`);
  }

  const target = name.slice(prefix.length);
  console.log(`Publishing ${name}@${version}`);
  const result = spawnSync(
    "npm",
    [
      "publish",
      "--access",
      "public",
      "--provenance",
      "--tag",
      releaseChannel(packageJson.version).tag,
    ],
    {
      cwd: path.join(packageDir, "npm", target),
      shell: process.platform === "win32",
      stdio: "inherit",
    },
  );
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
