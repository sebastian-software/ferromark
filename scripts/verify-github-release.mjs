import { execFileSync } from "node:child_process";
import { verifyGitHubRelease } from "../node/scripts/release-source.mjs";
export { verifyGitHubRelease } from "../node/scripts/release-source.mjs";
if (process.argv[1] === import.meta.filename) {
  const tag = process.env.RELEASE_TAG;
  const release = JSON.parse(
    execFileSync("gh", ["release", "view", tag, "--json", "tagName,isDraft,isPrerelease"], {
      encoding: "utf8",
    }),
  );
  verifyGitHubRelease(tag, release);
}
