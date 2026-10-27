import assert from "node:assert/strict";
import { appendFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { planNpmPublications } from "./native-registry.mjs";

export { planNpmPublications, registryVersionUrl, REQUEST_TIMEOUT_MS } from "./native-registry.mjs";

function readCliConfiguration() {
  const tagNodeDirectory = process.env.TAG_NODE_DIRECTORY;
  const archiveDirectory = process.argv[2];
  const workflowPackages = process.env.PACKAGES;
  const githubOutput = process.env.GITHUB_OUTPUT;
  if (
    !tagNodeDirectory ||
    !archiveDirectory ||
    process.argv.length > 3 ||
    !workflowPackages ||
    !githubOutput
  ) {
    throw new Error(
      "an archive directory, TAG_NODE_DIRECTORY, PACKAGES, and GITHUB_OUTPUT are required",
    );
  }
  assert.equal(
    resolve(archiveDirectory),
    resolve(tagNodeDirectory, "artifacts"),
    "the npm preflight must inspect the release-tag artifacts directory",
  );
  assert.equal(
    resolve(process.cwd()),
    resolve(tagNodeDirectory),
    "the npm preflight must run from the immutable release-tag checkout",
  );
  return {
    archiveDirectory,
    githubOutput,
    tagNodeDirectory,
    workflowPackages: workflowPackages.split(/\r?\n/).filter(Boolean),
  };
}

async function readTaggedArchives({ archiveDirectory, tagNodeDirectory, workflowPackages }) {
  // The workflow checks out its immutable release tag at node/, while this
  // helper is loaded from the workflow's immutable commit in release-tools/.
  // Reuse the tag's archive validator so registry checks name the exact files
  // whose manifests and facade dependencies were just verified.
  const releaseArchivesModule = await import(
    pathToFileURL(resolve(tagNodeDirectory, "scripts/release-archives.mjs")).href
  );
  const facade = JSON.parse(
    readFileSync(resolve(tagNodeDirectory, "ferromark/package.json"), "utf8"),
  );
  const { archives } = releaseArchivesModule.releaseArchives(resolve(archiveDirectory), facade);
  const archiveArguments = releaseArchivesModule.publishArguments(
    archives,
    resolve(tagNodeDirectory),
  );
  assert.deepEqual(
    workflowPackages,
    archiveArguments,
    "the npm preflight must receive every validated release archive in sidecar-first order",
  );

  return {
    archiveArguments,
    facade,
    packageNames: [...Object.keys(facade.optionalDependencies).sort(), facade.name],
  };
}

function writeGithubOutput({ githubOutput, plan, version }) {
  for (const packageName of plan.existingPackages) {
    console.log(`Skipping already published ${packageName}@${version}`);
  }
  for (const archive of plan.publishArguments) {
    console.log(`Will publish ${archive}`);
  }

  const output = [`publish_required=${plan.publishArguments.length > 0}`];
  if (plan.publishArguments.length > 0) {
    output.push("packages<<PUBLISH_PACKAGES_EOF", ...plan.publishArguments, "PUBLISH_PACKAGES_EOF");
  }
  appendFileSync(githubOutput, `${output.join("\n")}\n`);
}

async function main() {
  const configuration = readCliConfiguration();
  const release = await readTaggedArchives(configuration);
  const plan = await planNpmPublications({
    packageNames: release.packageNames,
    version: release.facade.version,
    archiveArguments: release.archiveArguments,
  });
  writeGithubOutput({
    githubOutput: configuration.githubOutput,
    plan,
    version: release.facade.version,
  });
}

if (process.argv[1] === import.meta.filename) {
  try {
    await main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
