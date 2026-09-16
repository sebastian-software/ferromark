import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import {
  candidateHistory,
  proposeRelease,
  readReleaseFiles,
  validateRelease,
} from "./lib/release-rehearsal.mjs";

if (process.argv.length !== 3)
  throw new Error("Usage: node scripts/rehearse-release.mjs <new-output-directory>");
const output = resolve(process.argv[2]);
mkdirSync(output); // Never overwrite an earlier rehearsal or a real checkout.
const results = [];

function record(label, version, proposal) {
  validateRelease(proposal.files, version);
  for (const [file, content] of proposal.files) {
    const target = resolve(output, label, file);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, content);
  }
  writeFileSync(
    resolve(output, label, "release-pr.md"),
    `# ${proposal.title}\n\n${proposal.body}\n`,
  );
  results.push({ label, version, title: proposal.title, updatedFiles: proposal.paths });
}

// What the Release Please workflow opens on the current `main` without any
// `Release-As` footer, starting from the repository's own released version.
record("automatic", "2.0.0-rc.2", await proposeRelease(readReleaseFiles(), candidateHistory));

// Forced transitions: the candidate series is automatic, leaving it is not.
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
