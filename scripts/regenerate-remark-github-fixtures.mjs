import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import { runRemarkGithubReference } from "./lib/remark-github-reference.mjs";

const fixturesUrl = new URL("../benchmarks/remark-github-oracle/fixtures.json", import.meta.url);

if (process.argv.length !== 3 || process.argv[2] !== "--write") {
  throw new Error("Regeneration requires the explicit --write flag.");
}

const fixtures = JSON.parse(await readFile(fixturesUrl, "utf8"));
assert.equal(fixtures.reference.package, "remark-github");
assert.equal(fixtures.reference.version, "12.0.0");
assert.equal(fixtures.reference.gitHead, "9cd4e9d8fa6cd3520d11aa922462a0089e1204c5");

for (const fixture of fixtures.cases) {
  const reference = await runRemarkGithubReference(
    fixture.source,
    fixtures.configuration.repository,
    fixture.parserPlugins ?? fixtures.configuration.parserPlugins,
  );
  Object.assign(fixture, reference);
}

await writeFile(fixturesUrl, `${JSON.stringify(fixtures, null, 2)}\n`);
console.log(`Regenerated ${fixtures.cases.length} remark-github reference fixtures.`);
