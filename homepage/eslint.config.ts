import { getEslintConfig } from "eslint-config-setup";

const config = await getEslintConfig({ node: true, oxlint: true });

config.unshift({
  ignores: [
    "**/dist/**",
    "build/**",
    ".react-router/**",
    "app/routes.ts",
    "coverage/**",
    "node_modules/**",
    "pnpm-lock.yaml",
    "**/*.json",
    "**/*.md",
  ],
});

// Fenced consumer examples describe imports installed by the reader. Ardo's
// package-boundary check still validates actual MDX runtime dependencies.
config.push({
  files: ["app/routes/**/*.mdx/**"],
  rules: { "import/no-extraneous-dependencies": "off" },
});

export default config;
