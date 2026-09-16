import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { it } from "node:test";
import TOML from "@iarna/toml";
import {
  candidateHistory,
  proposeRelease,
  readReleaseFiles,
  root,
  validateRelease,
} from "./lib/release-rehearsal.mjs";

async function developmentBaseline() {
  return (
    await proposeRelease(
      readReleaseFiles(),
      "chore: seed version rehearsal\n\nRelease-As: 2.0.0-dev.0",
    )
  ).files;
}

it("configures the native rust strategy with one root component", () => {
  const config = JSON.parse(readReleaseFiles().get("release-please-config.json"));
  assert.equal(config["release-type"], "rust");
  // The published tag is `v2.0.0-rc.1`, so the component must not enter it.
  assert.equal(config["include-component-in-tag"], false);
  assert.deepEqual(Object.keys(config.packages), ["."]);
  const pkg = config.packages["."];
  assert.equal(pkg.component, "ferromark");
  assert.equal(pkg.versioning, "prerelease");
  assert.equal(pkg["prerelease-type"], "rc");
  assert.equal(pkg.prerelease, true);
  assert.equal(pkg["version-file"], undefined, "version.txt belongs to the simple strategy");
  for (const entry of pkg["extra-files"]) {
    const path = typeof entry === "string" ? entry : entry.path;
    assert.ok(
      !/Cargo\.(toml|lock)$/.test(path),
      `${path}: the rust strategy updates Cargo files natively`,
    );
    assert.ok(!/lock\.yaml$/.test(path), `${path}: a lockfile is generated state`);
  }
  assert.ok(!existsSync(resolve(root, "version.txt")), "version.txt must not come back");
});

it("builds coordinated RC, subsequent RC, stable, and patch release PRs with the real updater", async () => {
  let files = await developmentBaseline();
  const external = TOML.parse(files.get("Cargo.lock")).package.filter((pkg) => pkg.source);
  for (const [version, message] of [
    ["2.0.0-rc.1", "feat!: prepare v2\n\nRelease-As: 2.0.0-rc.1"],
    ["2.0.0-rc.2", "fix: candidate correction\n\nRelease-As: 2.0.0-rc.2"],
    ["2.0.0", "feat: finalize v2\n\nRelease-As: 2.0.0"],
    ["2.0.1", "fix: ordinary maintenance correction\n\nRelease-As: 2.0.1"],
  ]) {
    const proposed = await proposeRelease(files, message);
    validateRelease(proposed.files, version);
    assert.ok(proposed.title.includes(version));
    assert.ok(proposed.body.includes(version));
    assert.ok(proposed.files.get("CHANGELOG.md").includes(version));
    assert.deepEqual(
      TOML.parse(proposed.files.get("Cargo.lock")).package.filter((pkg) => pkg.source),
      external,
      "External dependencies must not change",
    );
    // A version bump touches versions, never publication flags or the lockfile
    // whose only version information is a `workspace:*` reference.
    assert.equal(
      proposed.files.get("node/pnpm-lock.yaml"),
      files.get("node/pnpm-lock.yaml"),
      "The pnpm lockfile is not a release template target",
    );
    for (const [file, content] of files) {
      if (file.endsWith("package.json"))
        assert.equal(
          JSON.parse(proposed.files.get(file)).private,
          JSON.parse(content).private,
          "A version bump must not enable publishing",
        );
    }
    assert.deepEqual(
      TOML.parse(proposed.files.get("Cargo.toml")).workspace.package.publish,
      TOML.parse(files.get("Cargo.toml")).workspace.package.publish,
    );
    files = proposed.files;
  }
});

it("proposes the next release candidate from the commit range alone", async () => {
  // What the publish workflow opens after the first candidate: the released
  // version is 2.0.0-rc.1 and no commit carries `Release-As`. Seed that version
  // explicitly instead of reading the checkout's own, so the case also holds on
  // a release pull request branch, where the files already carry the next one.
  const files = (
    await proposeRelease(readReleaseFiles(), "chore: seed candidate\n\nRelease-As: 2.0.0-rc.1")
  ).files;
  assert.equal(JSON.parse(files.get(".release-please-manifest.json"))["."], "2.0.0-rc.1");
  const proposed = await proposeRelease(files, candidateHistory);
  validateRelease(proposed.files, "2.0.0-rc.2");
  assert.ok(proposed.title.includes("2.0.0-rc.2"), proposed.title);
  assert.ok(!proposed.title.includes("3.0.0"), "A breaking change must not leave the RC series");
  // The generated notes head the authored 2.0.0-rc.1 section already in Git.
  const changelog = proposed.files.get("CHANGELOG.md");
  assert.ok(changelog.startsWith("# Changelog\n\n## [2.0.0-rc.2]"), changelog.slice(0, 120));
  assert.ok(changelog.includes("### ⚠ BREAKING CHANGES"));
  assert.ok(changelog.includes("## 2.0.0-rc.1"), "Authored notes must survive");
  assert.ok(proposed.paths.includes("CHANGELOG.md"));
  assert.ok(proposed.paths.includes(".release-please-manifest.json"));
  // Merging the release pull request tags this version and publishes it.
  assert.ok(proposed.paths.includes("Cargo.toml"));
  assert.ok(proposed.paths.includes("Cargo.lock"));
  assert.ok(proposed.paths.includes("node/native/Cargo.toml"));
  for (const target of ["darwin-arm64", "win32-arm64-msvc"]) {
    assert.ok(proposed.paths.includes(`node/ferromark/npm/${target}/package.json`));
  }
});

it("requires an explicit Release-As once the version leaves the candidate series", async () => {
  // The prerelease strategy never proposes a bare stable version, so the
  // `Release-As` footer stays mandatory for the stable and patch transitions.
  const files = (await proposeRelease(readReleaseFiles(), "chore: seed\n\nRelease-As: 2.0.0"))
    .files;
  validateRelease(files, "2.0.0");
  const drifted = await proposeRelease(files, "fix: ordinary maintenance correction");
  assert.ok(drifted.title.includes("2.0.1-rc"), drifted.title);
  const pinned = await proposeRelease(
    files,
    "fix: ordinary maintenance correction\n\nRelease-As: 2.0.1",
  );
  validateRelease(pinned.files, "2.0.1");
});

for (const missing of [
  "node/ferromark/package.json",
  "node/ferromark/npm/darwin-arm64/package.json",
  "README.md",
]) {
  it(`detects missing release updates for ${missing}`, async () => {
    const files = await developmentBaseline();
    const config = JSON.parse(files.get("release-please-config.json"));
    config.packages["."]["extra-files"] = config.packages["."]["extra-files"].filter((entry) => {
      const path = typeof entry === "string" ? entry : entry.path;
      return !(path === missing || (entry.glob && missing.startsWith(path.split("*")[0])));
    });
    files.set("release-please-config.json", JSON.stringify(config));
    const proposed = await proposeRelease(files, "feat!: prepare v2\n\nRelease-As: 2.0.0-rc.1");
    assert.throws(() => validateRelease(proposed.files, "2.0.0-rc.1"), assert.AssertionError);
  });
}

it("rejects a member version the strategy cannot write", async () => {
  // Release Please cannot replace `version.workspace = true` with a concrete
  // version, which is why no package inherits its version any more. Its Cargo
  // updater refuses the manifest outright rather than leaving it stale.
  const files = await developmentBaseline();
  files.set(
    "node/native/Cargo.toml",
    files.get("node/native/Cargo.toml").replace(/^version = "[^"]+"$/m, "version.workspace = true"),
  );
  await assert.rejects(
    proposeRelease(files, "feat!: prepare v2\n\nRelease-As: 2.0.0-rc.1"),
    /package\.version is not tagged/,
  );
});

it("detects a published internal requirement the strategy skips", async () => {
  // A path dependency without an explicit `version` is deliberately skipped, so
  // the requirement would keep pointing at the previous release.
  const files = await developmentBaseline();
  files.set(
    "node/native/Cargo.toml",
    files
      .get("node/native/Cargo.toml")
      .replace(/^ferromark = \{[^}]*\}$/m, 'ferromark = { path = "../.." }'),
  );
  const proposed = await proposeRelease(files, "feat!: prepare v2\n\nRelease-As: 2.0.0-rc.1");
  const requirement = TOML.parse(proposed.files.get("node/native/Cargo.toml")).dependencies
    .ferromark;
  assert.equal(requirement.version, undefined, "The strategy leaves it without a version");
  assert.throws(() => validateRelease(proposed.files, "2.0.0-rc.1"), assert.AssertionError);
});
