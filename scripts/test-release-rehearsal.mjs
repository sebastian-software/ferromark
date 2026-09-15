import assert from "node:assert/strict";
import { it } from "node:test";
import TOML from "@iarna/toml";
import { proposeRelease, readReleaseFiles, validateRelease } from "./lib/release-rehearsal.mjs";

async function developmentBaseline() {
  return (
    await proposeRelease(
      readReleaseFiles(),
      "chore: seed version rehearsal\n\nRelease-As: 2.0.0-dev.0",
    )
  ).files;
}

it("builds coordinated RC, subsequent RC, stable, and patch release PRs with the real updater", async () => {
  let files = await developmentBaseline();
  const external = TOML.parse(files.get("Cargo.lock")).package.filter((pkg) => pkg.source);
  for (const [version, message] of [
    ["2.0.0-rc.1", "feat!: prepare v2\n\nRelease-As: 2.0.0-rc.1"],
    ["2.0.0-rc.2", "fix: candidate correction\n\nRelease-As: 2.0.0-rc.2"],
    ["2.0.0", "feat: finalize v2\n\nRelease-As: 2.0.0"],
    ["2.0.1", "fix: ordinary maintenance correction"],
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
    for (const [file, content] of files) {
      if (file.endsWith("/Cargo.toml"))
        assert.equal(proposed.files.get(file), content, "Member inheritance stays intact");
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

for (const missing of [
  "Cargo.toml",
  "Cargo.lock",
  "node/pnpm-lock.yaml",
  "node/ferromark/package.json",
]) {
  it(`detects missing release updates for ${missing}`, async () => {
    const files = await developmentBaseline();
    const config = JSON.parse(files.get("release-please-config.json"));
    config.packages["."]["extra-files"] = config.packages["."]["extra-files"].filter(
      (entry) => (typeof entry === "string" ? entry : entry.path) !== missing,
    );
    files.set("release-please-config.json", JSON.stringify(config));
    const proposed = await proposeRelease(files, "feat!: prepare v2\n\nRelease-As: 2.0.0-rc.1");
    assert.throws(() => validateRelease(proposed.files, "2.0.0-rc.1"), assert.AssertionError);
  });
}
