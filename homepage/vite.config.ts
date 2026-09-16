import { ardo } from "ardo/vite";
import { defineConfig } from "vite";

export default defineConfig({
  base: "/ferromark/",
  plugins: [
    ardo({
      title: "ferromark",
      description: "High-throughput Markdown to HTML parser for Rust",
      githubPages: false,
    }),
  ],
});
