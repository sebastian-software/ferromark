import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { preserveExactTransformPin } from "./lib/exact-transform-pin.mjs";
import { preserveRegistryLockfile } from "./lib/release-lockfile.mjs";

const rootCargoToml = readFileSync(resolve("Cargo.toml"), "utf8");
const transformsManifest = resolve("transforms/Cargo.toml");
const transformsCargoToml = readFileSync(transformsManifest, "utf8");
const result = preserveExactTransformPin(rootCargoToml, transformsCargoToml);

if (result.changed) writeFileSync(transformsManifest, result.content);

// The workflow supplies the lockfile from this release branch's base commit.
if (process.argv[2]) {
  const lockfile = resolve("Cargo.lock");
  const current = readFileSync(lockfile, "utf8");
  const corrected = preserveRegistryLockfile(readFileSync(process.argv[2], "utf8"), current);
  if (corrected !== current) writeFileSync(lockfile, corrected);
}
