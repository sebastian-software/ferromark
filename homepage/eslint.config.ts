import { getEslintConfig } from "eslint-config-setup";

const config = await getEslintConfig({ node: true, oxlint: true });

// Fenced documentation examples describe consumer packages rather than site dependencies.
config.push({
  files: ["app/routes/**/*.mdx", "app/routes/**/*.mdx/**"],
  rules: { "import/no-extraneous-dependencies": "off" },
});

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

export default config;
