import { ardo } from "ardo/vite";
import { defineConfig } from "vite";

export default defineConfig({
  base: "/",
  plugins: [
    ardo({
      title: "ferromark",
      description: "Native Markdown to HTML for Rust and Node.js, with reproducible benchmarks",
      githubPages: false,
      siteUrl: "https://ferromark.dev",
    }),
  ],
});
