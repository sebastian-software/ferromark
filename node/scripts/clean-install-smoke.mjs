import { spawnSync } from "node:child_process";
import { copyFile, mkdir, mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
const workspace = path.resolve(import.meta.dirname, "..");
const artifacts = path.join(workspace, "artifacts");
const packageTestFile = path.join(workspace, "ferromark", "test", "index.test.mjs");
const args = process.argv.slice(2);
const usage = `Usage: node ${path.basename(import.meta.filename)} [--target <platform-target>] [--package-tests]`;
// `--target` names the platform package this run must install, so a musl
// container cannot quietly verify a glibc addon; `--package-tests` also runs
// the package test suite against the installed copy.
const packageTests = args.includes("--package-tests");
const selection = args.filter((argument) => argument !== "--package-tests");
const requestedTarget = selection.length > 0 ? selection[1] : undefined;

if (
  selection.length > 0 &&
  (selection.length !== 2 || selection[0] !== "--target" || !/^[a-z0-9-]+$/.test(selection[1]))
) {
  throw new Error(usage);
}

const target = nativeTarget();

// The installed addon is loaded here, so a requested target is an assertion
// about this host rather than a way to exercise a foreign binary.
if (requestedTarget && requestedTarget !== target) {
  throw new Error(`Requested ${requestedTarget}, but this host loads ferromark-${target}`);
}

const artifactFiles = await readdir(artifacts);
const archives = artifactFiles.filter((file) => file.endsWith(".tgz"));
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
  if (packageTests) {
    runPackageTests(await installPackageTests(consumer));
  }

  console.log(
    `Clean install passed for ferromark ${installed.version} on ${target}${packageTests ? ", package tests included" : ""}`,
  );
} finally {
  await rm(consumer, { force: true, recursive: true });
}

/**
 * Place the package test suite inside the installed package.
 *
 * The suite imports the facade as `../index.mjs` and resolves `ferromark` by
 * name, so running it from the installed package directory points every
 * assertion at the installed loader and the installed native sidecar instead of
 * the workspace build.
 *
 * @param consumerDir Consumer directory holding the installation.
 * @returns The installed package directory the tests run in.
 */
async function installPackageTests(consumerDir) {
  const installedPackage = path.join(consumerDir, "node_modules", "ferromark");
  await mkdir(path.join(installedPackage, "test"), { recursive: true });
  await copyFile(packageTestFile, path.join(installedPackage, "test", "index.test.mjs"));
  return installedPackage;
}

/** @param installedPackage Installed package directory the tests run in. */
function runPackageTests(installedPackage) {
  const result = spawnSync(process.execPath, ["--test", "test/index.test.mjs"], {
    cwd: installedPackage,
    stdio: "inherit",
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function nativeTarget() {
  const base = `${process.platform}-${process.arch}`;
  if (process.platform !== "linux") {
    return base === "win32-arm64" || base === "win32-x64" ? `${base}-msvc` : base;
  }
  const report = process.report?.getReport?.();
  return `${base}-${report?.header?.glibcVersionRuntime ? "gnu" : "musl"}`;
}
