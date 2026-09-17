import { readFile } from "node:fs/promises";
import path from "node:path";

import { releaseChannel } from "./release-channel.mjs";

export const MAX_ATTEMPTS = 8;
export const RETRY_DELAY_MS = 15_000;
export const REQUEST_TIMEOUT_MS = 10_000;

export function registryVersionUrl(packageName, version) {
  return `https://registry.npmjs.org/${encodeURIComponent(packageName)}/${encodeURIComponent(version)}`;
}

export function registryPackageUrl(packageName) {
  return `https://registry.npmjs.org/${encodeURIComponent(packageName)}`;
}

const sleep = (milliseconds) =>
  new Promise((resolve) => {
    setTimeout(resolve, milliseconds);
  });

// The retry loop keeps each registry observation and diagnostic in one path.
// eslint-disable-next-line max-statements
export async function verifyNpmPublication({
  packageName,
  version,
  publishResult,
  fetchImpl = fetch,
  sleepImpl = sleep,
}) {
  const registryUrl = registryVersionUrl(packageName, version);
  // The channel a release publishes to is derived from the version alone, so
  // the same rule proves the dist-tag that installs it afterwards.
  const { tag: distTag } = releaseChannel(version);
  let published = false;
  let lastObservation = "the registry did not return the expected package metadata";

  for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt += 1) {
    const observation = await observePublication({
      registryUrl,
      packageName,
      version,
      distTag,
      fetchImpl,
    });
    if (observation.published) {
      published = true;
      console.log(
        `npm registry confirmed ${packageName}@${version} as ${distTag} on attempt ${attempt}`,
      );
      break;
    }
    lastObservation = observation.message;

    console.log(`npm release verification attempt ${attempt}/${MAX_ATTEMPTS}: ${lastObservation}`);
    if (attempt < MAX_ATTEMPTS) {
      await sleepImpl(RETRY_DELAY_MS);
    }
  }

  const failures = [];
  if (publishResult !== "success") {
    failures.push(`publish-npm concluded ${publishResult}`);
  }
  if (!published) {
    failures.push(
      `expected ${packageName}@${version} at ${registryUrl} with dist-tag ${distTag}; ${lastObservation}`,
    );
  }
  if (failures.length > 0) {
    throw new Error(`npm release verification failed: ${failures.join("; ")}`);
  }
}

async function main() {
  const scriptsDirectory = import.meta.dirname;
  const packageJsonPath = path.join(scriptsDirectory, "..", "ferromark", "package.json");
  const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8"));

  if (typeof packageJson.name !== "string" || typeof packageJson.version !== "string") {
    throw new TypeError(`Invalid npm package manifest: ${packageJsonPath}`);
  }

  // All nine packages, sidecars first: a facade that resolves while a sidecar
  // is missing installs for nobody, so a partial release has to fail here.
  for (const name of [...Object.keys(packageJson.optionalDependencies).sort(), packageJson.name]) {
    await verifyNpmPublication({
      packageName: name,
      version: packageJson.version,
      publishResult: process.env.NPM_PUBLISH_RESULT ?? "unknown",
    });
  }
}

if (process.argv[1] === import.meta.filename) {
  try {
    await main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

// A release is only complete when the version exists and its channel points at
// it: `npm install <name>@<tag>` resolves through the tag, not the version.
async function observePublication({ registryUrl, packageName, version, distTag, fetchImpl }) {
  const versionObservation = await observeVersion({ registryUrl, packageName, version, fetchImpl });
  if (!versionObservation.published) {
    return versionObservation;
  }
  return observeDistTag({ packageName, version, distTag, fetchImpl });
}

async function observeVersion({ registryUrl, packageName, version, fetchImpl }) {
  try {
    const response = await fetchImpl(registryUrl, {
      cache: "no-store",
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
    if (!response.ok) {
      return { published: false, message: `registry returned HTTP ${response.status}` };
    }
    const metadata = await response.json();
    if (metadata.name === packageName && metadata.version === version) {
      return { published: true, message: "" };
    }
    return {
      published: false,
      message: `registry returned ${metadata.name ?? "unknown"}@${metadata.version ?? "unknown"}`,
    };
  } catch (error) {
    return { published: false, message: `registry request failed: ${error.message}` };
  }
}

async function observeDistTag({ packageName, version, distTag, fetchImpl }) {
  const packageUrl = registryPackageUrl(packageName);
  try {
    const response = await fetchImpl(packageUrl, {
      cache: "no-store",
      // The abbreviated document carries dist-tags without every version's manifest.
      headers: { accept: "application/vnd.npm.install-v1+json" },
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
    if (!response.ok) {
      return {
        published: false,
        message: `registry returned HTTP ${response.status} for the dist-tags of ${packageName}`,
      };
    }
    const metadata = await response.json();
    const tagged = metadata["dist-tags"]?.[distTag];
    if (tagged === version) {
      return { published: true, message: "" };
    }
    return {
      published: false,
      message: `registry lists ${packageName}@${distTag} as ${tagged ?? "unset"}`,
    };
  } catch (error) {
    return { published: false, message: `registry dist-tag request failed: ${error.message}` };
  }
}
