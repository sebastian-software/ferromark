import { verifyPackedConsumer } from "ferramenta-native";

import config from "../native-npm.config.mjs";
await verifyPackedConsumer(
  config,
  'import { toHtml } from "ferromark"; if (toHtml("# Native") !== \'<h1 id="native">Native</h1>\\n\') throw new Error("HTML mismatch");',
);
