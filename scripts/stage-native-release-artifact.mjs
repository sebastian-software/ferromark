import { copyFileSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";

const platformId = process.env.PLATFORM_ID ?? "";
const binaryName = process.env.BINARY_NAME ?? "";
const sourceSha = (process.env.SOURCE_SHA ?? "").toLowerCase();
const runnerTemp = process.env.RUNNER_TEMP;
const platformPattern = /^[a-z0-9-]+$/;

if (!platformPattern.test(platformId)) throw new Error("Invalid platform ID: " + platformId);
if (binaryName !== "ferromark." + platformId + ".node") {
  throw new Error("Unexpected native binary name: " + binaryName);
}
if (!/^[0-9a-f]{40}$/.test(sourceSha)) throw new Error("SOURCE_SHA must be a full commit SHA");
if (!runnerTemp) throw new Error("RUNNER_TEMP is required");

const packageJson = JSON.parse(readFileSync(path.resolve("ferromark/package.json"), "utf8"));
if (!Object.hasOwn(packageJson.optionalDependencies, packageJson.name + "-" + platformId)) {
  throw new Error("Platform is not declared by the facade: " + platformId);
}

const checkout = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" });
if (checkout.error || checkout.status !== 0)
  throw new Error("Could not read the checked out source SHA");
if (checkout.stdout.trim().toLowerCase() !== sourceSha) {
  throw new Error("Native build checkout does not match the validated release SHA");
}

const binaryPath = path.resolve("ferromark/npm", platformId, binaryName);
const binary = statSync(binaryPath);
if (!binary.isFile() || binary.size === 0) {
  throw new Error("Invalid native binary: " + binaryPath);
}

const artifactDir = path.join(runnerTemp, "ferromark-native-" + platformId);
mkdirSync(artifactDir);
copyFileSync(binaryPath, path.join(artifactDir, binaryName));
writeFileSync(
  path.join(artifactDir, "ferromark-source-" + platformId + ".json"),
  JSON.stringify({ schemaVersion: 1, platformId, sourceSha }) + "\n",
);
console.log("Staged " + platformId + " from " + sourceSha);
