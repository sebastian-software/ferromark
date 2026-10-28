import { createHash } from "node:crypto";
import { copyFile, mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

export async function collectPluginNotices(metadata, output, root) {
  const notices = [];
  for (const pkg of runtimePackages(metadata))
    notices.push(await packageNotices(pkg, output, root));
  await mkdir(path.join(output, "THIRD_PARTY_NOTICES"), { recursive: true });
  await writeFile(
    path.join(output, "THIRD_PARTY_NOTICES", "packages.json"),
    `${JSON.stringify(notices, null, 2)}\n`,
  );
  return notices;
}

function runtimePackages(metadata) {
  const nodes = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
  const packages = new Map(metadata.packages.map((pkg) => [pkg.id, pkg]));
  const host = metadata.packages.find((pkg) => pkg.name === "ferromark-node");
  const pending = [host.id];
  const included = new Set();
  while (pending.length) {
    const id = pending.pop();
    if (included.has(id)) continue;
    included.add(id);
    for (const dep of nodes.get(id).deps) {
      if (dep.dep_kinds.some((kind) => kind.kind !== "dev")) pending.push(dep.pkg);
    }
  }
  return [...included].sort().map((id) => packages.get(id));
}

async function packageNotices(pkg, output, root) {
  const id = pkg.id;
  const directory = path.dirname(pkg.manifest_path);
  const key = `${pkg.name}-${pkg.version}-${createHash("sha256").update(id).digest("hex").slice(0, 8)}`;
  let files = await licenseFiles(directory);
  if (files.length === 0 && pkg.source === null) {
    files = ["LICENSE", "LICENSE-MIT"].map((file) => ({ file, from: path.join(root, file) }));
  }
  for (const { file, from } of files) {
    const target = path.join(output, "THIRD_PARTY_NOTICES", key, file);
    await mkdir(path.dirname(target), { recursive: true });
    await copyFile(from, target);
  }
  if (pkg.name === "ferrigura-text" && !files.some(({ file }) => file.startsWith("fonts/"))) {
    throw new Error("Ferrigura font attribution files are missing");
  }
  return {
    name: pkg.name,
    version: pkg.version,
    source: pkg.source,
    sourceArchive: pkg.source?.startsWith("registry+")
      ? `https://static.crates.io/crates/${pkg.name}/${pkg.name}-${pkg.version}.crate`
      : null,
    license: pkg.license,
    directory: key,
    files: files.map(({ file }) => file),
    textsSha256: createHash("sha256")
      .update(Buffer.concat(await Promise.all(files.map(({ from }) => readFile(from)))))
      .digest("hex"),
  };
}

async function licenseFiles(directory, prefix = "") {
  const result = [];
  for (const entry of await readdir(path.join(directory, prefix), { withFileTypes: true })) {
    const file = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isFile() && /^(?:licen[cs]e|notice|copying|ofl)(?:[._-]|$)/i.test(entry.name)) {
      result.push({ file, from: path.join(directory, file) });
    } else if (entry.isDirectory() && (prefix || entry.name === "fonts")) {
      result.push(...(await licenseFiles(directory, file)));
    }
  }
  return result.sort((a, b) => a.file.localeCompare(b.file));
}
