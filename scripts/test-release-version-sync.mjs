import assert from "node:assert/strict";
import { readdirSync } from "node:fs";
import { describe, it } from "node:test";

import {
  ContractError,
  deepCopy,
  parseYaml,
  readRepositoryFile,
  repositoryPath,
} from "./lib/contracts.mjs";

function failContract(message) {
  throw new ContractError(message);
}

function validate({ package: packageJson, platforms, lockfile, config }) {
  const pins = packageJson.optionalDependencies;
  const locked = lockfile.importers.ferromark.optionalDependencies;
  const updaters = config.packages["."]["extra-files"];
  if (Object.keys(pins).sort().join() !== Object.keys(platforms).sort().join()) {
    failContract("native platform packages and optional dependencies differ");
  }

  for (const [name, platform] of Object.entries(platforms)) {
    const version = packageJson.version;
    if (platform.version !== version || pins[name] !== version) {
      failContract(`${name}: native package and dependency must match ${version}`);
    }
    const entry = locked[name];
    if (entry?.specifier !== version) {
      failContract(`${name}: pnpm lockfile specifier must match ${version}`);
    }
    if (entry.version !== `link:npm/${name.replace(/^ferromark-/, "")}`) {
      failContract(`${name}: lockfile must resolve to its local workspace package`);
    }
    const expected = {
      type: "yaml",
      path: "node/pnpm-lock.yaml",
      jsonpath: `$.importers.ferromark.optionalDependencies['${name}'].specifier`,
    };
    const updated = updaters.some(
      (updater) =>
        typeof updater === "object" &&
        updater !== null &&
        updater.type === expected.type &&
        updater.path === expected.path &&
        updater.jsonpath === expected.jsonpath,
    );
    if (!updated) {
      failContract(`${name}: release-please must update its pnpm lockfile specifier`);
    }
  }
}

function readInputs() {
  const packageJson = JSON.parse(readRepositoryFile("node", "ferromark", "package.json"));
  const platforms = {};
  for (const entry of readdirSync(repositoryPath("node", "ferromark", "npm")).sort()) {
    const platform = JSON.parse(
      readRepositoryFile("node", "ferromark", "npm", entry, "package.json"),
    );
    platforms[platform.name] = platform;
  }
  return {
    package: packageJson,
    platforms,
    lockfile: parseYaml(readRepositoryFile("node", "pnpm-lock.yaml")),
    config: JSON.parse(readRepositoryFile("release-please-config.json")),
  };
}

function assertRejected(mutate) {
  const inputs = deepCopy(readInputs());
  mutate(inputs);
  assert.throws(() => validate(inputs), ContractError);
}

const inputs = readInputs();
const platformName = Object.keys(inputs.platforms).sort()[0];

describe("release version sync", () => {
  it("keeps every published version in step", () => {
    validate(inputs);
    assert.equal(Object.keys(inputs.platforms).length, 8);
  });

  it("rejects an outdated lockfile specifier", () => {
    assertRejected((copy) => {
      copy.lockfile.importers.ferromark.optionalDependencies[platformName].specifier = "0.0.0";
    });
  });

  it("rejects a mismatched native package version", () => {
    assertRejected((copy) => {
      copy.platforms[platformName].version = "0.0.0";
    });
  });

  it("rejects a mismatched dependency pin", () => {
    assertRejected((copy) => {
      copy.package.optionalDependencies[platformName] = "0.0.0";
    });
  });

  it("rejects a platform package resolved from the registry", () => {
    assertRejected((copy) => {
      copy.lockfile.importers.ferromark.optionalDependencies[platformName].version = "0.0.0";
    });
  });

  it("rejects a missing release updater", () => {
    assertRejected((copy) => {
      copy.config.packages["."]["extra-files"] = copy.config.packages["."]["extra-files"].filter(
        (updater) => !(typeof updater === "object" && updater !== null && updater.type === "yaml"),
      );
    });
  });
});
