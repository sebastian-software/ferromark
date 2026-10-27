import { runNativeCommand } from "ferramenta-native/cli";

import config from "../native-npm.config.mjs";
await runNativeCommand(config, ["metadata", ...process.argv.slice(2)]);
