import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { preserveExactTransformPin } from "./lib/exact-transform-pin.mjs";

const rootCargoToml = readFileSync(resolve("Cargo.toml"), "utf8");
const transformsManifest = resolve("transforms/Cargo.toml");
const transformsCargoToml = readFileSync(transformsManifest, "utf8");
const result = preserveExactTransformPin(rootCargoToml, transformsCargoToml);

if (result.changed) writeFileSync(transformsManifest, result.content);
