import { getOxlintConfig } from "eslint-config-setup";
import { defineConfig, type OxlintConfig } from "oxlint";

// eslint-disable-next-line @typescript-eslint/no-unsafe-type-assertion -- getOxlintConfig() is not yet typed against oxlint's own OxlintConfig
const config = getOxlintConfig({ node: true }) as OxlintConfig;

config.ignorePatterns = [
  "**/dist/**",
  "coverage/**",
  "node_modules/**",
  "ferromark/native.d.ts",
  "ferromark/native-target.mjs",
  "ferromark/native-target.d.mts",
];
config.overrides = [
  ...(config.overrides ?? []),
  {
    files: ["ferromark/index.d.mts"],
    rules: {
      // Keep public interfaces for declaration merging and the structure contract.
      "typescript/consistent-type-definitions": "off",
      // Keep the additive prepared-document and native-plugin APIs in the public declaration entry.
      "eslint/max-lines": ["error", { max: 420, skipBlankLines: true, skipComments: true }],
    },
  },
  {
    files: ["ferromark/index.mjs"],
    rules: {
      // AggregateError retains both native lookup failures for diagnostics.
      "preserve-caught-error": "off",
    },
  },
];

export default defineConfig(config);
