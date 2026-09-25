import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse } from "yaml";
import { runRemarkGithubReference } from "./lib/remark-github-reference.mjs";

const scriptsDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = dirname(scriptsDirectory);
const fixtures = JSON.parse(
  await readFile(join(repositoryRoot, "benchmarks/remark-github-oracle/fixtures.json"), "utf8"),
);
const packageJson = JSON.parse(await readFile(join(scriptsDirectory, "package.json"), "utf8"));
const lockfile = parse(await readFile(join(scriptsDirectory, "pnpm-lock.yaml"), "utf8"));

test("remark-github reference package and harness are pinned to the recorded provenance", async () => {
  assert.equal(fixtures.reference.package, "remark-github");
  assert.equal(fixtures.reference.version, "12.0.0");
  assert.equal(fixtures.reference.gitHead, "9cd4e9d8fa6cd3520d11aa922462a0089e1204c5");
  assert.equal(
    fixtures.reference.registryIntegrity,
    lockfile.packages["remark-github@12.0.0"].resolution.integrity,
  );

  for (const [name, version] of Object.entries(fixtures.configuration.packages)) {
    assert.equal(packageJson.devDependencies[name], version, `${name} must stay directly pinned`);
  }
});

test("checked-in fixtures match the executed upstream transform", async (t) => {
  assert.ok(fixtures.cases.length > 0, "the reference corpus must not be empty");
  assert.equal(new Set(fixtures.cases.map(({ name }) => name)).size, fixtures.cases.length);

  for (const fixture of fixtures.cases) {
    await t.test(fixture.name, async () => {
      const actual = await runRemarkGithubReference(
        fixture.source,
        fixtures.configuration.repository,
        fixture.parserPlugins ?? fixtures.configuration.parserPlugins,
      );
      assert.deepEqual(actual.parsedTree, fixture.parsedTree, "parser output changed");
      assert.deepEqual(actual.transformedTree, fixture.transformedTree, "plugin output changed");
      assert.equal(actual.markdown, fixture.markdown, "remark serialization changed");
    });
  }
});
