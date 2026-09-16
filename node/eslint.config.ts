import { getEslintConfig } from "eslint-config-setup";

const config = await getEslintConfig({ node: true, oxlint: true });

config.unshift({
  ignores: [
    "**/dist/**",
    "coverage/**",
    "node_modules/**",
    "pnpm-lock.yaml",
    "**/*.json",
    "**/*.md",
    "ferromark/native.d.ts",
  ],
});

config.push({
  name: "ferromark/local-project",
  files: ["**/*.{ts,mts,cts}"],
  languageOptions: {
    parserOptions: {
      project: "./tsconfig.eslint.json",
      projectService: false,
    },
  },
});

// The package typecheck intentionally covers the public declarations and type
// contract. JavaScript sources are still linted with the shared rules below.
config.push({
  name: "ferromark/javascript-project",
  files: ["**/*.{cjs,js,mjs}"],
  languageOptions: {
    parserOptions: {
      project: false,
      projectService: false,
    },
  },
});

// Public declaration interfaces are intentionally preserved for declaration
// merging and are checked by the repository's declaration structure contract.
// Explicit Buffer types resolve even when consumers disable ambient Node globals.
config.push({
  name: "ferromark/public-declarations",
  files: ["ferromark/index.d.mts"],
  rules: {
    "@typescript-eslint/consistent-type-definitions": "off",
    "perfectionist/sort-union-types": "off",
    "node/prefer-global/buffer": "off",
  },
});

// JSDoc in the runtime loader mirrors the checked JavaScript API types; the
// shared JSDoc rules do not understand those import() annotations.
config.push({
  name: "ferromark/runtime-jsdoc",
  files: ["ferromark/index.mjs", "ferromark/native-target.mjs"],
  rules: {
    "jsdoc/escape-inline-tags": "off",
    "jsdoc/no-types": "off",
    "jsdoc/no-undefined-types": "off",
  },
});

// Release and package contracts inspect repository paths and generated
// manifests; their filenames are trusted local inputs rather than user data.
config.push({
  name: "ferromark/local-contract-paths",
  files: ["scripts/**/*.mjs", "ferromark/scripts/**/*.mjs", "ferromark/test/**/*.mjs"],
  rules: {
    "security/detect-non-literal-fs-filename": "off",
    "security/detect-non-literal-regexp": "off",
  },
});

// The loader must pass a target-derived native filename to require() so it can
// select the platform package; the target comes from the fixed native table.
config.push({
  name: "ferromark/native-loader",
  files: ["ferromark/index.mjs"],
  rules: {
    "security/detect-non-literal-require": "off",
  },
});

export default config;
