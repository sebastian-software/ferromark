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

function versionUpdater(updaters, path) {
  return updaters.some(
    (updater) =>
      typeof updater === "object" &&
      updater !== null &&
      updater.type === "json" &&
      updater.jsonpath === "$.version" &&
      (updater.path === path ||
        (updater.glob === true &&
          path.startsWith(updater.path.split("*")[0]) &&
          path.endsWith(updater.path.split("*")[1]))),
  );
}

function validate({ package: packageJson, platforms, lockfile, config }) {
  const references = packageJson.optionalDependencies;
  const locked = lockfile.importers.ferromark.optionalDependencies;
  const updaters = config.packages["."]["extra-files"];
  if (Object.keys(references).sort().join() !== Object.keys(platforms).sort().join()) {
    failContract("native platform packages and optional dependencies differ");
  }
  if (!versionUpdater(updaters, "node/ferromark/package.json")) {
    failContract("release-please must update the npm facade version");
  }
  if (updaters.some((updater) => updater?.path?.endsWith("pnpm-lock.yaml"))) {
    failContract("the pnpm lockfile is generated state, not a release template target");
  }

  for (const [name, platform] of Object.entries(platforms)) {
    const version = packageJson.version;
    if (platform.version !== version) {
      failContract(`${name}: native package must match ${version}`);
    }
    // The facade carries no sidecar version at all: `workspace:*` resolves to
    // the sidecar's own version while packing, so a release bumps nine
    // `version` fields and nothing else — the lockfile included.
    if (references[name] !== "workspace:*") {
      failContract(`${name}: must be referenced with the workspace protocol`);
    }
    const entry = locked[name];
    if (entry?.specifier !== "workspace:*") {
      failContract(`${name}: pnpm lockfile specifier must be the workspace protocol`);
    }
    if (entry.version !== `link:npm/${name.replace(/^ferromark-/, "")}`) {
      failContract(`${name}: lockfile must resolve to its local workspace package`);
    }
    if (
      !versionUpdater(
        updaters,
        `node/ferromark/npm/${name.replace(/^ferromark-/, "")}/package.json`,
      )
    ) {
      failContract(`${name}: release-please must update its native package version`);
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

  it("rejects a lockfile specifier that pins a version", () => {
    assertRejected((copy) => {
      copy.lockfile.importers.ferromark.optionalDependencies[platformName].specifier = "0.0.0";
    });
  });

  it("rejects a mismatched native package version", () => {
    assertRejected((copy) => {
      copy.platforms[platformName].version = "0.0.0";
    });
  });

  it("rejects a sidecar reference that pins a version", () => {
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
        (updater) => updater?.glob !== true,
      );
    });
  });

  it("rejects a release updater that writes into the pnpm lockfile", () => {
    assertRejected((copy) => {
      copy.config.packages["."]["extra-files"].push({
        type: "yaml",
        path: "node/pnpm-lock.yaml",
        jsonpath: "$.importers.ferromark.optionalDependencies[*].specifier",
      });
    });
  });
});
