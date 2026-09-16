import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";

const required = ["index.mjs", "index.d.mts", "native-target.mjs", "native.d.ts", "package.json"];
await Promise.all(required.map((file) => access(new URL(`../${file}`, import.meta.url))));

const [declarations, packageJson, releaseConfig, buildNative] = await Promise.all([
  readFile(new URL("../native.d.ts", import.meta.url), "utf8"),
  readFile(new URL("../package.json", import.meta.url), "utf8").then(JSON.parse),
  readFile(new URL("../../../release-please-config.json", import.meta.url), "utf8").then(
    JSON.parse,
  ),
  readFile(new URL("build-native.mjs", import.meta.url), "utf8"),
]);
for (const name of ["Options", "toHtml", "toHtmlBuffer", "toHtmlWithRenderer"]) {
  if (!declarations.includes(name)) {
    throw new Error(`Generated native declarations are missing ${name}`);
  }
}

assert.ok(
  !packageJson.files.some((file) => file.endsWith(".node")),
  "the main package must not ship native binaries",
);
assert.match(
  buildNative,
  /endsWith\(["']-musl["']\)[\s\S]*args\.push\(["']--cross-compile["']\)/,
  "musl targets must use napi-rs cargo-zigbuild cross-compilation",
);
assert.match(
  buildNative,
  /includes\(["']-unknown-linux-gnu["']\)[\s\S]*args\.push\(["']--use-napi-cross["']\)/,
  "GNU Linux targets must use the napi-rs glibc 2.17 cross-toolchain",
);

// One typed entry per manifest, and nothing that writes a version into the
// pnpm lockfile: the sidecar references use the workspace protocol, so the
// release pull request only ever changes the nine `version` fields.
const extraFiles = releaseConfig.packages["."]["extra-files"];
const versioned = (entry) => entry?.type === "json" && entry.jsonpath === "$.version";
assert.ok(
  extraFiles.some((entry) => versioned(entry) && entry.path === "node/ferromark/package.json"),
  "the npm facade version must be versioned by release-please",
);
assert.ok(
  extraFiles.some(
    (entry) =>
      versioned(entry) && entry.glob === true && entry.path === "node/ferromark/npm/*/package.json",
  ),
  "the native package versions must be versioned by release-please",
);
assert.ok(
  !extraFiles.some((entry) => entry?.path?.endsWith("pnpm-lock.yaml")),
  "a lockfile is generated state, not a release template target",
);

for (const triple of packageJson.napi.targets) {
  const target = packageTarget(triple);
  const dependency = `${packageJson.name}-${target.suffix}`;
  assert.equal(
    packageJson.optionalDependencies[dependency],
    "workspace:*",
    `${dependency} must be referenced with the workspace protocol`,
  );

  const platformPackage = JSON.parse(
    await readFile(new URL(`../npm/${target.suffix}/package.json`, import.meta.url), "utf8"),
  );
  assert.equal(platformPackage.name, dependency);
  assert.equal(platformPackage.version, packageJson.version);
  assert.deepEqual(platformPackage.os, [target.os]);
  assert.deepEqual(platformPackage.cpu, [target.cpu]);
  assert.deepEqual(platformPackage.libc, target.libc ? [target.libc] : undefined);
  assert.equal(platformPackage.main, `ferromark.${target.suffix}.node`);
  assert.deepEqual(platformPackage.files, [platformPackage.main, "LICENSE", "LICENSE-MIT"]);
  assert.deepEqual(platformPackage.engines, packageJson.engines);
  assert.equal(platformPackage.publishConfig?.provenance, true);
}

assert.deepEqual(
  Object.keys(packageJson.optionalDependencies).sort(),
  packageJson.napi.targets
    .map((triple) => `${packageJson.name}-${packageTarget(triple).suffix}`)
    .sort(),
  "optional dependencies must exactly match the configured native targets",
);

function packageTarget(triple) {
  const match = triple.match(
    /^(aarch64|x86_64)-(apple|unknown-linux|pc-windows)-(darwin|gnu|musl|msvc)$/,
  );
  assert.ok(match, `Unsupported native target: ${triple}`);
  const [, rustCpu, platform, abi] = match;
  const cpu = rustCpu === "aarch64" ? "arm64" : "x64";
  const os = {
    apple: "darwin",
    "unknown-linux": "linux",
    "pc-windows": "win32",
  }[platform];
  return {
    cpu,
    os,
    libc: abi === "gnu" ? "glibc" : abi === "musl" ? "musl" : undefined,
    suffix: abi === "darwin" ? `${os}-${cpu}` : `${os}-${cpu}-${abi}`,
  };
}
