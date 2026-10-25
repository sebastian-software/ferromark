import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFile, mkdir, mkdtemp, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { parseArgs } from "node:util";

import { collectPluginNotices } from "./plugin-notices.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const source = path.join(root, "node/ferromark");
const { values } = parseArgs({
  options: { plugins: { type: "string" }, output: { type: "string" } },
});
const choices = {
  none: [],
  ferriki: ["ferriki"],
  ferrigura: ["ferrigura"],
  all: ["ferriki", "ferrigura"],
};
if (!Object.hasOwn(choices, values.plugins ?? "") || !values.output) {
  throw new Error(
    "Usage: build-plugin-bundle.mjs --plugins none|ferriki|ferrigura|all --output <new-directory>",
  );
}
const plugins = choices[values.plugins];
const output = path.resolve(values.output);
// A new directory keeps previous results and package artifacts intact.
await mkdir(output);
const temporary = await mkdtemp(path.join(tmpdir(), "ferromark-plugin-bundle-"));
const hash = (value) => createHash("sha256").update(value).digest("hex");
const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    ...options,
  });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`${command} failed:\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
};

try {
  const nativeOutput = path.join(temporary, "native");
  run(process.execPath, [path.join(source, "scripts/build-native.mjs")], {
    env: {
      ...process.env,
      FERROMARK_PGO: "0",
      FERROMARK_NAPI_NO_DEFAULT_FEATURES: "1",
      FERROMARK_NAPI_FEATURES: plugins.join(","),
      FERROMARK_NATIVE_OUTPUT_DIR: nativeOutput,
    },
  });
  const nativeFiles = await readdir(nativeOutput);
  const binary = nativeFiles.filter((name) => name.endsWith(".node"));
  if (binary.length !== 1) throw new Error("Expected exactly one native plugin addon");
  const target = binary[0].slice("ferromark.".length, -".node".length);
  const [platform, arch] = target.split("-");
  const manifest = JSON.parse(await readFile(path.join(source, "package.json"), "utf8"));
  const commit = run("git", ["rev-parse", "HEAD"]).trim();
  const dirty = run("git", ["status", "--porcelain"]).trim().length > 0;
  const version = `${manifest.version}${manifest.version.includes("-") ? "." : "-"}custom.${values.plugins}.${target}.g${commit.slice(0, 12)}${dirty ? ".dirty" : ""}`;
  const directory = path.join(temporary, "package");
  await mkdir(directory);
  for (const file of manifest.files)
    await copyFile(path.join(source, file), path.join(directory, file));
  await copyFile(path.join(nativeOutput, binary[0]), path.join(directory, binary[0]));
  // Only symbols exported by this feature selection belong to its declarations.
  await copyFile(path.join(nativeOutput, "native.d.ts"), path.join(directory, "native.d.ts"));
  const cargoArgs = [
    "metadata",
    "--locked",
    "--format-version",
    "1",
    "--manifest-path",
    path.join(root, "node/native/Cargo.toml"),
    "--no-default-features",
  ];
  if (plugins.length) cargoArgs.push("--features", plugins.join(","));
  const rustTarget =
    process.env.FERROMARK_RUST_TARGET ?? run("rustc", ["-vV"]).match(/^host: (.+)$/m)[1];
  cargoArgs.push("--filter-platform", rustTarget);
  const metadata = JSON.parse(run("cargo", cargoArgs));
  const notices = await collectPluginNotices(metadata, directory, root);
  const nativeBytes = await readFile(path.join(directory, binary[0]));
  const provenance = {
    rustVersion: run("rustc", ["-V"]).trim(),
    nodeVersion: process.version,
    plugins,
    target,
    rustTarget,
    version,
    commit,
    dirty,
    pgo: false,
    binary: { file: binary[0], bytes: nativeBytes.length, sha256: hash(nativeBytes) },
    notices,
    cargoLockSha256: hash(await readFile(path.join(root, "Cargo.lock"))),
  };
  delete manifest.optionalDependencies;
  delete manifest.scripts;
  delete manifest.napi;
  delete manifest.publishConfig;
  manifest.version = version;
  manifest.private = true; // Installable tarball; publishing still uses the reviewed release workflow.
  manifest.os = [platform];
  manifest.cpu = [arch];
  if (platform === "linux") manifest.libc = [target.endsWith("musl") ? "musl" : "glibc"];
  manifest.files = [...manifest.files, binary[0], "THIRD_PARTY_NOTICES", "plugin-build.json"];
  manifest.ferromark = { plugins, target, customBuild: true };
  await writeFile(path.join(directory, "package.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  await writeFile(
    path.join(directory, "plugin-build.json"),
    `${JSON.stringify(provenance, null, 2)}\n`,
  );
  const pack = JSON.parse(
    run(
      process.platform === "win32" ? "npm.cmd" : "npm",
      ["pack", "--ignore-scripts", "--json", "--pack-destination", output],
      {
        cwd: directory,
        shell: process.platform === "win32",
        env: { ...process.env, npm_config_cache: path.join(temporary, "npm-cache") },
      },
    ),
  )[0];
  const archive = await readFile(path.join(output, pack.filename));
  await writeFile(
    path.join(output, "bundle.json"),
    `${JSON.stringify(
      {
        ...provenance,
        archive: {
          file: pack.filename,
          bytes: archive.length,
          unpackedBytes: pack.unpackedSize,
          sha256: hash(archive),
        },
      },
      null,
      2,
    )}\n`,
  );
  process.stdout.write(`${path.join(output, pack.filename)}\n`);
} finally {
  await rm(temporary, { recursive: true, force: true });
}
