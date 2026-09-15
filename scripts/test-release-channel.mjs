import assert from "node:assert/strict";
import { it } from "node:test";
import { releaseChannel } from "../node/scripts/release-channel.mjs";
import { publishArguments } from "../node/scripts/publish-packages.mjs";

it("publishes release candidates only to next and marks them prereleases", () => {
  assert.deepEqual(releaseChannel("2.0.0-rc.1"), { tag: "next", prerelease: true });
  assert.deepEqual(publishArguments("package.tgz", "2.0.0-rc.1"), [
    "publish",
    "package.tgz",
    "--access",
    "public",
    "--provenance",
    "--tag",
    "next",
  ]);
});
it("publishes stable versions to latest", () => {
  assert.deepEqual(releaseChannel("2.0.0"), { tag: "latest", prerelease: false });
});
it("rejects development versions and malformed release versions", () => {
  for (const version of [
    "2.0.0-dev.0",
    "v2.0.0",
    "2.0",
    "2.0.0-rc.01",
    "02.0.0",
    "2.0.0\n",
    "2.0.0-rc.1 --tag latest",
  ]) {
    assert.throws(() => releaseChannel(version));
  }
});
