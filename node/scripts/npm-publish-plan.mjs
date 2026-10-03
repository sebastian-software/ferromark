import assert from "node:assert/strict";
import { appendFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

export const REQUEST_TIMEOUT_MS = 10_000;

export function registryVersionUrl(packageName, version) {
  const encodedPackageName = encodeURIComponent(packageName);
  const encodedVersion = encodeURIComponent(version);
  return `https://registry.npmjs.org/${encodedPackageName}/${encodedVersion}`;
}

export async function planNpmPublications({
  packageNames,
  version,
  archiveArguments,
  fetchImpl = fetch,
}) {
  assert.equal(
    packageNames.length,
    archiveArguments.length,
    "every release package must have one validated archive argument",
  );

  const publishArguments = [];
  const existingPackages = [];
  for (const [index, packageName] of packageNames.entries()) {
    const exists = await hasExactRegistryVersion(packageName, version, fetchImpl);
    if (exists) {
      existingPackages.push(packageName);
    } else {
      publishArguments.push(archiveArguments[index]);
    }
  }

  return { existingPackages, publishArguments };
}

async function hasExactRegistryVersion(packageName, version, fetchImpl) {
  const response = await fetchRegistryVersion(packageName, version, fetchImpl);
  if (response.status === 404) return false;
  if (response.status !== 200) {
    throw new Error(`npm registry returned HTTP ${response.status} for ${packageName}@${version}`);
  }
  await verifyRegistryIdentity(response, packageName, version);
  return true;
}

async function fetchRegistryVersion(packageName, version, fetchImpl) {
  try {
    return await fetchImpl(registryVersionUrl(packageName, version), {
      cache: "no-store",
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
  } catch (error) {
    const message = `npm registry request failed for ${packageName}@${version}: ${error.message}`;
    throw new Error(message, { cause: error });
  }
}

async function verifyRegistryIdentity(response, packageName, version) {
  let metadata;
  try {
    metadata = await response.json();
  } catch (error) {
    const message = `npm registry returned invalid metadata for ${packageName}@${version}: ${error.message}`;
    throw new Error(message, { cause: error });
  }
  if (metadata?.name !== packageName || metadata?.version !== version) {
    const receivedVersion = `${metadata?.name ?? "unknown"}@${metadata?.version ?? "unknown"}`;
    throw new Error(`npm registry returned ${receivedVersion} for ${packageName}@${version}`);
  }
}

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
