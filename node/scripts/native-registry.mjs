import assert from "node:assert/strict";

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
