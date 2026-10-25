import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import TOML from "@iarna/toml";
import { CargoLock } from "release-please/build/src/updaters/rust/cargo-lock.js";
import { preserveRegistryLockfile } from "./lib/release-lockfile.mjs";
import { proposeRelease, readReleaseFiles } from "./lib/release-rehearsal.mjs";

test("release bumps keep the registry identity when a workspace crate has the same name/version", async () => {
  const files = readReleaseFiles();
  const original = TOML.parse(files.get("Cargo.lock")).package;
  const registries = original.filter((pkg) => pkg.source);
  assert.ok(registries.some((pkg) => pkg.name === "ferromark"));
  const result = await proposeRelease(files, "feat: plugin release\n\nRelease-As: 3.7.0");
  const packages = TOML.parse(result.files.get("Cargo.lock")).package;
  assert.deepEqual(
    packages.filter((pkg) => pkg.source),
    registries,
  );
  assert.equal(packages.find((pkg) => pkg.name === "ferromark" && !pkg.source).version, "3.7.0");
  const native = packages.find((pkg) => pkg.name === "ferromark-node");
  assert.ok(native.dependencies.includes("ferromark 3.7.0"));
  assert.ok(!native.dependencies.includes("ferromark 3.6.0"));
  assert.equal(
    preserveRegistryLockfile(result.files.get("Cargo.lock"), result.files.get("Cargo.lock")),
    result.files.get("Cargo.lock"),
  );
});

test("the production correction requires the release branch base lockfile", () => {
  const workflow = readFileSync(
    new URL("../.github/workflows/publish.yml", import.meta.url),
    "utf8",
  );
  assert.match(workflow, /git merge-base HEAD origin\/main/);
  assert.match(
    workflow,
    /preserve-exact-transform-pin\.mjs "\$RUNNER_TEMP\/release-base-Cargo.lock"/,
  );
  assert.match(workflow, /git add transforms\/Cargo.toml Cargo.lock/);
});

test("the production CLI repairs the real updater's registry collision and exact transform pin", async () => {
  const files = readReleaseFiles();
  const proposal = await proposeRelease(files, "feat: plugin release\n\nRelease-As: 3.7.0");
  const broken = new CargoLock(new Map([["ferromark", "3.7.0"]])).updateContent(
    proposal.files.get("Cargo.lock"),
  );
  assert.notEqual(broken, proposal.files.get("Cargo.lock"));
  const directory = mkdtempSync(path.join(tmpdir(), "ferromark-release-correction-"));
  try {
    mkdirSync(path.join(directory, "transforms"));
    writeFileSync(path.join(directory, "Cargo.toml"), proposal.files.get("Cargo.toml"));
    writeFileSync(path.join(directory, "Cargo.lock"), broken);
    writeFileSync(path.join(directory, "base.lock"), files.get("Cargo.lock"));
    writeFileSync(
      path.join(directory, "transforms/Cargo.toml"),
      proposal.files
        .get("transforms/Cargo.toml")
        .replace('version = "=3.7.0"', 'version = "3.7.0"'),
    );
    const result = spawnSync(
      process.execPath,
      [
        fileURLToPath(new URL("./preserve-exact-transform-pin.mjs", import.meta.url)),
        path.join(directory, "base.lock"),
      ],
      { cwd: directory, encoding: "utf8" },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.equal(
      readFileSync(path.join(directory, "Cargo.lock"), "utf8"),
      proposal.files.get("Cargo.lock"),
    );
    assert.equal(
      readFileSync(path.join(directory, "transforms/Cargo.toml"), "utf8"),
      proposal.files.get("transforms/Cargo.toml"),
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
