import { spawnSync } from "node:child_process";

export function verifyGitHubRelease(tag, release) {
  const releaseTagPattern =
    /^v(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-rc\.(?:0|[1-9]\d*))?$/;
  if (!releaseTagPattern.test(tag)) {
    throw new Error("Invalid release tag: " + (tag || "missing"));
  }
  if (release.tagName !== tag || release.isDraft !== false) {
    throw new Error("Tag " + tag + " does not have a published GitHub release");
  }
  const expectsPrerelease = tag.includes("-rc.");
  if (release.isPrerelease !== expectsPrerelease) {
    throw new Error("GitHub release status does not match tag " + tag);
  }
}

if (process.argv[1] === import.meta.filename) {
  const tag = process.env.RELEASE_TAG ?? "";
  if (!tag) throw new Error("RELEASE_TAG is required");

  const result = spawnSync(
    "gh",
    ["release", "view", tag, "--json", "tagName,isDraft,isPrerelease"],
    { encoding: "utf8" },
  );
  if (result.error || result.status !== 0) {
    throw new Error("No readable GitHub release exists for " + tag);
  }
  verifyGitHubRelease(tag, JSON.parse(result.stdout));
  console.log("Verified the published GitHub release for " + tag);
}
