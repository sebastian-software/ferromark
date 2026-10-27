import { appendFileSync } from "node:fs";
import config from "../node/native-npm.config.mjs";
import { isLegacyReleaseVersion, resolveReleaseSource } from "../node/scripts/release-source.mjs";
const { tag, sourceSha, version } = resolveReleaseSource({
  tag: process.env.RELEASE_TAG,
  legacyRecoveryThrough: config.release.legacyRecoveryThrough,
});
const historicalRetry =
  process.env.GITHUB_EVENT_NAME === "workflow_dispatch" &&
  process.env.GITHUB_REF === "refs/heads/main" &&
  isLegacyReleaseVersion(config, version);
appendFileSync(
  process.env.GITHUB_OUTPUT,
  `tag_name=${tag}\nsource_sha=${sourceSha}\nrequire_ci=${!historicalRetry}\n`,
);
