import { appendFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const tag = process.env.RELEASE_TAG ?? "";
const eventName = process.env.GITHUB_EVENT_NAME ?? "";
const eventRef = process.env.GITHUB_REF ?? "";

const releaseTagPattern =
  /^v(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-rc\.(?:0|[1-9]\d*))?$/;

if (eventName !== "push" && eventName !== "workflow_dispatch") {
  throw new Error("Unsupported release event: " + (eventName || "missing"));
}
if (eventRef !== "refs/heads/main") {
  throw new Error("Release publishing must run from refs/heads/main");
}
if (!releaseTagPattern.test(tag)) {
  throw new Error("Invalid release tag: " + (tag || "missing"));
}

fetchRefspec("+refs/heads/main:refs/remotes/origin/main");
fetchRefspec("+refs/tags/" + tag + ":refs/tags/" + tag);

const sourceSha = git(["rev-parse", "refs/tags/" + tag + "^{commit}"])
  .trim()
  .toLowerCase();
if (!/^[0-9a-f]{40}$/.test(sourceSha)) {
  throw new Error("Release tag " + tag + " did not resolve to a full commit SHA");
}

const ancestry = spawnSync(
  "git",
  ["merge-base", "--is-ancestor", sourceSha, "refs/remotes/origin/main"],
  { encoding: "utf8" },
);
if (ancestry.error || ancestry.status !== 0) {
  throw new Error("Release tag " + tag + " does not point to a commit on origin/main");
}

if (!process.env.GITHUB_OUTPUT) {
  throw new Error("GITHUB_OUTPUT is required");
}
appendFileSync(process.env.GITHUB_OUTPUT, "tag_name=" + tag + "\nsource_sha=" + sourceSha + "\n");
console.log("Resolved " + tag + " to " + sourceSha + " on origin/main");

function fetchRefspec(refspec) {
  const result = spawnSync("git", ["fetch", "--force", "--no-tags", "origin", refspec], {
    encoding: "utf8",
  });
  if (result.error || result.status !== 0) {
    throw new Error("Could not resolve release ref " + refspec);
  }
}

function git(args) {
  const result = spawnSync("git", args, { encoding: "utf8" });
  if (result.error || result.status !== 0) {
    throw new Error("git " + args[0] + " failed while resolving release source");
  }
  return result.stdout;
}
