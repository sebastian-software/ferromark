import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { copyFile, mkdir, readdir, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";

const pnpm = process.platform === "win32" ? "pnpm.cmd" : "pnpm";
const exe = process.platform === "win32" ? ".exe" : "";
const outputDir = process.env.FERROMARK_NATIVE_OUTPUT_DIR ?? ".";
const packageDir = fileURLToPath(new URL("..", import.meta.url));
const repoRoot = path.resolve(packageDir, "..", "..");
const buildOutputDir = outputDir === "." ? ".napi-artifacts" : outputDir;

// The frozen benchmark corpus: 57 real documents plus link-scan diagnostics.
const corpusArchive = path.join(
  repoRoot,
  "docs",
  "reports",
  "2026-09-14-simd-round",
  "corpus.json.gz",
);
// Per corpus document, per option combination and per lifecycle. 72 documents
// times four combinations times two lifecycles keeps the training run well
// inside a minute on a CI runner.
const trainingMilliseconds = 30;

if (outputDir === ".") {
  await rm(path.join(packageDir, buildOutputDir), { force: true, recursive: true });
}

const args = [
  "exec",
  "napi",
  "build",
  "--platform",
  "--profile",
  "release-node",
  "--manifest-path",
  "../native/Cargo.toml",
  "--dts",
  "native.d.ts",
  "--no-js",
  "--output-dir",
  buildOutputDir,
];

if (process.env.FERROMARK_RUST_TARGET) {
  args.push("--target", process.env.FERROMARK_RUST_TARGET);
  if (process.env.FERROMARK_RUST_TARGET.includes("-unknown-linux-gnu")) {
    args.push("--use-napi-cross");
  } else if (process.env.FERROMARK_RUST_TARGET.endsWith("-musl")) {
    args.push("--cross-compile");
  }
}

if (process.env.FERROMARK_NAPI_FEATURES) {
  args.push("--features", process.env.FERROMARK_NAPI_FEATURES);
}

args.push("--", "--locked");

// Profile-guided optimization is opt-in: local builds stay fast by default and
// CI sets FERROMARK_PGO=1 for the eight published addons. See
// docs/arch/ADR-0019-profile-guided-native-addon.md.
const buildEnv = { ...process.env };
if (process.env.FERROMARK_PGO === "1") {
  const profile = await collectProfile();
  setRustflags(buildEnv, `-Cprofile-use=${profile} -Cllvm-args=-pgo-warn-missing-function`);
  console.log(`PGO: applying ${profile} to the addon build`);
} else {
  console.log("PGO: disabled (set FERROMARK_PGO=1 to build a profile-guided addon)");
}

const result = spawnSync(pnpm, args, {
  cwd: new URL("..", import.meta.url),
  env: buildEnv,
  // Windows command shims such as pnpm.cmd require cmd.exe for spawning.
  shell: process.platform === "win32",
  stdio: "inherit",
});

if (result.error) {
  throw result.error;
}
if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

// Normal builds keep the local addon for development and copy it into the
// matching optional package. Isolated verification builds use a temporary
// output directory and must not modify package artifacts.
if (outputDir === ".") {
  await copyFile(
    path.join(packageDir, buildOutputDir, "native.d.ts"),
    path.join(packageDir, "native.d.ts"),
  );
  const artifacts = spawnSync(
    pnpm,
    ["exec", "napi", "artifacts", "--output-dir", buildOutputDir, "--npm-dir", "npm"],
    {
      cwd: new URL("..", import.meta.url),
      shell: process.platform === "win32",
      stdio: "inherit",
    },
  );
  if (artifacts.error) {
    throw artifacts.error;
  }
  await rm(path.join(packageDir, buildOutputDir), { force: true, recursive: true });
  process.exit(artifacts.status ?? 1);
}

/**
 * Builds the instrumented training binary, runs it over the frozen benchmark
 * corpus and merges the raw counters. Returns the absolute merged profile path.
 */
// Profile collection must preserve the exact build, run, and merge sequence.
// eslint-disable-next-line max-statements
async function collectProfile() {
  const host = hostTriple();
  const profdata = llvmProfdata(host);
  const requested = process.env.FERROMARK_RUST_TARGET;

  // A profile only applies to code whose Cargo unit hash matches, and that hash
  // covers the Cargo profile and the target triple. The training build must
  // therefore use `release-node` and `--target`, exactly as `napi build` always
  // does. It cannot use a cross-compiled triple, because the instrumented binary
  // has to run on this machine.
  const workDir = path.join(repoRoot, "target", "pgo");
  const profrawDir = path.join(workDir, "profraw");
  const corpusDir = path.join(workDir, "corpus");
  const instrumentedDir = path.join(workDir, "instrumented");
  await rm(workDir, { force: true, recursive: true });
  await mkdir(profrawDir, { recursive: true });
  await mkdir(corpusDir, { recursive: true });

  const cargoArgs = [
    "build",
    "--profile",
    "release-node",
    "--bin",
    "pgo_train",
    "-p",
    "ferromark-node",
    "--features",
    "pgo-train",
    "--locked",
    "--target",
    host,
  ];
  const instrumentedEnv = { ...process.env, CARGO_TARGET_DIR: instrumentedDir };
  setRustflags(instrumentedEnv, `-Cprofile-generate=${profrawDir}`);
  console.log(`PGO: building the instrumented training binary for ${host}`);
  run("cargo", cargoArgs, { cwd: repoRoot, env: instrumentedEnv });

  const documents = await writeCorpus(corpusDir);
  const binary = path.join(instrumentedDir, host, "release-node", `pgo_train${exe}`);
  console.log(`PGO: training on ${documents} corpus documents, ${trainingMilliseconds} ms each`);
  run(binary, [corpusDir, String(trainingMilliseconds)]);

  const profileFiles = await readdir(profrawDir);
  const raw = profileFiles.filter((name) => name.endsWith(".profraw"));
  if (raw.length === 0) {
    throw new Error(`The training run produced no profile data in ${profrawDir}`);
  }
  const merged = path.join(workDir, "merged.profdata");
  run(profdata, ["merge", "-o", merged, ...raw.map((name) => path.join(profrawDir, name))]);
  const { size } = await stat(merged);
  console.log(`PGO: merged ${raw.length} raw profile(s) into ${merged} (${size} bytes)`);
  if (requested && requested !== host) {
    console.log(
      `PGO: ${requested} is cross-compiled from ${host}. ` +
        "The unit hash of a profile collected on the host does not match a different " +
        "triple, so those functions build without profile data.",
    );
  } else {
    console.log(`PGO: the profile was collected for ${host}, the triple this addon is built for`);
  }
  return merged;
}

/** Expands the frozen benchmark corpus into one Markdown file per case. */
async function writeCorpus(corpusDir) {
  const corpus = JSON.parse(gunzipSync(readFileSync(corpusArchive)).toString("utf8"));
  await Promise.all(
    corpus.cases.map((entry) => {
      if (!/^[\w-]+$/.test(entry.name)) {
        throw new Error(`Unexpected corpus case name: ${entry.name}`);
      }
      return writeFile(path.join(corpusDir, `${entry.name}.md`), entry.input);
    }),
  );
  return corpus.cases.length;
}

function hostTriple() {
  const host = capture("rustc", ["-vV"])
    .match(/^host: (.+)$/m)?.[1]
    ?.trim();
  if (!host) {
    throw new Error("Cannot determine the host target triple from `rustc -vV`");
  }
  return host;
}

function llvmProfdata(host) {
  const sysroot = capture("rustc", ["--print", "sysroot"]).trim();
  const tool = path.join(sysroot, "lib", "rustlib", host, "bin", `llvm-profdata${exe}`);
  if (!existsSync(tool)) {
    throw new Error(
      `FERROMARK_PGO=1 needs llvm-profdata, which is missing at ${tool}. ` +
        "Run `rustup component add llvm-tools` for this toolchain, or add " +
        "`components: llvm-tools` to the CI rust-toolchain step.",
    );
  }
  return tool;
}

/**
 * Extends the inherited Rust flags instead of replacing them. Cargo reads
 * CARGO_ENCODED_RUSTFLAGS first, then RUSTFLAGS, then CARGO_BUILD_RUSTFLAGS,
 * and any of them replaces `build.rustflags` and `[target.*].rustflags` from a
 * Cargo configuration file, so a configuration file with those keys has to be
 * merged by hand rather than silently dropped.
 */
function setRustflags(env, flags) {
  for (const directory of [
    repoRoot,
    path.join(repoRoot, "node"),
    path.join(repoRoot, "node", "native"),
    packageDir,
  ]) {
    for (const name of ["config.toml", "config"]) {
      assertNoCargoRustflags(path.join(directory, ".cargo", name));
    }
  }
  const encoded = process.env.CARGO_ENCODED_RUSTFLAGS;
  const inherited =
    encoded === undefined
      ? (process.env.RUSTFLAGS ?? process.env.CARGO_BUILD_RUSTFLAGS ?? "")
      : encoded.split("\u001F").filter(Boolean).join(" ");
  delete env.CARGO_ENCODED_RUSTFLAGS;
  env.RUSTFLAGS = `${inherited} ${flags}`.trim();
}

function assertNoCargoRustflags(file) {
  if (!existsSync(file)) return;
  const hasRustflags = readFileSync(file, "utf8")
    .split("\n")
    .some((line) => line.trimStart().startsWith("rustflags") && line.includes("="));
  if (hasRustflags) {
    throw new Error(
      `${file} sets rustflags; merge them into build-native.mjs before building with FERROMARK_PGO=1`,
    );
  }
}

function run(command, commandArgs, options = {}) {
  const child = spawnSync(command, commandArgs, { stdio: "inherit", ...options });
  if (child.error) {
    throw child.error;
  }
  if (child.status !== 0) {
    throw new Error(`${command} exited with status ${child.status}`);
  }
}

function capture(command, commandArgs) {
  const child = spawnSync(command, commandArgs, { encoding: "utf8" });
  if (child.error) {
    throw child.error;
  }
  if (child.status !== 0) {
    throw new Error(`${command} exited with status ${child.status}: ${child.stderr}`);
  }
  return child.stdout;
}
