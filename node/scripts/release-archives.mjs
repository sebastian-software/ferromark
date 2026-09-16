import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { relative, resolve } from "node:path";

import { releaseChannel } from "./release-channel.mjs";

// The archives `verify-pack.mjs` produced are what gets published: the facade
// references its sidecars with the workspace protocol, and only a pnpm-packed
// tarball carries the resolved versions. Everything here is a check on those
// exact files, before the first registry mutation.
export function releaseArchives(directory, facade, { readManifest = archiveManifest } = {}) {
  const { tag } = releaseChannel(facade.version);
  assert.equal(facade.private, false, "the facade must be publishable");
  const names = [...Object.keys(facade.optionalDependencies).sort(), facade.name];
  const expected = names.map((name) => `${name}-${facade.version}.tgz`);
  assert.deepEqual(
    readdirSync(directory)
      .filter((name) => name.endsWith(".tgz"))
      .sort(),
    [...expected].sort(),
    "the archive directory must hold exactly the nine release archives",
  );

  const archives = names.map((name, index) => {
    const archive = resolve(directory, expected[index]);
    const manifest = readManifest(archive);
    assert.equal(manifest.name, name, `${name}: archive manifest name`);
    assert.equal(manifest.version, facade.version, `${name}: archive manifest version`);
    assert.equal(manifest.private, false, `${name}: archive manifest publishability`);
    if (name === facade.name) {
      // pnpm resolved `workspace:*` while packing. An unresolved reference
      // would install as a literal `workspace:*` range for every consumer.
      assert.deepEqual(
        manifest.optionalDependencies,
        Object.fromEntries(
          Object.keys(facade.optionalDependencies).map((sidecar) => [sidecar, facade.version]),
        ),
        "the packed facade must pin every sidecar at the release version",
      );
    }
    return archive;
  });
  // Sidecars first, facade last: the facade is only installable once its
  // optional dependencies exist on the registry.
  return { distTag: tag, archives };
}

/**
 * The archive paths as `npm publish` arguments, relative to `workingDirectory`
 * and always starting with `./`.
 *
 * npm reads a bare `directory/name.tgz` as the GitHub shorthand `owner/repo`
 * and runs `git ls-remote ssh://git@github.com/directory/name.tgz.git`, which
 * is exactly how the first v2.0.0-rc.2 publish attempt failed. A leading `./`
 * is what makes npm treat the argument as a local tarball.
 */
export function publishArguments(archives, workingDirectory) {
  return archives.map((archive) => `./${relative(workingDirectory, archive)}`);
}

function archiveManifest(archive) {
  const result = spawnSync("tar", ["-xOzf", archive, "package/package.json"], { encoding: "utf8" });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `cannot read the manifest of ${archive}`);
  return JSON.parse(result.stdout);
}

if (process.argv[1] === import.meta.filename) {
  const directory = process.argv[2];
  if (!directory || process.argv.length > 3) {
    throw new Error("Usage: node scripts/release-archives.mjs <archive-directory>");
  }
  const facade = JSON.parse(
    readFileSync(new URL("../ferromark/package.json", import.meta.url), "utf8"),
  );
  const { distTag, archives } = releaseArchives(resolve(directory), facade);
  const relative = publishArguments(archives, resolve(directory, ".."));
  if (process.env.GITHUB_OUTPUT) {
    const { appendFileSync } = await import("node:fs");
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `dist-tag=${distTag}\npackages<<PACKAGES_EOF\n${relative.join("\n")}\nPACKAGES_EOF\n`,
    );
  }
  console.log(`Verified ${archives.length} archives for the ${distTag} channel:`);
  for (const archive of relative) console.log(`  ${archive}`);
}
