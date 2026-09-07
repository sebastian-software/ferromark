#!/usr/bin/env node
// Checks every README that carries the Ferramenta family block against the
// generator in sebastian-software/ferramenta, which renders the block from the
// registry in `src/family.ts` -- the single source of truth for tool names,
// jobs, groups, and links.
//
// The revision below is the only place this repository pins the generator. A
// registry change reaches this repository by bumping FAMILY_GENERATOR_REVISION
// and re-running this script with `--write`; the block is generated, so the
// diff shows exactly what moved.
import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";

// Resolved without ./lib/contracts.mjs so the check stays runnable before
// `pnpm install` in scripts/.
const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

export const FAMILY_GENERATOR_REVISION = "3225743e20818805a27e3e1a77cf1726bfb6f939";

export const FAMILY_GENERATOR_SPEC =
  `github:sebastian-software/ferramenta#${FAMILY_GENERATOR_REVISION}` + "&path:/packages/family";

// The tool this repository publishes, lowercase as the registry spells it.
export const FAMILY_CURRENT_TOOL = "ferromark";

// `github` is the full block with the grouped tables, for the repository README
// that GitHub and crates.io render. `registry` is the two plain-Markdown lines
// without HTML or tables, for the README npm renders for the published package.
// The platform sidecar packages under node/ferromark/npm/* carry no block.
export const FAMILY_READMES = [
  { path: "README.md", variant: "github" },
  { path: "node/ferromark/README.md", variant: "registry" },
];

function runGenerator(readme, variant, mode) {
  const result = spawnSync(
    "pnpm",
    [
      "dlx",
      FAMILY_GENERATOR_SPEC,
      "--current",
      FAMILY_CURRENT_TOOL,
      "--variant",
      variant,
      mode,
      path.join(repositoryRoot, readme),
    ],
    { shell: process.platform === "win32", stdio: "inherit" },
  );

  if (result.error) {
    throw result.error;
  }
  return result.status === 0;
}

export function main(argv = []) {
  const write = argv.includes("--write");
  const mode = write ? "--write" : "--check";

  let drifted = false;
  for (const { path: readme, variant } of FAMILY_READMES) {
    if (!runGenerator(readme, variant, mode)) {
      drifted = true;
      console.error(
        write
          ? `${readme}: the generator could not write the ${variant} family block.`
          : `${readme}: the ${variant} family block does not match the registry at ` +
              `${FAMILY_GENERATOR_REVISION}. Regenerate it with ` +
              "`node ./scripts/check-readme-family.mjs --write`.",
      );
    }
  }

  if (drifted) {
    process.exitCode = 1;
    return;
  }

  console.log(
    write
      ? "Ferramenta family blocks regenerated from the pinned registry"
      : "Ferramenta family blocks match the pinned registry",
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv.slice(2));
}
