import { readFile, stat } from "node:fs/promises";
import path from "node:path";

const options = parseOptions(process.argv.slice(2));
const root = path.resolve(options.root ?? path.resolve(import.meta.dirname, "../.."));
const workspace = path.join(root, "node");
const packageDir = path.join(workspace, "ferromark");
const packageJson = JSON.parse(await readFile(path.join(packageDir, "package.json"), "utf8"));
const cargo = await readFile(path.join(root, "Cargo.toml"), "utf8");
const cargoVersion = cargo.match(/^version = "([^"]+)"/m)?.[1];

if (!cargoVersion || cargoVersion !== packageJson.version) {
  throw new Error(
    `Rust/npm version mismatch: Cargo=${cargoVersion ?? "missing"}, npm=${packageJson.version}`,
  );
}

const transformsCargo = await readFile(path.join(root, "transforms/Cargo.toml"), "utf8");
const nativeCargo = await readFile(path.join(root, "node/native/Cargo.toml"), "utf8");
for (const [name, manifest] of [
  ["ferromark-transforms", transformsCargo],
  ["ferromark-node", nativeCargo],
]) {
  const version = manifest.match(/^version = "([^"]+)"/m)?.[1];
  if (version !== packageJson.version) {
    throw new Error(
      `${name} version mismatch: Cargo=${version ?? "missing"}, npm=${packageJson.version}`,
    );
  }
}

const transformCoreRequirement = transformsCargo.match(
  /^ferromark = \{ version = "([^"]+)", path = "\.\." \}$/m,
)?.[1];
if (transformCoreRequirement !== `=${packageJson.version}`) {
  throw new Error("Transform crate does not pin the matching core version");
}

const targets = Object.keys(packageJson.optionalDependencies).map((name) => {
  const prefix = `${packageJson.name}-`;
  if (!name.startsWith(prefix)) throw new Error(`Unexpected platform package: ${name}`);
  return name.slice(prefix.length);
});
if (targets.length !== 7 || new Set(targets).size !== targets.length) {
  throw new Error(`Expected seven unique platform packages, found ${targets.length}`);
}

for (const target of targets) {
  const platformDir = path.join(packageDir, "npm", target);
  const platformPackage = JSON.parse(
    await readFile(path.join(platformDir, "package.json"), "utf8"),
  );
  if (platformPackage.name !== `${packageJson.name}-${target}`) {
    throw new Error(`Unexpected platform package name: ${platformPackage.name}`);
  }
  if (platformPackage.version !== packageJson.version) {
    throw new Error(
      `Platform package version mismatch: ${platformPackage.name}@${platformPackage.version}`,
    );
  }
  // The workspace reference resolves to the sidecar version while packing.
  if (packageJson.optionalDependencies[platformPackage.name] !== "workspace:*") {
    throw new Error(`Main package does not reference ${platformPackage.name} as workspace:*`);
  }
  if (!options.metadataOnly) {
    const file = path.join(platformDir, `ferromark.${target}.node`);
    const info = await stat(file);
    if (!info.isFile() || info.size === 0) {
      throw new Error(`Invalid native binary: ${file}`);
    }
  }
}

if (
  options.tag &&
  (!isCanonicalReleaseVersion(packageJson.version) || options.tag !== `v${packageJson.version}`)
) {
  throw new Error(
    `Release tag ${options.tag} does not match package version ${packageJson.version}`,
  );
}

console.log(
  `Verified ferromark ${packageJson.version} with ${targets.length} platform packages${options.metadataOnly ? " (metadata only)" : ""}`,
);

function parseOptions(args) {
  const options = { metadataOnly: false };
  for (let index = 0; index < args.length; index++) {
    const value = args[index];
    if (value === "--metadata-only") {
      options.metadataOnly = true;
      continue;
    }
    if (value === "--root" || value === "--tag") {
      options[value.slice(2)] = requiredOptionValue(args, ++index, value);
      continue;
    }
    throw new Error(`Unknown argument: ${value}`);
  }
  if (options.metadataOnly && !options.tag) {
    throw new Error("--metadata-only requires --tag");
  }
  return options;
}

function requiredOptionValue(args, index, option) {
  const value = args[index];
  if (!value) throw new Error(`${option} requires a value`);
  return value;
}

function isCanonicalReleaseVersion(version) {
  const [core, rc, extra] = version.split("-rc.");
  if (extra !== undefined) return false;
  const parts = core.split(".");
  return (
    parts.length === 3 &&
    parts.every((part) => isCanonicalNumber(part)) &&
    (rc === undefined || isCanonicalNumber(rc))
  );
}

function isCanonicalNumber(value) {
  if (value.length === 0 || (value.length > 1 && value[0] === "0")) return false;
  for (const digit of value) {
    if (digit < "0" || digit > "9") return false;
  }
  return true;
}
