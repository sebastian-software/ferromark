import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

export const repositoryRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);

export function repositoryPath(...segments) {
  return path.join(repositoryRoot, ...segments);
}

export function readRepositoryFile(...segments) {
  return readFileSync(repositoryPath(...segments), "utf8");
}

// YAML 1.1 merge keys keep workflow fixtures that reuse an anchored step
// readable; GitHub itself rejects them, so the scanners must still see through.
export function parseYaml(source) {
  return parse(source, { merge: true });
}

export function readYaml(...segments) {
  return parseYaml(readRepositoryFile(...segments));
}

export function deepCopy(value) {
  return structuredClone(value);
}

export function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export class ContractError extends Error {
  constructor(message) {
    super(message);
    this.name = "ContractError";
  }
}
