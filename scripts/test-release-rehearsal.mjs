import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { it } from "node:test";
import TOML from "@iarna/toml";
import { preserveExactTransformPin } from "./lib/exact-transform-pin.mjs";
import {
  maintenanceHistory,
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

it("configures one release component for the versioned Cargo workspace", () => {
  const config = JSON.parse(readReleaseFiles().get("release-please-config.json"));
  assert.equal(config["release-type"], "rust");
  // The published tag is `v2.0.0`, so the component must not enter it.
  assert.equal(config["include-component-in-tag"], false);
  assert.deepEqual(Object.keys(config.packages), ["."]);
  const pkg = config.packages["."];
  assert.equal(pkg.component, "ferromark");
  // The candidate series ended with 2.0.0; the stable series uses the default
  // versioning strategy, so none of the prerelease keys may come back.
  assert.equal(pkg.versioning, undefined, "the stable series uses default versioning");
  assert.equal(pkg["prerelease-type"], undefined, "no candidate suffix is configured");
  assert.equal(pkg.prerelease, undefined, "no prerelease flag is configured");
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

it("restores the exact transform dependency pin after the Rust updater", () => {
  const files = readReleaseFiles();
  const rootVersion = TOML.parse(files.get("Cargo.toml")).package.version;
  const transformManifest = files.get("transforms/Cargo.toml");
  const updaterOutput = transformManifest.replace(
    `version = "=${rootVersion}"`,
    `version = "${rootVersion}"`,
  );
  assert.notEqual(updaterOutput, transformManifest, "fixture models the Rust updater's caret pin");

  const restored = preserveExactTransformPin(files.get("Cargo.toml"), updaterOutput);
  assert.equal(restored.changed, true);
  assert.equal(TOML.parse(restored.content).dependencies.ferromark.version, `=${rootVersion}`);
  assert.equal(preserveExactTransformPin(files.get("Cargo.toml"), restored.content).changed, false);
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

it("proposes the next patch release from the commit range alone", async () => {
  // What the publish workflow opens after the stable release: the released
  // version is 2.0.0 and no commit carries `Release-As`. Seed that version
  // explicitly instead of reading the checkout's own, so the case also holds on
  // a release pull request branch, where the files already carry the next one.
  const files = (
    await proposeRelease(readReleaseFiles(), "chore: seed release\n\nRelease-As: 2.0.0")
  ).files;
  assert.equal(JSON.parse(files.get(".release-please-manifest.json"))["."], "2.0.0");
  const proposed = await proposeRelease(files, maintenanceHistory);
  validateRelease(proposed.files, "2.0.1");
  assert.ok(proposed.title.includes("2.0.1"), proposed.title);
  assert.ok(!proposed.title.includes("-rc"), "The stable series carries no candidate suffix");
  // The generated notes head the authored 2.0.0-rc.1 section already in Git.
  const changelog = proposed.files.get("CHANGELOG.md");
  assert.ok(changelog.startsWith("# Changelog\n\n## [2.0.1]"), changelog.slice(0, 120));
  assert.ok(changelog.includes("### Bug Fixes"));
  assert.ok(changelog.includes("## 2.0.0-rc.1"), "Authored notes must survive");
  assert.ok(proposed.paths.includes("CHANGELOG.md"));
  assert.ok(proposed.paths.includes(".release-please-manifest.json"));
  // Merging the release pull request tags this version and publishes it.
  assert.ok(proposed.paths.includes("Cargo.toml"));
  assert.ok(proposed.paths.includes("Cargo.lock"));
  assert.ok(proposed.paths.includes("node/native/Cargo.toml"));
  assert.ok(proposed.paths.includes("transforms/Cargo.toml"));
  for (const target of ["darwin-arm64", "win32-arm64-msvc"]) {
    assert.ok(proposed.paths.includes(`node/ferromark/npm/${target}/package.json`));
  }
});

it("requires an explicit Release-As to leave a candidate series", async () => {
  // With the default strategy a candidate series drifts rather than ending: a
  // bare `feat:` on `2.0.0-rc.2` keeps the suffix and proposes `2.1.0-rc.2`.
  // That is why the switch and the `Release-As: 2.0.0` footer belong together.
  const files = (await proposeRelease(readReleaseFiles(), "chore: seed\n\nRelease-As: 2.0.0-rc.2"))
    .files;
  const drifted = await proposeRelease(files, "feat: a candidate addition");
  assert.ok(drifted.title.includes("2.1.0-rc.2"), drifted.title);
  const promoted = await proposeRelease(files, "feat: finalize v2\n\nRelease-As: 2.0.0");
  validateRelease(promoted.files, "2.0.0");
});

it("proposes ordinary stable versions once the candidate series has ended", async () => {
  const files = (await proposeRelease(readReleaseFiles(), "chore: seed\n\nRelease-As: 2.0.0"))
    .files;
  validateRelease(files, "2.0.0");
  for (const [expected, message] of [
    ["2.0.1", "fix: ordinary maintenance correction"],
    ["2.1.0", "feat: an added capability"],
    ["3.0.0", "feat!: a removed option"],
  ]) {
    const proposed = await proposeRelease(files, message);
    assert.ok(proposed.title.includes(expected), proposed.title);
    assert.ok(!proposed.title.includes("-rc"), proposed.title);
    validateRelease(proposed.files, expected);
    if (message.startsWith("feat!")) {
      assert.ok(proposed.files.get("CHANGELOG.md").includes("### ⚠ BREAKING CHANGES"));
    }
  }
  // Starting a new candidate series again is deliberate, never automatic.
  const candidate = await proposeRelease(files, "feat: prepare v3\n\nRelease-As: 3.0.0-rc.1");
  validateRelease(candidate.files, "3.0.0-rc.1");
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
