import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import TOML from "@iarna/toml";
import { Manifest, setLogger } from "release-please";
import { parse as parseYaml } from "yaml";

export const root = resolve(import.meta.dirname, "../..");
const quiet = Object.fromEntries(
  ["info", "debug", "trace", "warn", "error"].map((name) => [name, () => {}]),
);

// Representative of `git log --format=%B v2.0.0-rc.1..main`: a breaking renderer
// change, a Node feature and documentation, and no `Release-As` footer. The
// prerelease strategy must answer a breaking change on `2.0.0-rc.1` with
// `2.0.0-rc.2` rather than promoting the candidate to a stable `3.0.0`.
export const candidateHistory = [
  "perf(renderer)!: borrow default renderer option strings\n\n" +
    "BREAKING CHANGE: `HtmlRendererOptions::soft_break`, `hard_break` and `base_url` borrow their defaults.",
  "feat(node): add a profile-guided training driver",
  "docs(perf): record the third Apple Silicon iteration round",
];

export function readReleaseFiles() {
  const read = (file) => readFileSync(resolve(root, file), "utf8");
  const config = JSON.parse(read("release-please-config.json"));
  const workspace = TOML.parse(read("Cargo.toml")).workspace;
  const paths = new Set([
    "release-please-config.json",
    ".release-please-manifest.json",
    "Cargo.toml",
    "Cargo.lock",
    "version.txt",
    "node/pnpm-lock.yaml",
    ...workspace.members.map((member) => `${member}/Cargo.toml`),
    ...config.packages["."]["extra-files"].map((entry) =>
      typeof entry === "string" ? entry : entry.path,
    ),
  ]);
  if (existsSync(resolve(root, "CHANGELOG.md"))) paths.add("CHANGELOG.md");
  return new Map([...paths].map((file) => [file, read(file)]));
}

// The real Manifest/strategy/updaters run against local file contents and a
// synthetic commit history. No network client or publishing method exists here.
// `history` is one commit message or, newest first, the range since the last
// release tag; version selection reads the whole range, not just its tip.
export async function proposeRelease(files, history) {
  setLogger(quiet);
  const messages = Array.isArray(history) ? history : [history];
  assert.ok(messages.length > 0, "A release proposal needs at least one commit");
  const previous = JSON.parse(files.get(".release-please-manifest.json"))["."];
  const github = {
    repository: { owner: "sebastian-software", repo: "ferromark", defaultBranch: "main" },
    async getFileJson(file) {
      return JSON.parse(files.get(file));
    },
    async getFileContentsOnBranch(file) {
      assert.ok(files.has(file), `Missing rehearsal file: ${file}`);
      return {
        parsedContent: files.get(file),
        content: Buffer.from(files.get(file)).toString("base64"),
        sha: "a".repeat(40),
      };
    },
    async *releaseIterator() {
      yield { tagName: `v${previous}`, sha: "a".repeat(40), notes: "Rehearsal baseline" };
    },
    async *mergeCommitIterator() {
      for (const [index, message] of messages.entries()) {
        // Distinct hexadecimal shas; "a" is reserved for the released commit.
        const sha = `${index + 1}`.padStart(40, "b");
        yield { sha, message, files: ["crates/ferromark/src/lib.rs"] };
      }
      yield { sha: "a".repeat(40), message: "chore: preceding release", files: [] };
    },
  };
  const manifest = await Manifest.fromManifest(
    github,
    "main",
    "release-please-config.json",
    ".release-please-manifest.json",
    { logger: quiet },
  );
  const candidates = await manifest.buildPullRequests();
  assert.equal(candidates.length, 1, "Exactly one coordinated release PR is required");
  const proposal = candidates[0];
  const updated = new Map(files);
  for (const change of proposal.updates) {
    assert.ok(
      updated.has(change.path) || change.createIfMissing,
      `Unexpected file: ${change.path}`,
    );
    updated.set(change.path, change.updater.updateContent(updated.get(change.path), quiet));
  }
  return {
    files: updated,
    title: proposal.title.toString(),
    body: proposal.body.toString(),
    paths: proposal.updates.map((u) => u.path),
  };
}

export function validateRelease(files, expectedVersion) {
  const json = (file) => JSON.parse(files.get(file));
  const cargo = TOML.parse(files.get("Cargo.toml"));
  assert.equal(cargo.workspace.package.version, expectedVersion, "Rust workspace version");
  assert.equal(files.get("version.txt").trim(), expectedVersion, "Release version file");
  assert.equal(json(".release-please-manifest.json")["."], expectedVersion, "Release manifest");
  const localNames = [];
  for (const member of cargo.workspace.members) {
    const pkg = TOML.parse(files.get(`${member}/Cargo.toml`)).package;
    assert.deepEqual(pkg.version, { workspace: true }, `${member}: version inheritance`);
    localNames.push(pkg.name);
  }
  const lock = TOML.parse(files.get("Cargo.lock"));
  for (const name of localNames) {
    const matches = lock.package.filter((pkg) => pkg.name === name && !pkg.source);
    assert.equal(matches.length, 1, `${name}: unique local lockfile entry`);
    assert.equal(matches[0].version, expectedVersion, `${name}: Cargo.lock version`);
    if (name !== "ferromark-node") {
      assert.equal(
        cargo.workspace.dependencies[name].version,
        `=${expectedVersion}`,
        `${name}: exact dependency pin`,
      );
    }
  }
  const main = json("node/ferromark/package.json");
  const pnpm = parseYaml(files.get("node/pnpm-lock.yaml"));
  assert.equal(main.version, expectedVersion, "npm facade version");
  assert.equal(Object.keys(main.optionalDependencies).length, 8, "Eight native packages");
  for (const [name, pin] of Object.entries(main.optionalDependencies)) {
    const target = name.replace(/^ferromark-/, "");
    assert.equal(pin, expectedVersion, `${name}: optional dependency pin`);
    assert.equal(
      json(`node/ferromark/npm/${target}/package.json`).version,
      expectedVersion,
      `${name}: npm version`,
    );
    assert.deepEqual(
      pnpm.importers.ferromark.optionalDependencies[name],
      { specifier: expectedVersion, version: `link:npm/${target}` },
      `${name}: pnpm lockfile`,
    );
  }
}
