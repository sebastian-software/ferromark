import { spawnSync } from "node:child_process";
import { mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
const workspace = path.resolve(import.meta.dirname, "..");
const artifacts = path.join(workspace, "artifacts");
const artifactFiles = await readdir(artifacts);
const archives = artifactFiles.filter((file) => file.endsWith(".tgz"));
const target = nativeTarget();
const mainArchive = archives.find((file) => /^ferromark-\d/.test(file));
const platformArchive = archives.find((file) => file.startsWith(`ferromark-${target}-`));

if (!mainArchive || !platformArchive) {
  throw new Error(
    `Expected main and ${target} archives, found: ${archives.toSorted().join(", ") || "none"}`,
  );
}

const consumer = await mkdtemp(path.join(tmpdir(), "ferromark-consumer-"));
try {
  await writeFile(
    path.join(consumer, "package.json"),
    JSON.stringify({ private: true, type: "module" }),
  );
  const install = spawnSync(
    "npm",
    [
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--omit=optional",
      path.join(artifacts, mainArchive),
      path.join(artifacts, platformArchive),
    ],
    {
      cwd: consumer,
      encoding: "utf8",
      env: { ...process.env, npm_config_cache: path.join(tmpdir(), "ferromark-npm-cache") },
    },
  );
  if (install.status !== 0) {
    process.stderr.write(install.stderr);
    process.exit(install.status ?? 1);
  }

  const smoke = `
    import { toHtml } from 'ferromark'
    const html = toHtml('# Clean install')
    if (html !== '<h1 id="clean-install">Clean install</h1>\\n') {
      throw new Error('Unexpected clean-install output: ' + html)
    }
  `;
  await writeFile(path.join(consumer, "smoke.mjs"), smoke);
  const run = spawnSync(process.execPath, ["smoke.mjs"], {
    cwd: consumer,
    encoding: "utf8",
  });
  if (run.status !== 0) {
    process.stderr.write(run.stderr);
    process.exit(run.status ?? 1);
  }

  const installedManifest = await readFile(
    path.join(consumer, "node_modules/ferromark/package.json"),
    "utf8",
  );
  const installed = JSON.parse(installedManifest);
  const mainFiles = await readdir(path.join(consumer, "node_modules/ferromark"));
  if (mainFiles.some((file) => file.endsWith(".node"))) {
    throw new Error("The installed main package unexpectedly contains a native binary");
  }
  await readFile(
    path.join(consumer, "node_modules", `ferromark-${target}`, `ferromark.${target}.node`),
  );
  console.log(`Clean install passed for ferromark ${installed.version}`);
} finally {
  await rm(consumer, { force: true, recursive: true });
}

function nativeTarget() {
  const base = `${process.platform}-${process.arch}`;
  if (process.platform !== "linux") {
    return base === "win32-arm64" || base === "win32-x64" ? `${base}-msvc` : base;
  }
  const report = process.report?.getReport?.();
  return `${base}-${report?.header?.glibcVersionRuntime ? "gnu" : "musl"}`;
}
