import { runNativeCommand } from "ferramenta-native/cli";

import config from "../native-npm.config.mjs";
await runNativeCommand(config, ["pack", ...process.argv.slice(2)]);
