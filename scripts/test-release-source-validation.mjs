import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { verifyGitHubRelease } from "./verify-github-release.mjs";

const resolver = new URL("./resolve-release-source.mjs", import.meta.url);
const stageNative = new URL("./stage-native-release-artifact.mjs", import.meta.url);
const verifyNative = new URL("./verify-native-release-artifacts.mjs", import.meta.url);
const root = path.resolve(import.meta.dirname, "..");

function command(cwd, executable, args) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, GIT_TERMINAL_PROMPT: "0" },
  });
  assert.equal(result.status, 0, result.stderr || executable + " " + args.join(" "));
  return result.stdout.trim();
}

function fixture(t) {
  const directory = mkdtempSync(path.join(tmpdir(), "ferromark-release-source-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const origin = path.join(directory, "origin.git");
  const checkout = path.join(directory, "checkout");
  command(directory, "git", ["init", "--bare", origin]);
  command(directory, "git", ["init", "--initial-branch=main", checkout]);
  command(checkout, "git", ["config", "user.name", "Release Fixture"]);
  command(checkout, "git", ["config", "user.email", "release-fixture@example.invalid"]);
  command(checkout, "git", ["config", "commit.gpgsign", "false"]);
  writeFileSync(path.join(checkout, "release.txt"), "approved release source\n");
  command(checkout, "git", ["add", "release.txt"]);
  command(checkout, "git", ["commit", "-m", "release source"]);
  command(checkout, "git", ["remote", "add", "origin", origin]);
  command(checkout, "git", ["push", "-u", "origin", "main"]);
  command(checkout, "git", ["tag", "-a", "v1.2.3", "-m", "v1.2.3"]);
  command(checkout, "git", ["push", "origin", "refs/tags/v1.2.3"]);
  writeFileSync(path.join(checkout, "release.txt"), "main advanced after release\n");
  command(checkout, "git", ["add", "release.txt"]);
  command(checkout, "git", ["commit", "-m", "advance main after release"]);
  command(checkout, "git", ["push", "origin", "main"]);
  return { directory, checkout, origin };
}

function resolve(
  fixtureValue,
  tag,
  { eventName = "workflow_dispatch", ref = "refs/heads/main" } = {},
) {
  const output = path.join(fixtureValue.directory, "github-output-" + Math.random());
  const result = spawnSync(process.execPath, [resolver.pathname], {
    cwd: fixtureValue.checkout,
    encoding: "utf8",
    env: {
      ...process.env,
      GITHUB_EVENT_NAME: eventName,
      GITHUB_REF: ref,
      GITHUB_OUTPUT: output,
      RELEASE_TAG: tag,
      GIT_TERMINAL_PROMPT: "0",
    },
  });
  return { result, output };
}

test("resolves a release tag to one full commit for dispatch and push after main advances", (t) => {
  const repo = fixture(t);
  const expectedSha = command(repo.checkout, "git", ["rev-parse", "refs/tags/v1.2.3^{commit}"]);
  for (const eventName of ["workflow_dispatch", "push"]) {
    const { result, output } = resolve(repo, "v1.2.3", { eventName });
    assert.equal(result.status, 0, eventName + ": " + result.stderr);
    assert.equal(
      readFileSync(output, "utf8"),
      "tag_name=v1.2.3\nsource_sha=" + expectedSha + "\n",
      eventName,
    );
  }
});

test("rejects non-release input and dispatches that do not run from main", (t) => {
  const repo = fixture(t);
  for (const tag of ["main", "refs/tags/v1.2.3", "a".repeat(40), "v1.2.3-dev.1", "v1.2.3-rc.01"]) {
    const { result } = resolve(repo, tag);
    assert.notEqual(result.status, 0, "accepted invalid tag " + tag);
  }
  const { result } = resolve(repo, "v1.2.3", { ref: "refs/heads/release" });
  assert.notEqual(result.status, 0, "accepted a dispatch from a non-main ref");
  const unsupportedEvent = resolve(repo, "v1.2.3", { eventName: "pull_request" });
  assert.notEqual(unsupportedEvent.result.status, 0, "accepted an unsupported event");
});

test("rejects missing and off-main tags", (t) => {
  const repo = fixture(t);
  assert.notEqual(resolve(repo, "v9.8.7").result.status, 0, "accepted a missing tag");

  command(repo.checkout, "git", ["switch", "--orphan", "unrelated-release"]);
  writeFileSync(path.join(repo.checkout, "release.txt"), "unrelated source\n");
  command(repo.checkout, "git", ["add", "release.txt"]);
  command(repo.checkout, "git", ["commit", "-m", "unrelated release source"]);
  command(repo.checkout, "git", ["tag", "-a", "v9.8.7", "-m", "v9.8.7"]);
  command(repo.checkout, "git", ["push", "origin", "refs/tags/v9.8.7"]);
  command(repo.checkout, "git", ["switch", "main"]);

  const { result } = resolve(repo, "v9.8.7");
  assert.notEqual(result.status, 0, "accepted a tag outside main ancestry");
});

test("stages native artifacts only from the validated checkout SHA", (t) => {
  const repo = fixture(t);
  const sourceSha = command(repo.checkout, "git", ["rev-parse", "HEAD"]);
  const platformId = "test-platform";
  const binaryName = "ferromark." + platformId + ".node";
  const packageDir = path.join(repo.checkout, "ferromark");
  const platformDir = path.join(packageDir, "npm", platformId);
  const runnerTemp = path.join(repo.directory, "runner-temp");
  mkdirSync(platformDir, { recursive: true });
  mkdirSync(runnerTemp);
  writeFileSync(
    path.join(packageDir, "package.json"),
    JSON.stringify({
      name: "ferromark",
      optionalDependencies: { ["ferromark-" + platformId]: "workspace:*" },
    }),
  );
  writeFileSync(path.join(platformDir, binaryName), "native fixture");

  const result = spawnSync(process.execPath, [stageNative.pathname], {
    cwd: repo.checkout,
    encoding: "utf8",
    env: {
      ...process.env,
      PLATFORM_ID: platformId,
      BINARY_NAME: binaryName,
      SOURCE_SHA: sourceSha,
      RUNNER_TEMP: runnerTemp,
    },
  });
  assert.equal(result.status, 0, result.stderr);

  const artifactDir = path.join(runnerTemp, "ferromark-native-" + platformId);
  assert.equal(readFileSync(path.join(artifactDir, binaryName), "utf8"), "native fixture");
  assert.deepEqual(
    JSON.parse(
      readFileSync(path.join(artifactDir, "ferromark-source-" + platformId + ".json"), "utf8"),
    ),
    { schemaVersion: 1, platformId, sourceSha },
  );

  const wrongSha = spawnSync(process.execPath, [stageNative.pathname], {
    cwd: repo.checkout,
    encoding: "utf8",
    env: {
      ...process.env,
      PLATFORM_ID: platformId,
      BINARY_NAME: binaryName,
      SOURCE_SHA: "f".repeat(40),
      RUNNER_TEMP: runnerTemp,
    },
  });
  assert.notEqual(wrongSha.status, 0, "accepted a build checkout with the wrong source SHA");
});

test("rejects mixed-SHA native artifacts before creating the promotion directory", (t) => {
  const directory = mkdtempSync(path.join(tmpdir(), "ferromark-native-provenance-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const artifacts = path.join(directory, "artifacts");
  let destination = path.join(directory, "destination");
  mkdirSync(artifacts);
  const packageJson = JSON.parse(
    readFileSync(path.join(root, "node/ferromark/package.json"), "utf8"),
  );
  const sourceSha = "1".repeat(40);
  const platformIds = Object.keys(packageJson.optionalDependencies).map((name) =>
    name.slice("ferromark-".length),
  );

  for (const platformId of platformIds) {
    writeFileSync(path.join(artifacts, "ferromark." + platformId + ".node"), "fixture");
    writeFileSync(
      path.join(artifacts, "ferromark-source-" + platformId + ".json"),
      JSON.stringify({ schemaVersion: 1, platformId, sourceSha }),
    );
  }

  const verify = () =>
    spawnSync(process.execPath, [verifyNative.pathname], {
      cwd: path.join(root, "node"),
      encoding: "utf8",
      env: {
        ...process.env,
        ARTIFACT_DIRECTORY: artifacts,
        DESTINATION_DIRECTORY: destination,
        SOURCE_SHA: sourceSha,
      },
    });
  const valid = verify();
  assert.equal(valid.status, 0, valid.stderr);
  assert.deepEqual(
    readdirSync(destination).sort(),
    platformIds.map((id) => "ferromark." + id + ".node").sort(),
  );

  const badMetadataPath = path.join(artifacts, "ferromark-source-" + platformIds[0] + ".json");
  const mixedDestination = path.join(directory, "mixed-destination");
  destination = mixedDestination;
  writeFileSync(
    badMetadataPath,
    JSON.stringify({ schemaVersion: 1, platformId: platformIds[0], sourceSha: "2".repeat(40) }),
  );
  const mixed = verify();
  assert.notEqual(mixed.status, 0, "accepted artifacts built from different source commits");
  assert.equal(
    existsSync(mixedDestination),
    false,
    "created a promotion directory for mixed artifacts",
  );
});

test("accepts only a published GitHub release that matches the tag channel", () => {
  assert.doesNotThrow(() =>
    verifyGitHubRelease("v1.2.3", {
      tagName: "v1.2.3",
      isDraft: false,
      isPrerelease: false,
    }),
  );
  assert.doesNotThrow(() =>
    verifyGitHubRelease("v1.2.3-rc.1", {
      tagName: "v1.2.3-rc.1",
      isDraft: false,
      isPrerelease: true,
    }),
  );
  assert.throws(() =>
    verifyGitHubRelease("v1.2.3", {
      tagName: "v1.2.3",
      isDraft: true,
      isPrerelease: false,
    }),
  );
  assert.throws(() =>
    verifyGitHubRelease("v1.2.3-rc.1", {
      tagName: "v1.2.3-rc.1",
      isDraft: false,
      isPrerelease: false,
    }),
  );
  assert.throws(() =>
    verifyGitHubRelease("v1.2.3", {
      tagName: "v1.2.4",
      isDraft: false,
      isPrerelease: false,
    }),
  );
});

test("metadata release gate checks the checked-out release manifests", () => {
  const cargo = readFileSync(path.join(root, "Cargo.toml"), "utf8");
  const version = cargo.match(/^version = "([^"]+)"/m)?.[1];
  assert.ok(version);
  assert.match(version, /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-rc\.(?:0|[1-9]\d*))?$/);

  const result = spawnSync(
    process.execPath,
    [
      path.join(root, "node/scripts/verify-release.mjs"),
      "--metadata-only",
      "--root",
      root,
      "--tag",
      "v" + version,
    ],
    { cwd: root, encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /metadata only/);
  const wrongTag = spawnSync(
    process.execPath,
    [
      path.join(root, "node/scripts/verify-release.mjs"),
      "--metadata-only",
      "--root",
      root,
      "--tag",
      "v9.8.7",
    ],
    { cwd: root, encoding: "utf8" },
  );
  assert.notEqual(wrongTag.status, 0, "accepted a release tag with a mismatched package version");
});
