import { ardo } from "ardo/vite";
import { defineConfig } from "vite";

export default defineConfig({
  base: "/",
  plugins: [
    ardo({
      title: "ferromark",
      description: "High-throughput Markdown to HTML parser for Rust",
      githubPages: false,
      siteUrl: "https://ferromark.dev",
    }),
  ],
});
