import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import TOML from "@iarna/toml";
import { Manifest, setLogger } from "release-please";
import { parse as parseYaml } from "yaml";

export const root = resolve(import.meta.dirname, "../..");
const quiet = Object.fromEntries(
  ["info", "debug", "trace", "warn", "error"].map((name) => [name, () => {}]),
);

// Representative of an ordinary maintenance range on top of a stable release:
// corrections and documentation, and no `Release-As` footer. The default
// strategy must answer that range with the next patch version on its own.
export const maintenanceHistory = [
  "fix(renderer): keep footnote back-references inside the definition list",
  "docs(releasing): describe the stable version selection",
  "chore(deps): refresh the pinned toolchain",
];

/** The eight native sidecar directories, read from the workspace itself. */
export function nativeTargets() {
  return readdirSync(resolve(root, "node/ferromark/npm")).sort();
}

/**
 * Expand one `extra-files` entry to the repository-relative paths it updates.
 * `glob: true` entries are matched against the real directory, the way the
 * library matches them against the target branch.
 */
function extraFilePaths(entry) {
  if (typeof entry === "string") return [entry];
  if (!entry.glob) return [entry.path];
  const [prefix, suffix] = entry.path.split("*");
  return nativeTargets().map((target) => `${prefix}${target}${suffix}`);
}

export function readReleaseFiles() {
  const read = (file) => readFileSync(resolve(root, file), "utf8");
  const config = JSON.parse(read("release-please-config.json"));
  const manifest = TOML.parse(read("Cargo.toml"));
  const paths = new Set([
    "release-please-config.json",
    ".release-please-manifest.json",
    "Cargo.toml",
    "Cargo.lock",
    "node/pnpm-lock.yaml",
    ...manifest.workspace.members.map((member) => `${member}/Cargo.toml`),
    ...config.packages["."]["extra-files"].flatMap(extraFilePaths),
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
    // The `extra-files` glob for the eight native manifests resolves against
    // the known file set rather than against a branch on GitHub.
    async findFilesByGlobAndRef(glob, _ref, prefix) {
      const pattern = new RegExp(
        `^${glob
          .split("*")
          .map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
          .join("[^/]*")}$`,
      );
      const base = prefix === undefined || prefix === "." ? "" : `${prefix}/`;
      return [...files.keys()]
        .filter((file) => file.startsWith(base) && pattern.test(file.slice(base.length)))
        .map((file) => file.slice(base.length));
    },
    async *releaseIterator() {
      yield { tagName: `v${previous}`, sha: "a".repeat(40), notes: "Rehearsal baseline" };
    },
    async *mergeCommitIterator() {
      for (const [index, message] of messages.entries()) {
        // Distinct hexadecimal shas; "a" is reserved for the released commit.
        const sha = `${index + 1}`.padStart(40, "b");
        yield { sha, message, files: ["src/lib.rs"] };
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

  // `release-type: rust` writes the root package version, every member version,
  // the published internal requirements and Cargo.lock — natively, with no
  // `version.txt` and no Cargo `extra-files` to keep in step.
  assert.equal(cargo.package.name, "ferromark", "Root package");
  assert.equal(cargo.package.version, expectedVersion, "Root package version");
  assert.equal(
    cargo.workspace.package?.version,
    undefined,
    "An inherited workspace version cannot be updated and must not come back",
  );
  assert.equal(json(".release-please-manifest.json")["."], expectedVersion, "Release manifest");

  const localNames = ["ferromark"];
  for (const member of cargo.workspace.members) {
    const memberManifest = TOML.parse(files.get(`${member}/Cargo.toml`));
    assert.equal(memberManifest.package.version, expectedVersion, `${member}: package version`);
    assert.equal(memberManifest.package.publish, false, `${member}: stays unpublished`);
    const requirement = memberManifest.dependencies?.ferromark;
    if (requirement) {
      assert.equal(requirement.version, expectedVersion, `${member}: ferromark requirement`);
      assert.equal(requirement.path, "../..", `${member}: ferromark path`);
    }
    localNames.push(memberManifest.package.name);
  }

  const lock = TOML.parse(files.get("Cargo.lock"));
  for (const name of localNames) {
    const matches = lock.package.filter((pkg) => pkg.name === name && !pkg.source);
    assert.equal(matches.length, 1, `${name}: unique local lockfile entry`);
    assert.equal(matches[0].version, expectedVersion, `${name}: Cargo.lock version`);
  }

  // The docs.rs reference is the only versioned line left in a README, so it is
  // the one assertion that proves the generic updater still runs. The install
  // lines name the package alone and must stay that way after a bump.
  for (const readme of ["README.md.src", "README.md"]) {
    const text = files.get(readme);
    assert.ok(
      text.includes(`https://docs.rs/ferromark/${expectedVersion}/ferromark/`),
      `${readme}: docs.rs link`,
    );
    assert.ok(text.includes("cargo add ferromark\n"), `${readme}: unversioned cargo add`);
    assert.ok(text.includes("npm install ferromark\n"), `${readme}: unversioned npm install`);
  }

  const main = json("node/ferromark/package.json");
  const pnpm = parseYaml(files.get("node/pnpm-lock.yaml"));
  assert.equal(main.version, expectedVersion, "npm facade version");
  assert.equal(Object.keys(main.optionalDependencies).length, 8, "Eight native packages");
  for (const [name, reference] of Object.entries(main.optionalDependencies)) {
    const target = name.replace(/^ferromark-/, "");
    // The sidecar reference carries no version at all, which is what keeps the
    // pnpm lockfile out of the release template.
    assert.equal(reference, "workspace:*", `${name}: workspace reference`);
    assert.equal(
      json(`node/ferromark/npm/${target}/package.json`).version,
      expectedVersion,
      `${name}: npm version`,
    );
    assert.deepEqual(
      pnpm.importers.ferromark.optionalDependencies[name],
      { specifier: "workspace:*", version: `link:npm/${target}` },
      `${name}: pnpm lockfile`,
    );
  }
}
