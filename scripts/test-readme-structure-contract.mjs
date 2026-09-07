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
const CANONICAL_HOMEPAGE = "https://sebastian-software.github.io/ferromark/";
const TAGLINE = "Markdown to HTML with a secure default and every GFM extension included.";

// The family block is generated from the registry in sebastian-software/ferramenta
// and inserted between these markers; the company footer below it belongs to
// @sebastian-software/standards. Neither is hand-written, so this contract
// checks their shape and their drift gate rather than their wording.
const FAMILY_START = "<!-- ferramenta-family:start -->";
const FAMILY_END = "<!-- ferramenta-family:end -->";
const BRANDING_START = "<!-- sebastian-software-branding:start -->";
const FAMILY_CHECK_COMMAND = "node ./scripts/check-readme-family.mjs";
const FAMILY_TABLE_ROW =
  /^\| (?<openingEmphasis>\*\*)?\[(?<name>[a-z0-9-]+)\]\(https:\/\/[^)]+\)(?<closingEmphasis>\*\*)? \| (?<job>.+) \|$/;

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
  const parsedRows = rows.map((row) => {
    const match = row.match(FAMILY_TABLE_ROW);
    if (!match || match.groups.openingEmphasis !== match.groups.closingEmphasis) {
      failContract(`family rows must link a lowercase tool name: ${JSON.stringify(row)}`);
    }
    return match.groups;
  });

  const current = parsedRows.filter(({ openingEmphasis }) => openingEmphasis === "**");
  if (current.length !== 1 || current[0].name !== FAMILY_CURRENT_TOOL) {
    failContract(`the github family block must bold ${FAMILY_CURRENT_TOOL} and no other tool`);
  }
  if (current[0].job !== TAGLINE) {
    failContract(`the ${FAMILY_CURRENT_TOOL} registry job must match the canonical tagline`);
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
  if (!FAMILY_GENERATOR_SPEC.includes("&path:/packages/family")) {
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

function cargoPackageField(cargoToml, field) {
  const packageHeader = cargoToml.match(/^\[package\]$\n/m);
  if (packageHeader?.index === undefined) {
    failContract("Cargo.toml must declare a [package] table");
  }
  const packageStart = packageHeader.index + packageHeader[0].length;
  const followingTable = cargoToml.slice(packageStart).search(/^\[/m);
  const packageTable =
    followingTable === -1
      ? cargoToml.slice(packageStart)
      : cargoToml.slice(packageStart, packageStart + followingTable);
  const value = packageTable.match(new RegExp(`^${escapeRegExp(field)} = "([^"]+)"$`, "m"))?.[1];
  if (!value) {
    failContract(`Cargo.toml must declare package ${field}`);
  }
  return value;
}

function validateBrandMetadata(document, cargoToml, nodePackage) {
  const header = document.slice(0, document.indexOf("## Quick start\n"));
  if (!header.includes(`[Documentation site](${CANONICAL_HOMEPAGE})`)) {
    failContract("README header must link the canonical homepage");
  }
  if (!header.includes(TAGLINE)) {
    failContract("README header must state the canonical tagline");
  }
  if (cargoPackageField(cargoToml, "homepage") !== CANONICAL_HOMEPAGE) {
    failContract("Cargo.toml homepage must match the canonical homepage");
  }
  if (cargoPackageField(cargoToml, "description") !== TAGLINE) {
    failContract("Cargo.toml description must match the canonical tagline");
  }
  if (nodePackage.homepage !== CANONICAL_HOMEPAGE) {
    failContract("the npm package homepage must match the canonical homepage");
  }
  if (nodePackage.description !== TAGLINE) {
    failContract("the npm package description must match the canonical tagline");
  }
}

function validateEvidenceBackedBadges(document, cargoToml) {
  const rustVersion = cargoPackageField(cargoToml, "rust-version");
  const header = document.slice(0, document.indexOf("## Quick start\n"));
  const allowedBadges = new Set([
    "[![Powered by Sebastian Software](https://img.shields.io/badge/Powered%20by-Sebastian%20Software-00718d?style=flat-square)](https://oss.sebastian-software.com)",
    "[![CI](https://github.com/sebastian-software/ferromark/actions/workflows/ci.yml/badge.svg)](https://github.com/sebastian-software/ferromark/actions/workflows/ci.yml)",
    "[![crates.io](https://img.shields.io/crates/v/ferromark.svg)](https://crates.io/crates/ferromark)",
    "[![coverage gate ≥ 90%](https://img.shields.io/badge/coverage%20gate-%E2%89%A5%2090%25-brightgreen.svg)](https://github.com/sebastian-software/ferromark/blob/main/.github/workflows/ci.yml)",
    "[![docs.rs](https://docs.rs/ferromark/badge.svg)](https://docs.rs/ferromark)",
    "[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)",
    `[![Rust ${rustVersion}+](https://img.shields.io/badge/rust-${rustVersion}%2B-orange.svg)](#minimum-supported-rust-version)`,
  ]);
  const badgeRows = header.split("\n").filter((line) => line.startsWith("[!["));
  const badgePattern = /^\[!\[[^\]]+\]\([^\s)]+\)\]\([^\s)]+\)$/;
  if (badgeRows.length !== allowedBadges.size) {
    failContract("README header must contain only the allowed evidence-backed badge rows");
  }
  for (const badge of badgeRows) {
    if (!badgePattern.test(badge) || !allowedBadges.delete(badge)) {
      failContract(`README header must not include an unexpected claim badge: ${badge}`);
    }
  }
  if (allowedBadges.size !== 0) {
    failContract("README header must retain every allowed evidence-backed badge row");
  }
}

function validateMdxEvidence(document) {
  if (/\b\d+(?:\.\d+)?%\+? of real-world (?:MDX |\.mdx )?(?:files|patterns)/i.test(document)) {
    failContract("README must not make an unmeasured MDX coverage percentage claim");
  }
  if (!document.includes("`tests/mdx_segment_tests.rs` exercises that supported set.")) {
    failContract("README MDX coverage must point to the supporting fixture tests");
  }
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
    cargoToml = readRepositoryFile("Cargo.toml"),
    nodePackage = JSON.parse(readRepositoryFile("node/ferromark/package.json")),
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

  validateBrandMetadata(document, cargoToml, nodePackage);
  validateEvidenceBackedBadges(document, cargoToml);
  validateMdxEvidence(document);

  const benchmarks = section(document, "Benchmarks");
  if (
    !benchmarks.includes("benchmarks/bun-comparison/README.md") ||
    !benchmarks.includes("docs/reports/2026-09-05-bun-comparison.md") ||
    !benchmarks.includes("different compiler and allocator from the tables above")
  ) {
    failContract("Benchmarks must link and distinguish the exploratory Bun comparison");
  }
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

  it("rejects a README header without the canonical homepage", () => {
    assert.throws(
      () => validate(document.replace(CANONICAL_HOMEPAGE, "https://example.com/")),
      ContractError,
    );
  });

  it("rejects a Cargo description that drifts from the canonical tagline", () => {
    assert.throws(
      () =>
        validate(document, {
          cargoToml: readRepositoryFile("Cargo.toml").replace(TAGLINE, "A different tagline."),
        }),
      ContractError,
    );
  });

  it("rejects a canonical Cargo description outside the package table", () => {
    assert.throws(
      () =>
        validate(document, {
          cargoToml: readRepositoryFile("Cargo.toml")
            .replace(`description = "${TAGLINE}"\n`, "")
            .replace("[workspace]", `[workspace]\ndescription = "${TAGLINE}"`),
        }),
      ContractError,
    );
  });

  it("rejects an npm homepage that drifts from the canonical homepage", () => {
    assert.throws(
      () =>
        validate(document, {
          nodePackage: {
            ...JSON.parse(readRepositoryFile("node/ferromark/package.json")),
            homepage: "https://example.com/",
          },
        }),
      ContractError,
    );
  });

  it("rejects a Rust badge without the MSRV policy link", () => {
    assert.throws(
      () =>
        validate(document.replace("#minimum-supported-rust-version", "https://www.rust-lang.org")),
      ContractError,
    );
  });

  it("rejects an unexpected claim badge", () => {
    assert.throws(
      () =>
        validate(
          document.replace(
            "[![CI](https://github.com/sebastian-software/ferromark/actions/workflows/ci.yml/badge.svg)](https://github.com/sebastian-software/ferromark/actions/workflows/ci.yml)",
            "[![clippy](https://img.shields.io/badge/clippy--strict-passing-brightgreen.svg)](https://doc.rust-lang.org/clippy/)",
          ),
        ),
      ContractError,
    );
  });

  it("rejects a required badge replaced with a plain link", () => {
    assert.throws(
      () =>
        validate(
          document.replace(
            "[![docs.rs](https://docs.rs/ferromark/badge.svg)](https://docs.rs/ferromark)",
            "[docs.rs](https://docs.rs/ferromark)",
          ),
        ),
      ContractError,
    );
  });

  it("rejects an unmeasured MDX coverage percentage", () => {
    assert.throws(
      () => validate(document.replace("block-level patterns", "90%+ of real-world MDX patterns")),
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

  it("rejects a Bun comparison without its measurement caveat", () => {
    assert.throws(
      () =>
        validate(
          document.replace(
            "different compiler and allocator from the tables above",
            "the same configuration",
          ),
        ),
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

  it("rejects a registry job that drifts from the canonical tagline", () => {
    assert.throws(
      () =>
        validate(
          document.replace(
            `| **[${FAMILY_CURRENT_TOOL}](https://sebastian-software.github.io/ferromark/)** | ${TAGLINE} |`,
            "| **[ferromark](https://sebastian-software.github.io/ferromark/)** | Markdown to HTML — CommonMark & GFM |",
          ),
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
