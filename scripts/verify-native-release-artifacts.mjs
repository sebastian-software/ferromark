import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

const artifactDir = path.resolve(process.env.ARTIFACT_DIRECTORY ?? "");
const destinationDir = path.resolve(process.env.DESTINATION_DIRECTORY ?? "");
const sourceSha = (process.env.SOURCE_SHA ?? "").toLowerCase();
if (!/^[0-9a-f]{40}$/.test(sourceSha)) throw new Error("SOURCE_SHA must be a full commit SHA");
if (!process.env.ARTIFACT_DIRECTORY) throw new Error("ARTIFACT_DIRECTORY is required");
if (!process.env.DESTINATION_DIRECTORY) throw new Error("DESTINATION_DIRECTORY is required");

const packageJson = JSON.parse(readFileSync(path.resolve("ferromark/package.json"), "utf8"));
const prefix = packageJson.name + "-";
const expectedPlatforms = Object.keys(packageJson.optionalDependencies).map((name) => {
  if (!name.startsWith(prefix)) throw new Error("Unexpected platform package: " + name);
  return name.slice(prefix.length);
});
const expectedMetadata = expectedPlatforms
  .map((platformId) => "ferromark-source-" + platformId + ".json")
  .sort();
const actualFiles = readdirSync(artifactDir);
const actualMetadata = actualFiles.filter((name) => name.startsWith("ferromark-source-")).sort();
assert.deepEqual(
  actualMetadata,
  expectedMetadata,
  "native provenance files must match the release matrix",
);

const expectedBinaries = expectedPlatforms.map((id) => "ferromark." + id + ".node").sort();
const actualBinaries = actualFiles.filter((name) => name.endsWith(".node")).sort();
assert.deepEqual(actualBinaries, expectedBinaries, "native binaries must match the release matrix");

for (const platformId of expectedPlatforms) {
  const metadataName = "ferromark-source-" + platformId + ".json";
  const metadata = JSON.parse(readFileSync(path.join(artifactDir, metadataName), "utf8"));
  assert.deepEqual(metadata, { schemaVersion: 1, platformId, sourceSha }, metadataName);

  const binaryPath = path.join(artifactDir, "ferromark." + platformId + ".node");
  const binary = statSync(binaryPath);
  assert.ok(binary.isFile() && binary.size > 0, "invalid native binary: " + binaryPath);
}

mkdirSync(destinationDir, { recursive: true });
assert.deepEqual(readdirSync(destinationDir), [], "native assembly directory must start empty");
for (const binaryName of expectedBinaries) {
  copyFileSync(path.join(artifactDir, binaryName), path.join(destinationDir, binaryName));
}

console.log("Verified " + expectedPlatforms.length + " native artifacts from " + sourceSha);
