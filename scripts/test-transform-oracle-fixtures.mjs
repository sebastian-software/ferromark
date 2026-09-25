import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse } from "yaml";
import { runTransformReference } from "./lib/transform-reference.mjs";

const scriptsDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = dirname(scriptsDirectory);
const fixtures = JSON.parse(
  await readFile(join(repositoryRoot, "benchmarks/native-transform-oracles/fixtures.json"), "utf8"),
);
const packageJson = JSON.parse(await readFile(join(scriptsDirectory, "package.json"), "utf8"));
const lockfile = parse(await readFile(join(scriptsDirectory, "pnpm-lock.yaml"), "utf8"));

test("reference artifacts match the recorded immutable package identities", () => {
  const directPackages = Object.values(fixtures.referencePackages).filter((reference) =>
    Boolean(reference.package),
  );

  for (const reference of directPackages) {
    assert.equal(
      packageJson.devDependencies[reference.package],
      reference.version,
      reference.package + " must stay directly pinned",
    );
    assert.equal(
      lockfile.packages[reference.lockKey].resolution.integrity,
      reference.registryIntegrity,
      reference.package + " integrity changed",
    );

    for (const dependency of reference.dependencies ?? []) {
      assert.equal(
        lockfile.packages[dependency.lockKey].resolution.integrity,
        dependency.registryIntegrity,
        dependency.package + " integrity changed",
      );
    }
  }

  assert.equal(
    fixtures.referencePackages.runner.packages.remark,
    packageJson.devDependencies.remark,
  );
  assert.equal(
    fixtures.referencePackages.runner.packages["remark-gfm"],
    packageJson.devDependencies["remark-gfm"],
  );
});

test("frozen reference outputs still match each executed transform", async (t) => {
  for (const fixture of fixtures.cases) {
    await t.test(fixture.name, async () => {
      const actual = await runTransformReference(fixture);
      assert.deepEqual(actual.parsedTree, fixture.parsedTree, "parser output changed");
      assert.deepEqual(actual.transformedTree, fixture.transformedTree, "transform output changed");
      assert.equal(actual.markdown, fixture.markdown, "remark serialization changed");
    });
  }
});
