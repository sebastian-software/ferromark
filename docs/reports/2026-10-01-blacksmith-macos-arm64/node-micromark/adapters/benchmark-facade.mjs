// Keep the public JS API byte-for-byte; only private Intel-Mac target loading
// differs because the published facade has no darwin-x64 sidecar. All copying
// and module loading happen before verification/timing.
import { copyFileSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { benchmarkTarget } from "./benchmark-target.mjs";

export async function loadBenchmarkFacade(platform = process.platform, arch = process.arch) {
  const facade = new URL("../../node/ferromark/index.mjs", import.meta.url);
  if (platform !== "darwin" || arch !== "x64") return import(facade.href);
  const target = benchmarkTarget(platform, arch);
  const directory = mkdtempSync(path.join(tmpdir(), "ferromark-benchmark-facade-"));
  process.on("exit", () => rmSync(directory, { recursive: true, force: true }));
  copyFileSync(fileURLToPath(facade), path.join(directory, "index.mjs"));
  const originalTarget = new URL("../../node/ferromark/native-target.mjs", import.meta.url).href;
  writeFileSync(
    path.join(directory, "native-target.mjs"),
    `export { linuxLibc } from ${JSON.stringify(originalTarget)};\n` +
      `import { nativeTarget as original } from ${JSON.stringify(originalTarget)};\n` +
      `export function nativeTarget(platform, arch, libc) { return platform === "darwin" && arch === "x64" ? "darwin-x64" : original(platform, arch, libc); }\n`,
  );
  symlinkSync(
    fileURLToPath(new URL(`../../node/ferromark/ferromark.${target}.node`, import.meta.url)),
    path.join(directory, `ferromark.${target}.node`),
  );
  return import(pathToFileURL(path.join(directory, "index.mjs")).href);
}
