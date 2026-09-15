import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { publishPackages } from "../node/scripts/publish-packages.mjs";

const pkg = JSON.parse(readFileSync(new URL("../node/ferromark/package.json", import.meta.url)));
function fixtures(t, transform = (value) => value) {
  const root = mkdtempSync(join(tmpdir(), "ferromark-publisher-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(join(root, "package"));
  for (const name of [...Object.keys(pkg.optionalDependencies), pkg.name]) {
    const manifest = transform(
      name === pkg.name ? pkg : { name, version: pkg.version, private: false },
    );
    writeFileSync(join(root, "package/package.json"), JSON.stringify(manifest));
    assert.equal(
      spawnSync("tar", ["-czf", join(root, `${name}-${pkg.version}.tgz`), "-C", root, "package"])
        .status,
      0,
    );
  }
  return root;
}

test("publishes native archives before facade and selects the RC channel explicitly", async (t) => {
  const root = fixtures(t);
  const calls = [];
  await publishPackages(root, pkg.version, {
    request: async () => ({ ok: false, status: 404 }),
    execute: (args) => calls.push(args),
  });
  assert.equal(calls.length, 9);
  assert.ok(calls.at(-1)[1].endsWith(`/ferromark-${pkg.version}.tgz`));
  for (const args of calls)
    assert.deepEqual(args.slice(2), ["--access", "public", "--provenance", "--tag", "next"]);
});

test("rejects a conflicting final package before publishing any earlier package", async (t) => {
  const root = fixtures(t);
  const calls = [];
  const request = async (url) =>
    url.includes("/ferromark/")
      ? { ok: true, json: async () => ({ dist: { integrity: "sha512-conflict" } }) }
      : { ok: false, status: 404 };
  await assert.rejects(
    publishPackages(root, pkg.version, { request, execute: (args) => calls.push(args) }),
    /existing version differs/,
  );
  assert.deepEqual(calls, []);
});

test("accepts identical archives on retry without republishing immutable versions", async (t) => {
  const root = fixtures(t);
  const calls = [];
  const request = async (url) => {
    const name = new URL(url).pathname.split("/")[1];
    const integrity = `sha512-${createHash("sha512")
      .update(readFileSync(join(root, `${name}-${pkg.version}.tgz`)))
      .digest("base64")}`;
    return { ok: true, json: async () => ({ dist: { integrity } }) };
  };
  await publishPackages(root, pkg.version, { request, execute: (args) => calls.push(args) });
  assert.equal(calls.length, 9);
  for (const args of calls)
    assert.deepEqual([args[0], args[1], args[3]], ["dist-tag", "add", "next"]);
});

test("rejects a facade with stale native dependencies before any publication", async (t) => {
  const root = fixtures(t, (manifest) =>
    manifest.name === pkg.name ? { ...manifest, optionalDependencies: {} } : manifest,
  );
  const calls = [];
  await assert.rejects(
    publishPackages(root, pkg.version, {
      request: async () => ({ ok: false, status: 404 }),
      execute: (args) => calls.push(args),
    }),
    assert.AssertionError,
  );
  assert.deepEqual(calls, []);
});
