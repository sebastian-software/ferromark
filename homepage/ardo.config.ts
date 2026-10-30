import { defineArdoConfig } from "ardo/config";

export default defineArdoConfig({
  title: "Ferromark",
  description: "A native Markdown engine for Rust and Node.js, with reproducible benchmarks.",
  siteUrl: "https://ferromark.dev",
  icons: false,
  markdown: { headingAttributes: true },
  linkCheck: { level: "error" },
});
