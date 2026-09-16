import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { releaseChannel } from "./release-channel.mjs";

export function publishArguments(archive, version) {
  return [
    "publish",
    archive,
    "--access",
    "public",
    "--provenance",
    "--tag",
    releaseChannel(version).tag,
  ];
}

// Validate every archive and existing version before the first registry mutation.
export async function preparePackages(directory, version, { request = fetch } = {}) {
  releaseChannel(version);
  const pkg = JSON.parse(readFileSync(new URL("../ferromark/package.json", import.meta.url)));
  assert.equal(pkg.version, version);
  assert.equal(pkg.private, false);
  const names = [...Object.keys(pkg.optionalDependencies).sort(), pkg.name];
  const expected = names.map((name) => `${name}-${version}.tgz`);
  assert.deepEqual(
    readdirSync(directory)
      .filter((name) => name.endsWith(".tgz"))
      .sort(),
    [...expected].sort(),
  );
  const packages = [];
  for (const [index, name] of names.entries()) {
    const archive = resolve(directory, expected[index]);
    packages.push(await inspectPackage({ name, archive, version, pkg, request }));
  }
  return packages;
}

async function inspectPackage({ name, archive, version, pkg, request }) {
  const manifestResult = spawnSync("tar", ["-xOzf", archive, "package/package.json"], {
    encoding: "utf8",
  });
  if (manifestResult.error) throw manifestResult.error;
  assert.equal(manifestResult.status, 0, `${name}: cannot read archive manifest`);
  const manifest = JSON.parse(manifestResult.stdout);
  assert.equal(manifest.name, name);
  assert.equal(manifest.version, version);
  assert.equal(manifest.private, false);
  if (name === pkg.name) assert.deepEqual(manifest.optionalDependencies, pkg.optionalDependencies);
  const response = await request(
    `https://registry.npmjs.org/${encodeURIComponent(name)}/${encodeURIComponent(version)}`,
    { signal: AbortSignal.timeout(30_000) },
  );
  if (response.ok) {
    const metadata = await response.json();
    const integrity = `sha512-${createHash("sha512").update(readFileSync(archive)).digest("base64")}`;
    assert.equal(
      metadata.dist?.integrity,
      integrity,
      `${name}: existing version differs from verified archive`,
    );
  } else {
    assert.equal(response.status, 404, `${name}: registry preflight failed`);
  }
  return { name, archive, exists: response.ok };
}

export async function publishPackages(directory, version, { execute = run, request = fetch } = {}) {
  const packages = await preparePackages(directory, version, { request });
  for (const { name, archive, exists } of packages) {
    execute(
      exists
        ? ["dist-tag", "add", `${name}@${version}`, releaseChannel(version).tag]
        : publishArguments(archive, version),
    );
  }
}

function run(args) {
  const result = spawnSync("npm", args, { stdio: "inherit", shell: process.platform === "win32" });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`npm ${args[0]} failed (${result.status})`);
}

if (process.argv[1] === import.meta.filename) {
  const directory = process.argv[2];
  const version = process.argv[3];
  const mode = process.argv[4];
  if (!directory || !version || (mode && mode !== "--check") || process.argv.length > 5) {
    throw new Error(
      "Usage: node scripts/publish-packages.mjs <archive-directory> <version> [--check]",
    );
  }
  if (mode === "--check") await preparePackages(resolve(directory), version);
  else await publishPackages(resolve(directory), version);
}
