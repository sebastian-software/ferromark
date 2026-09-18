import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import {
  maintenanceHistory,
  proposeRelease,
  readReleaseFiles,
  root,
  validateRelease,
} from "./lib/release-rehearsal.mjs";

if (process.argv.length !== 3)
  throw new Error("Usage: node scripts/rehearse-release.mjs <new-output-directory>");
const output = resolve(process.argv[2]);
mkdirSync(output); // Never overwrite an earlier rehearsal or a real checkout.
const results = [];

// The blueprint's "prove the candidate without publishing": the generated files
// are not only compared field by field, they are handed to the real tools.
// Cargo resolves the candidate manifests against the candidate lockfile, and
// pnpm re-locks the candidate manifests and must produce no change.
function run(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, encoding: "utf8" });
  if (result.error) throw result.error;
  return result;
}

function proveCargo(directory) {
  // `cargo metadata --locked` fails when the generated Cargo.lock no longer
  // matches the generated manifests, which is exactly the staleness that a
  // hand-maintained Cargo `extra-files` list used to introduce.
  // The sources are borrowed; only the manifests under review are generated,
  // and a generated file (README.md) is never replaced by a link to the source.
  for (const entry of ["src", "tests", "benches", "examples", "README.md", "LICENSE"]) {
    if (!existsSync(resolve(directory, entry))) {
      symlinkSync(resolve(root, entry), resolve(directory, entry));
    }
  }
  cpSync(resolve(root, "node/native"), resolve(directory, "node/native"), {
    recursive: true,
    filter: (source) => !source.includes("/target"),
  });
  // The generated member manifest wins over the copied one.
  writeFileSync(
    resolve(directory, "node/native/Cargo.toml"),
    readFileSync(resolve(directory, "node/native/Cargo.toml.generated"), "utf8"),
  );
  const result = run(
    "cargo",
    [
      "metadata",
      "--locked",
      "--offline",
      "--format-version",
      "1",
      "--manifest-path",
      resolve(directory, "Cargo.toml"),
    ],
    directory,
  );
  if (result.status !== 0) {
    throw new Error(`cargo metadata --locked rejected the candidate:\n${result.stderr}`);
  }
  const metadata = JSON.parse(result.stdout);
  return Object.fromEntries(
    metadata.packages
      .filter((entry) => entry.source === null)
      .map((entry) => [entry.name, entry.version]),
  );
}

function provePnpmLockfile(directory) {
  const node = resolve(directory, "node");
  cpSync(resolve(root, "node/pnpm-workspace.yaml"), resolve(node, "pnpm-workspace.yaml"));
  cpSync(resolve(root, "node/package.json"), resolve(node, "package.json"));
  const before = readFileSync(resolve(node, "pnpm-lock.yaml"), "utf8");
  const install = run("pnpm", ["install", "--lockfile-only", "--ignore-scripts"], node);
  if (install.status !== 0) {
    throw new Error(`pnpm install --lockfile-only failed on the candidate:\n${install.stderr}`);
  }
  const after = readFileSync(resolve(node, "pnpm-lock.yaml"), "utf8");
  if (before !== after) {
    throw new Error(
      "The candidate's pnpm lockfile is stale. With `workspace:*` sidecar references a " +
        "version bump must not change it; see docs/releasing.md.",
    );
  }
  return "clean";
}

function record(label, version, proposal) {
  validateRelease(proposal.files, version);
  const directory = resolve(output, label);
  for (const [file, content] of proposal.files) {
    const target = resolve(directory, file);
    mkdirSync(dirname(target), { recursive: true });
    // node/native/Cargo.toml is restored after the real crate is copied in.
    writeFileSync(file === "node/native/Cargo.toml" ? `${target}.generated` : target, content);
  }
  writeFileSync(resolve(directory, "release-pr.md"), `# ${proposal.title}\n\n${proposal.body}\n`);
  results.push({
    label,
    version,
    title: proposal.title,
    updatedFiles: proposal.paths,
    cargoPackages: proveCargo(directory),
    pnpmLockfile: provePnpmLockfile(directory),
  });
}

// What the publish workflow opens after the stable release without any
// `Release-As` footer; the released version is seeded so the case does not
// depend on the version the checkout itself carries.
record(
  "automatic",
  "2.0.1",
  await proposeRelease(
    (await proposeRelease(readReleaseFiles(), "chore: seed release\n\nRelease-As: 2.0.0")).files,
    maintenanceHistory,
  ),
);

// Forced transitions: the candidate series that ended with 2.0.0 needed an
// explicit footer to leave, which is what the rc.2 → 2.0.0 step rehearses.
let files = (
  await proposeRelease(
    readReleaseFiles(),
    "chore: seed version rehearsal\n\nRelease-As: 2.0.0-dev.0",
  )
).files;
for (const [version, message] of [
  ["2.0.0-rc.1", "feat!: prepare v2\n\nRelease-As: 2.0.0-rc.1"],
  ["2.0.0-rc.2", "fix: candidate correction\n\nRelease-As: 2.0.0-rc.2"],
  ["2.0.0", "feat: finalize v2\n\nRelease-As: 2.0.0"],
  ["2.0.1", "fix: ordinary maintenance correction\n\nRelease-As: 2.0.1"],
]) {
  const proposal = await proposeRelease(files, message);
  record(version, version, proposal);
  files = proposal.files;
}
writeFileSync(resolve(output, "results.json"), `${JSON.stringify(results, null, 2)}\n`);
console.log(`Verified ${results.length} coordinated releases; review files in ${output}`);
