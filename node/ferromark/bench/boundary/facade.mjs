// Loads the package facade (index.mjs) against the diagnostic addon, so the
// facade lanes time the real JavaScript entries on the same binary as every
// other lane.

import { copyFileSync, mkdtempSync, rmSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

/**
 * Copies the facade from `facadeDir` into a temporary directory next to a link
 * to the addon. The facade loads `ferromark.<target>.node` from its own
 * directory first, which is the name `napi build` gives the addon, and Node.js
 * caches addons by their real path, so the facade shares the script's addon.
 * @param addon The loaded addon: its file and its exports.
 * @param facadeDir The directory holding index.mjs and native-target.mjs.
 * @returns The facade module.
 */
export async function loadFacade(addon, facadeDir) {
  const directory = mkdtempSync(path.join(tmpdir(), "ferromark-boundary-facade-"));
  process.on("exit", () => rmSync(directory, { force: true, recursive: true }));
  for (const file of ["index.mjs", "native-target.mjs"]) {
    copyFileSync(path.join(facadeDir, file), path.join(directory, file));
  }
  symlinkSync(addon.file, path.join(directory, path.basename(addon.file)));
  const facade = await import(pathToFileURL(path.join(directory, "index.mjs")).href);
  assertSharedAddon(facade, addon.native, facadeDir);
  return facade;
}

// The facade must call into the addon the script loaded, not another copy.
function assertSharedAddon(facade, native, facadeDir) {
  const { toHtml } = native;
  let called = false;
  native.toHtml = (...args) => {
    called = true;
    return toHtml(...args);
  };
  try {
    facade.toHtml("");
  } finally {
    native.toHtml = toHtml;
  }
  if (!called) {
    throw new Error(`The facade in ${facadeDir} did not load the diagnostic addon`);
  }
}
