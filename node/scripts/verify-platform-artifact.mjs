import { verifyPlatform } from "ferramenta-native/packages";

import config from "../native-npm.config.mjs";
const [target, ...extra] = process.argv.slice(2);
if (!target || extra.length) throw new Error("Usage: verify-platform-artifact.mjs <target>");
console.log(await verifyPlatform(config, target));
