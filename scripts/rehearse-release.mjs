import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { proposeRelease, readReleaseFiles, validateRelease } from "./lib/release-rehearsal.mjs";

if (process.argv.length !== 3)
  throw new Error("Usage: node scripts/rehearse-release.mjs <new-output-directory>");
const output = resolve(process.argv[2]);
mkdirSync(output); // Never overwrite an earlier rehearsal or a real checkout.
let files = readReleaseFiles();
const results = [];
for (const [version, message] of [
  ["2.0.0-rc.1", "feat!: prepare v2\n\nRelease-As: 2.0.0-rc.1"],
  ["2.0.0-rc.2", "fix: candidate correction\n\nRelease-As: 2.0.0-rc.2"],
  ["2.0.0", "feat: finalize v2\n\nRelease-As: 2.0.0"],
  ["2.0.1", "fix: ordinary maintenance correction"],
]) {
  const proposal = await proposeRelease(files, message);
  validateRelease(proposal.files, version);
  for (const [file, content] of proposal.files) {
    const target = resolve(output, version, file);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, content);
  }
  writeFileSync(
    resolve(output, version, "release-pr.md"),
    `# ${proposal.title}\n\n${proposal.body}\n`,
  );
  results.push({ version, title: proposal.title, updatedFiles: proposal.paths });
  files = proposal.files;
}
writeFileSync(resolve(output, "results.json"), `${JSON.stringify(results, null, 2)}\n`);
console.log(`Verified ${results.length} coordinated releases; review files in ${output}`);
