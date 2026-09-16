import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { releaseChannel } from "../node/scripts/release-channel.mjs";
import { publishArguments, releaseArchives } from "../node/scripts/release-archives.mjs";

const facade = JSON.parse(
  readFileSync(new URL("../node/ferromark/package.json", import.meta.url), "utf8"),
);

/** Nine archives with the manifests pnpm writes after resolving `workspace:*`. */
function archives(t, transform = (manifest) => manifest) {
  const root = mkdtempSync(join(tmpdir(), "ferromark-release-archives-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, "package"));
  const sidecars = Object.fromEntries(
    Object.keys(facade.optionalDependencies).map((name) => [name, facade.version]),
  );
  for (const name of [...Object.keys(facade.optionalDependencies), facade.name]) {
    const manifest = transform(
      name === facade.name
        ? { ...facade, optionalDependencies: sidecars }
        : { name, version: facade.version, private: false },
    );
    writeFileSync(join(root, "package/package.json"), JSON.stringify(manifest));
    assert.equal(
      spawnSync("tar", ["-czf", join(root, `${name}-${facade.version}.tgz`), "-C", root, "package"])
        .status,
      0,
    );
  }
  return root;
}

test("publishes release candidates only to the next channel", () => {
  assert.deepEqual(releaseChannel("2.0.0-rc.1"), { tag: "next", prerelease: true });
});

test("publishes stable versions to latest", () => {
  assert.deepEqual(releaseChannel("2.0.0"), { tag: "latest", prerelease: false });
});

test("rejects development versions and malformed release versions", () => {
  for (const version of [
    "2.0.0-dev.0",
    "v2.0.0",
    "2.0",
    "2.0.0-rc.01",
    "02.0.0",
    "2.0.0\n",
    "2.0.0-rc.1 --tag latest",
  ]) {
    assert.throws(() => releaseChannel(version));
  }
});

test("orders the native archives before the facade and names the channel", (t) => {
  const { distTag, archives: ordered } = releaseArchives(archives(t), facade);
  assert.equal(ordered.length, 9);
  assert.equal(distTag, releaseChannel(facade.version).tag);
  assert.ok(ordered.at(-1).endsWith(`/ferromark-${facade.version}.tgz`));
  for (const archive of ordered.slice(0, -1)) {
    assert.match(archive, /\/ferromark-[a-z0-9-]+-\d/);
  }
});

test("rejects a packed facade whose sidecar references were not resolved", (t) => {
  const directory = archives(t, (manifest) =>
    manifest.name === facade.name
      ? {
          ...manifest,
          optionalDependencies: Object.fromEntries(
            Object.keys(manifest.optionalDependencies).map((name) => [name, "workspace:*"]),
          ),
        }
      : manifest,
  );
  assert.throws(() => releaseArchives(directory, facade), /pin every sidecar/);
});

test("rejects an archive set that is missing a platform", (t) => {
  const directory = archives(t);
  rmSync(join(directory, `ferromark-darwin-arm64-${facade.version}.tgz`));
  assert.throws(() => releaseArchives(directory, facade), /exactly the nine release archives/);
});

test("hands npm publish local tarball paths, never a GitHub shorthand", (t) => {
  const directory = archives(t);
  const { archives: ordered } = releaseArchives(directory, facade);
  const args = publishArguments(ordered, join(directory, ".."));
  assert.equal(args.length, 9);
  for (const arg of args) {
    // `artifacts/name.tgz` would be read as the shorthand `owner/repo`.
    assert.match(arg, /^\.\/[^/]+\/[^/]+\.tgz$/, arg);
    assert.ok(arg.endsWith(".tgz"));
  }
  assert.equal(args.at(-1), `./${basename(directory)}/${facade.name}-${facade.version}.tgz`);
});
