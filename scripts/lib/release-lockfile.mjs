// Release Please matches Cargo.lock packages by name alone. A workspace crate
// and a registry dependency can share that name; only the workspace identity
// belongs to the coordinated version bump. Start from the base lockfile so
// registry versions, checksums, dependencies, and formatting remain untouched.
function blocks(content) {
  return content.split(/(?=^\[\[package\]\]$)/m).map((text) => ({
    text,
    name: text.match(/^name = "([^"]+)"$/m)?.[1],
    version: text.match(/^version = "([^"]+)"$/m)?.[1],
    source: /^source = /m.test(text),
  }));
}

export function preserveRegistryLockfile(base, proposed) {
  const before = blocks(base);
  const after = blocks(proposed);
  const locals = new Map(
    after.filter((pkg) => pkg.name && !pkg.source).map((pkg) => [pkg.name, pkg.version]),
  );
  const references = new Map();
  for (const pkg of before.filter((pkg) => pkg.name && !pkg.source)) {
    const version = locals.get(pkg.name);
    if (!version) throw new Error(`Missing local release package: ${pkg.name}`);
    references.set(`${pkg.name} ${pkg.version}`, `${pkg.name} ${version}`);
  }
  return before
    .map((pkg) => {
      if (!pkg.name || pkg.source) return pkg.text;
      return pkg.text
        .replace(/^version = "[^"]+"$/m, `version = "${locals.get(pkg.name)}"`)
        .replace(/^ "([^"]+)",$/gm, (line, reference) =>
          references.has(reference) ? ` "${references.get(reference)}",` : line,
        );
    })
    .join("");
}
