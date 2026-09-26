const requirementPattern =
  /^(ferromark\s*=\s*\{\s*version\s*=\s*")([^"]+)("\s*,\s*path\s*=\s*"\.\."\s*\})$/m;

export function preserveExactTransformPin(rootCargoToml, transformsCargoToml) {
  const rootVersion = rootCargoToml.match(/^\[package\][\s\S]*?^version = "([^"]+)"$/m)?.[1];
  if (!rootVersion) throw new Error("Could not read the root ferromark package version");
  const transformPackageVersion = transformsCargoToml.match(
    /^\[package\][\s\S]*?^version = "([^"]+)"$/m,
  )?.[1];
  if (transformPackageVersion !== rootVersion) {
    throw new Error(
      `The transform crate is ${transformPackageVersion ?? "missing a version"}, but the root package is ${rootVersion}`,
    );
  }

  const match = transformsCargoToml.match(requirementPattern);
  if (!match)
    throw new Error("Could not find the ferromark path dependency in transforms/Cargo.toml");

  const currentVersion = match[2];
  const unprefixedVersion = currentVersion.startsWith("=")
    ? currentVersion.slice(1)
    : currentVersion;
  if (unprefixedVersion !== rootVersion) {
    throw new Error(
      `The transform crate requires ferromark ${currentVersion}, but the root package is ${rootVersion}`,
    );
  }

  if (currentVersion === `=${rootVersion}`) {
    return { content: transformsCargoToml, changed: false, version: rootVersion };
  }

  const content = transformsCargoToml.replace(requirementPattern, `$1=${rootVersion}$3`);
  return { content, changed: true, version: rootVersion };
}
