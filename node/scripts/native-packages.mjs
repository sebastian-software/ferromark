import { generatePackages, generateRuntime } from "ferramenta-native";

import config from "../native-npm.config.mjs";
const check = process.argv.includes("--check");
await generatePackages(config, { check });
await generateRuntime(config, { check });
