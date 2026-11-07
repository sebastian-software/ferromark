/* eslint-disable security/detect-non-literal-fs-filename -- Both URLs are fixed build inputs and outputs within this repository. */
import { readFile, writeFile } from "node:fs/promises";

// Read the release manifests during the build; browser modules stay inside the site package.
const manifest = JSON.parse(
  await readFile(new URL("../../node/ferromark/package.json", import.meta.url), "utf8"),
);
const cargo = await readFile(new URL("../../Cargo.toml", import.meta.url), "utf8");
const rustVersion = /^rust-version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1];
const nodeVersion = /^>=\s*(\S+)$/.exec(manifest.engines?.node ?? "")?.[1];
if (rustVersion === undefined || nodeVersion === undefined) {
  throw new Error("The release manifests must state a minimum Rust and Node.js version");
}
await writeFile(
  new URL("../app/data/release.json", import.meta.url),
  `${JSON.stringify({ version: manifest.version, rustVersion, nodeVersion }, null, 2)}\n`,
);
