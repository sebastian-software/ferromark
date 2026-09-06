import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  FAMILY_CURRENT_TOOL,
  FAMILY_GENERATOR_REVISION,
  FAMILY_GENERATOR_SPEC,
} from "./check-readme-family.mjs";
import { ContractError, escapeRegExp, readRepositoryFile, readYaml } from "./lib/contracts.mjs";

const MIGRATION_GUIDE = "[0.4–0.7 migration guide](docs/migration-0.4.md)";
const PROJECT_STRUCTURE_FILES = ["highlight.rs", "events.rs", "strict.rs"];

// The family block is generated from the registry in sebastian-software/ferramenta
// and inserted between these markers; the company footer below it belongs to
// @sebastian-software/standards. Neither is hand-written, so this contract
// checks their shape and their drift gate rather than their wording.
const FAMILY_START = "<!-- ferramenta-family:start -->";
const FAMILY_END = "<!-- ferramenta-family:end -->";
const BRANDING_START = "<!-- sebastian-software-branding:start -->";
const FAMILY_CHECK_COMMAND = "node ./scripts/check-readme-family.mjs";
const FAMILY_TABLE_ROW = /^\| (\*\*)?\[[a-z0-9-]+\]\(https:\/\/[^)]+\)(\*\*)? \| .+ \|$/;

function failContract(message) {
  throw new ContractError(`README structure contract: ${message}`);
}

function familyBlock(document, label) {
  const starts = document.split(FAMILY_START).length - 1;
  const ends = document.split(FAMILY_END).length - 1;
  if (starts !== 1 || ends !== 1) {
    failContract(`${label} must carry exactly one generated family block`);
  }
  const start = document.indexOf(FAMILY_START);
  const end = document.indexOf(FAMILY_END);
  if (start > end) {
    failContract(`${label} family block markers must be ordered`);
  }
  return document.slice(start + FAMILY_START.length, end);
}

function validateGithubFamilyBlock(document) {
  const block = familyBlock(document, "README");

  const branding = document.indexOf(BRANDING_START);
  if (branding !== -1 && document.indexOf(FAMILY_START) > branding) {
    failContract("the family block must sit above the company footer");
  }
  if (document.indexOf(FAMILY_START) < document.indexOf("## License")) {
    failContract("the family block must sit below the repository's own content");
  }
  if (document.replace(block, "").includes("ferramenta.dev")) {
    failContract("the family must be described once, inside the generated block");
  }
  if (!block.includes("https://ferramenta.dev")) {
    failContract("the family block must link ferramenta.dev");
  }
  if (!block.includes("| Tool | Job |")) {
    failContract("the github family block must list each group as a table");
  }

  const rows = block
    .split("\n")
    .filter((line) => line.startsWith("| "))
    .filter((line) => !line.startsWith("| Tool |") && !line.startsWith("| --- |"));
  if (rows.length === 0) {
    failContract("the github family block must list the family");
  }
  for (const row of rows) {
    if (!FAMILY_TABLE_ROW.test(row)) {
      failContract(`family rows must link a lowercase tool name: ${JSON.stringify(row)}`);
    }
  }

  const current = rows.filter((row) => row.startsWith("| **["));
  if (current.length !== 1 || !current[0].startsWith(`| **[${FAMILY_CURRENT_TOOL}](`)) {
    failContract(`the github family block must bold ${FAMILY_CURRENT_TOOL} and no other tool`);
  }
}

function validateRegistryFamilyBlock(nodeReadme) {
  const block = familyBlock(nodeReadme, "the npm package README");

  if (!block.includes(`**${FAMILY_CURRENT_TOOL}**`)) {
    failContract(`the registry family block must name ${FAMILY_CURRENT_TOOL}`);
  }
  if (!block.includes("https://ferramenta.dev")) {
    failContract("the registry family block must link ferramenta.dev");
  }
  if (/^[|#<]/m.test(block.trim())) {
    failContract("the registry family block must stay plain Markdown without tables or HTML");
  }
  const siblings = block.split("\n").find((line) => line.startsWith("Siblings:"));
  if (!siblings) {
    failContract("the registry family block must list the siblings");
  }
  if (siblings.includes(`[${FAMILY_CURRENT_TOOL}]`)) {
    failContract(
      `the registry family block must leave ${FAMILY_CURRENT_TOOL} out of its own sibling list`,
    );
  }
}

function validateFamilyDriftGate(workflow, contributing) {
  if (!/^[0-9a-f]{40}$/.test(FAMILY_GENERATOR_REVISION)) {
    failContract("the family generator must be pinned to a full commit SHA, not a branch");
  }
  if (!FAMILY_GENERATOR_SPEC.includes("&path:/packages/ardo-config")) {
    failContract("the family generator spec must select the generator package");
  }

  const fmtCommands = workflow.jobs.fmt.steps
    .map((step) => step.run)
    .filter((run) => run !== undefined);
  if (!fmtCommands.some((command) => command.includes(FAMILY_CHECK_COMMAND))) {
    failContract(`the fmt job must run ${FAMILY_CHECK_COMMAND}`);
  }
  if (!contributing.includes(FAMILY_CHECK_COMMAND)) {
    failContract("CONTRIBUTING.md must document how to regenerate the family block");
  }
  if (!contributing.includes("FAMILY_GENERATOR_REVISION")) {
    failContract("CONTRIBUTING.md must document how to bump the pinned generator revision");
  }
}

function section(document, heading) {
  const marker = `## ${heading}\n`;
  const start = document.indexOf(marker);
  if (start === -1) {
    failContract(`must contain a ${JSON.stringify(heading)} section`);
  }
  const contentStart = start + marker.length;
  const following = document.slice(contentStart).search(/^## /m);
  return following === -1
    ? document.slice(contentStart)
    : document.slice(contentStart, contentStart + following);
}

function lockedBenchmarkVersion(packageName) {
  const lockfile = readRepositoryFile("benchmarks/md4c-comparison/Cargo.lock");
  const match = lockfile.match(
    new RegExp(`^name = "${escapeRegExp(packageName)}"\\nversion = "([^"]+)"$`, "m"),
  );
  if (!match) {
    failContract(`benchmark lockfile must contain ${packageName}`);
  }
  return match[1];
}

function pinnedMd4cRevision() {
  const buildScript = readRepositoryFile("benchmarks/md4c-comparison/build.rs");
  const revision = buildScript.match(/^const MD4C_REVISION: &str = "([0-9a-f]{40})";$/m)?.[1];
  if (!revision) {
    failContract("benchmark build must declare a full MD4C_REVISION");
  }
  return revision;
}

function validate(
  document,
  {
    contributing = readRepositoryFile("CONTRIBUTING.md"),
    performancePlan = readRepositoryFile("docs/arch/ARCH-PLAN-001-performance-opportunities.md"),
    nodeReadme = readRepositoryFile("node/ferromark/README.md"),
    workflow = readYaml(".github", "workflows", "ci.yml"),
  } = {},
) {
  const headings = [...document.matchAll(/^## (.+)$/gm)].map((match) => match[1]);
  if (new Set(headings).size !== headings.length) {
    failContract("must not repeat top-level headings");
  }

  for (const heading of headings) {
    if (section(document, heading).trim().length === 0) {
      failContract(`${JSON.stringify(heading)} must not be empty`);
    }
  }

  const cliStart = document.indexOf("## CLI\n");
  const configurationStart = document.indexOf("## Markdown configuration\n");
  if (cliStart === -1 || configurationStart === -1 || cliStart >= configurationStart) {
    failContract("CLI must precede Markdown configuration so each section owns its content");
  }

  const cli = section(document, "CLI");
  const configuration = section(document, "Markdown configuration");
  if (!cli.includes("cargo install ferromark")) {
    failContract("CLI must document installation");
  }
  if (!cli.includes("--trusted")) {
    failContract("CLI must document trusted mode");
  }
  if (!configuration.includes("Options::minimal()")) {
    failContract("Markdown configuration must describe presets");
  }
  if (!configuration.includes("`Options` is non-exhaustive")) {
    failContract("Markdown configuration must document Options construction");
  }
  if (!document.includes(MIGRATION_GUIDE)) {
    failContract("README must preserve the migration guide link");
  }

  const benchmarks = section(document, "Benchmarks");
  if (!benchmarks.includes("These rankings are Apple Silicon results only")) {
    failContract("Benchmarks must scope published rankings to Apple Silicon");
  }
  if (!benchmarks.includes("has not been re-measured") || !benchmarks.includes("x86-64")) {
    failContract("Benchmarks must disclose the missing x86-64 comparison");
  }
  for (const packageName of ["pulldown-cmark", "comrak"]) {
    const lockedVersion = lockedBenchmarkVersion(packageName);
    const pattern = new RegExp(`${escapeRegExp(packageName)}\\s+${escapeRegExp(lockedVersion)}`);
    if (!pattern.test(benchmarks)) {
      failContract(`Benchmarks must state locked ${packageName} ${lockedVersion}`);
    }
  }

  const shortRevision = pinnedMd4cRevision().slice(0, 7);
  if (!benchmarks.includes(`md4c @ ${shortRevision}`)) {
    failContract(`Benchmarks must state pinned md4c revision ${shortRevision}`);
  }
  if (
    !benchmarks.includes(`checkout --detach ${shortRevision}`) ||
    !contributing.includes(`checkout --detach ${shortRevision}`)
  ) {
    failContract(`README and CONTRIBUTING must check out pinned md4c revision ${shortRevision}`);
  }
  for (const instructions of [benchmarks, contributing]) {
    if (
      !instructions.includes("cargo bench --locked") ||
      !instructions.includes("--manifest-path benchmarks/md4c-comparison/Cargo.toml")
    ) {
      failContract("README and CONTRIBUTING must use the locked isolated benchmark manifest");
    }
  }
  if (performancePlan.includes("PERF_ATTEMPTS.md")) {
    failContract("Performance plan must not reference the removed PERF_ATTEMPTS.md");
  }
  const comparisonCommands = [
    ...performancePlan.matchAll(/`([^`\n]*cargo bench[^`\n]*comparison[^`\n]*)`/g),
  ].map((match) => match[1]);
  if (comparisonCommands.length === 0) {
    failContract("Performance plan must document the comparison benchmark command");
  }
  if (
    !comparisonCommands.every(
      (command) =>
        command.includes("MD4C_DIR=/path/to/md4c cargo bench --locked") &&
        command.includes("--manifest-path benchmarks/md4c-comparison/Cargo.toml"),
    )
  ) {
    failContract(
      "Every performance-plan comparison command must use the locked isolated benchmark",
    );
  }
  if (!document.includes("baseline SSE2 (x86-64)")) {
    failContract("README must describe the x86-64 inline SIMD path");
  }

  const projectStructure = section(document, "Project structure");
  for (const filename of PROJECT_STRUCTURE_FILES) {
    if (!projectStructure.includes(filename)) {
      failContract(`Project structure must include ${filename}`);
    }
  }

  validateGithubFamilyBlock(document);
  validateRegistryFamilyBlock(nodeReadme);
  validateFamilyDriftGate(workflow, contributing);
}

const document = readRepositoryFile("README.md");
const nodeReadme = readRepositoryFile("node/ferromark/README.md");

describe("README structure contract", () => {
  it("accepts the current README", () => {
    validate(document);
  });

  it("rejects an empty Markdown configuration section", () => {
    assert.throws(
      () =>
        validate(
          document.replace(
            "## Markdown configuration\n\nStart from",
            "## Markdown configuration\n\n## Configuration details\n\nStart from",
          ),
        ),
      ContractError,
    );
  });

  it("rejects misowned Markdown configuration content", () => {
    assert.throws(
      () => validate(document.replace("## Markdown configuration\n", "")),
      ContractError,
    );
  });

  it("rejects renamed CLI trusted guidance", () => {
    assert.throws(() => validate(document.replaceAll("--trusted", "--safe")), ContractError);
  });

  it("rejects a missing migration guide link", () => {
    assert.throws(
      () => validate(document.replace(MIGRATION_GUIDE, "migration guide")),
      ContractError,
    );
  });

  it("rejects an unscoped Apple Silicon benchmark claim", () => {
    assert.throws(
      () =>
        validate(
          document.replace(
            "These rankings are Apple Silicon results only",
            "These rankings apply everywhere",
          ),
        ),
      ContractError,
    );
  });

  it("rejects a dropped x86-64 benchmark caveat", () => {
    assert.throws(
      () => validate(document.replace("has not been re-measured", "has been re-measured")),
      ContractError,
    );
  });

  it("rejects a stale locked comrak version", () => {
    const lockedVersion = lockedBenchmarkVersion("comrak");
    assert.throws(
      () =>
        validate(
          document.replace(new RegExp(`comrak\\s+${escapeRegExp(lockedVersion)}`), "comrak 0.0.0"),
        ),
      ContractError,
    );
  });

  it("rejects an unpinned md4c contributor checkout", () => {
    const revision = pinnedMd4cRevision().slice(0, 7);
    assert.throws(
      () =>
        validate(document, {
          contributing: readRepositoryFile("CONTRIBUTING.md").replace(
            `checkout --detach ${revision}`,
            "checkout --detach main",
          ),
        }),
      ContractError,
    );
  });

  it("rejects a reference to the removed performance evidence file", () => {
    assert.throws(
      () =>
        validate(document, {
          performancePlan: `References PERF_ATTEMPTS.md\n${readRepositoryFile("docs/arch/ARCH-PLAN-001-performance-opportunities.md")}`,
        }),
      ContractError,
    );
  });

  it("rejects a non-isolated performance benchmark command", () => {
    assert.throws(
      () =>
        validate(document, {
          performancePlan: readRepositoryFile(
            "docs/arch/ARCH-PLAN-001-performance-opportunities.md",
          ).replaceAll("MD4C_DIR=/path/to/md4c cargo bench --locked", "cargo bench"),
        }),
      ContractError,
    );
  });

  for (const filename of PROJECT_STRUCTURE_FILES) {
    it(`rejects a project structure without ${filename}`, () => {
      assert.throws(
        () => validate(document.replace(filename, "omitted-source-file.rs")),
        ContractError,
      );
    });
  }

  it("rejects a duplicated family block", () => {
    assert.throws(() => validate(`${document}\n${FAMILY_START}\n${FAMILY_END}\n`), ContractError);
  });

  it("rejects a family block below the company footer", () => {
    const block = document.slice(
      document.indexOf(FAMILY_START),
      document.indexOf(FAMILY_END) + FAMILY_END.length,
    );
    assert.throws(() => validate(document.replace(block, "") + `\n${block}\n`), ContractError);
  });

  it("rejects a second, hand-written family description", () => {
    assert.throws(
      () =>
        validate(document.replace("## License\n", "## License\n\nSee https://ferramenta.dev.\n")),
      ContractError,
    );
  });

  it("rejects a family block that bolds another tool", () => {
    assert.throws(
      () =>
        validate(
          document
            .replace(`**[${FAMILY_CURRENT_TOOL}](`, `[${FAMILY_CURRENT_TOOL}](`)
            .replace("| [ferriki](", "| **[ferriki](")
            .replace(") | Shiki-compatible", ")** | Shiki-compatible"),
        ),
      ContractError,
    );
  });

  it("rejects a registry block that carries a table", () => {
    assert.throws(
      () =>
        validate(document, {
          nodeReadme: nodeReadme.replace("Siblings: ", "| Tool |\n| --- |\nSiblings: "),
        }),
      ContractError,
    );
  });

  it("rejects a registry block that repeats the current tool as a sibling", () => {
    assert.throws(
      () =>
        validate(document, {
          nodeReadme: nodeReadme.replace(
            "Siblings: ",
            `Siblings: [${FAMILY_CURRENT_TOOL}](https://ferramenta.dev) · `,
          ),
        }),
      ContractError,
    );
  });

  it("rejects an npm package README without a family block", () => {
    assert.throws(
      () =>
        validate(document, {
          nodeReadme: nodeReadme.slice(0, nodeReadme.indexOf(FAMILY_START)),
        }),
      ContractError,
    );
  });

  it("rejects CI without the pinned family drift check", () => {
    const workflow = readYaml(".github", "workflows", "ci.yml");
    workflow.jobs.fmt.steps = workflow.jobs.fmt.steps.filter(
      (step) => !step.run?.includes(FAMILY_CHECK_COMMAND),
    );
    assert.throws(() => validate(document, { workflow }), ContractError);
  });

  it("rejects CONTRIBUTING without the regeneration procedure", () => {
    assert.throws(
      () =>
        validate(document, {
          contributing: readRepositoryFile("CONTRIBUTING.md").replaceAll(
            FAMILY_CHECK_COMMAND,
            "some other command",
          ),
        }),
      ContractError,
    );
  });
});
