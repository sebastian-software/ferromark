/* eslint-disable security/detect-non-literal-fs-filename -- Both URLs are fixed build inputs and outputs within this repository. */
import { readFile, writeFile } from "node:fs/promises";

// Read the release manifest during the build; browser modules stay inside the site package.
const { version } = JSON.parse(
  await readFile(new URL("../../node/ferromark/package.json", import.meta.url), "utf8"),
);
await writeFile(
  new URL("../app/data/release.json", import.meta.url),
  `${JSON.stringify({ version }, null, 2)}\n`,
);
