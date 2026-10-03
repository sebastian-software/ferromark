import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { planNpmPublications, registryVersionUrl } from "../../scripts/npm-publish-plan.mjs";

const names = ["ferromark-darwin-arm64", "ferromark-linux-x64-gnu", "ferromark"];
const archives = names.map((name) => `./artifacts/${name}-3.1.0.tgz`);
const version = "3.1.0";

function registry(responses) {
  const requests = [];
  return {
    requests,
    fetchImpl(url, options) {
      requests.push({ url, options });
      const response = responses.get(url);
      if (response instanceof Error) return Promise.reject(response);
      if (!response) throw new Error(`unexpected registry request: ${url}`);
      return Promise.resolve(response);
    },
  };
}

function published(packageName, packageVersion = version) {
  return {
    status: 200,
    async json() {
      return { name: packageName, version: packageVersion };
    },
  };
}

function missing() {
  return { status: 404 };
}

function responsesFor(entries) {
  return new Map(entries.map(([name, response]) => [registryVersionUrl(name, version), response]));
}

test("skips only exact versions already published", async () => {
  const { fetchImpl, requests } = registry(
    responsesFor(names.map((name) => [name, published(name)])),
  );

  const plan = await planNpmPublications({
    packageNames: names,
    version,
    archiveArguments: archives,
    fetchImpl,
  });

  assert.deepEqual(plan, { existingPackages: names, publishArguments: [] });
  assert.deepEqual(
    requests.map(({ url }) => url),
    names.map((name) => registryVersionUrl(name, version)),
  );
});

test("publishes all missing packages in the supplied sidecar-first order", async () => {
  const { fetchImpl } = registry(responsesFor(names.map((name) => [name, missing()])));

  const plan = await planNpmPublications({
    packageNames: names,
    version,
    archiveArguments: archives,
    fetchImpl,
  });

  assert.deepEqual(plan, { existingPackages: [], publishArguments: archives });
});

test("filters mixed existing and missing archives without reordering", async () => {
  const { fetchImpl, requests } = registry(
    responsesFor([
      [names[0], published(names[0])],
      [names[1], missing()],
      [names[2], missing()],
    ]),
  );

  const plan = await planNpmPublications({
    packageNames: names,
    version,
    archiveArguments: archives,
    fetchImpl,
  });

  assert.deepEqual(plan, {
    existingPackages: [names[0]],
    publishArguments: [archives[1], archives[2]],
  });
  assert.deepEqual(
    requests.map(({ url }) => url),
    names.map((name) => registryVersionUrl(name, version)),
  );
});

test("fails closed on registry network errors", async () => {
  const failure = new Error("network unavailable");
  const { fetchImpl } = registry(responsesFor([[names[0], failure]]));

  await assert.rejects(
    planNpmPublications({
      packageNames: names,
      version,
      archiveArguments: archives,
      fetchImpl,
    }),
    /request failed for ferromark-darwin-arm64@3\.1\.0: network unavailable/,
  );
});

test("fails closed on registry errors other than HTTP 404", async () => {
  const { fetchImpl } = registry(responsesFor([[names[0], { status: 503 }]]));

  await assert.rejects(
    planNpmPublications({
      packageNames: names,
      version,
      archiveArguments: archives,
      fetchImpl,
    }),
    /registry returned HTTP 503 for ferromark-darwin-arm64@3\.1\.0/,
  );
});

test("fails closed when HTTP 200 metadata does not identify the exact package and version", async () => {
  const { fetchImpl } = registry(
    responsesFor([[names[0], published("some-other-package", version)]]),
  );

  await assert.rejects(
    planNpmPublications({
      packageNames: names,
      version,
      archiveArguments: archives,
      fetchImpl,
    }),
    /registry returned some-other-package@3\.1\.0 for ferromark-darwin-arm64@3\.1\.0/,
  );
});

test("fails closed when HTTP 200 metadata identifies the package at a different version", async () => {
  const { fetchImpl } = registry(responsesFor([[names[0], published(names[0], "3.0.9")]]));

  await assert.rejects(
    planNpmPublications({
      packageNames: names,
      version,
      archiveArguments: archives,
      fetchImpl,
    }),
    /registry returned ferromark-darwin-arm64@3\.0\.9 for ferromark-darwin-arm64@3\.1\.0/,
  );
});

test("fails closed on malformed HTTP 200 metadata", async () => {
  const { fetchImpl } = registry(
    responsesFor([
      [
        names[0],
        {
          status: 200,
          async json() {
            throw new Error("invalid JSON");
          },
        },
      ],
    ]),
  );

  await assert.rejects(
    planNpmPublications({
      packageNames: names,
      version,
      archiveArguments: archives,
      fetchImpl,
    }),
    /invalid metadata for ferromark-darwin-arm64@3\.1\.0: invalid JSON/,
  );
});

test("fails closed on unexpected successful statuses and mismatched archive lists", async () => {
  const { fetchImpl } = registry(responsesFor([[names[0], { status: 201 }]]));

  await assert.rejects(
    planNpmPublications({
      packageNames: names,
      version,
      archiveArguments: archives,
      fetchImpl,
    }),
    /registry returned HTTP 201/,
  );
  await assert.rejects(
    planNpmPublications({
      packageNames: names,
      version,
      archiveArguments: archives.slice(1),
      fetchImpl,
    }),
    /every release package must have one validated archive argument/,
  );
});

test("workflow keeps the release tag checkout and pins its helper to the workflow commit", () => {
  const workflow = readFileSync(
    new URL("../../../.github/workflows/publish.yml", import.meta.url),
    "utf8",
  );

  assert.match(
    workflow,
    /ref: \$\{\{ inputs\.tag \|\| needs\.release-please\.outputs\.tag_name \}\}/,
  );
  assert.match(
    workflow,
    /ref: \$\{\{ github\.workflow_sha \}\}\s+path: release-tools\s+persist-credentials: false/,
  );
  assert.match(
    workflow,
    /node \.\.\/release-tools\/node\/scripts\/npm-publish-plan\.mjs \.\/artifacts/,
  );
  assert.match(
    workflow,
    /if: \$\{\{ steps\.npm-publish-plan\.outputs\.publish_required == 'true' \}\}/,
  );
  assert.match(workflow, /packages: \$\{\{ steps\.npm-publish-plan\.outputs\.packages \}\}/);
  const publishAction =
    "uses: sebastian-software/standards/.github/actions/publish-npm@" +
    "a840766444dbb0e421f1f1868c48bee315ead769";
  assert.ok(workflow.includes(publishAction));
  assert.match(
    workflow,
    /verify-npm-publish:[\s\S]*?needs: \[release-please, native-matrix, publish-npm\]/,
  );
  assert.match(workflow, /NPM_PUBLISH_RESULT: \$\{\{ needs\.publish-npm\.result \}\}/);
});
