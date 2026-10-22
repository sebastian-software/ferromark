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
    // MDX code blocks are documentation snippets, not project sources; they
    // have no tsconfig entry for the type-aware rules to use.
    "**/*.mdx/**",
  ],
});

export default config;
