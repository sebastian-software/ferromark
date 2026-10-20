import { defineConfig } from "ardo/vite";

import manifest from "../node/ferromark/package.json" with { type: "json" };

// Release metadata is read only by the build config. Documentation runtime
// imports remain inside the package boundary enforced by Ardo 6.
export default defineConfig({
  define: { __FERROMARK_VERSION__: JSON.stringify(manifest.version) },
});
