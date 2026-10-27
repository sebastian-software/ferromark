import path from "node:path";
const root = import.meta.dirname;
export default {
  name: "ferromark",
  packageDir: path.join(root, "ferromark"),
  platformDir: path.join(root, "ferromark/npm"),
  packagePattern: "ferromark-{target}",
  binaryPattern: "ferromark.{target}.node",
  runtimeFile: "native-target.mjs",
  repositoryPlatformDir: "node/ferromark/npm",
  licenses: ["LICENSE", "LICENSE-MIT"],
  workspaceProtocol: true,
};
