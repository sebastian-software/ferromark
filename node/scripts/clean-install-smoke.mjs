import { fileURLToPath } from "node:url";
import { runNativeCommand } from "ferramenta-native/cli";

import config from "../native-npm.config.mjs";
await runNativeCommand(config, [
  "consumer",
  "--directory",
  fileURLToPath(new URL("../artifacts", import.meta.url)),
  ...process.argv.slice(2),
]);
