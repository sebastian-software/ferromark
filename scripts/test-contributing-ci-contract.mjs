import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, it } from "node:test";

import { ContractError, parseYaml, readRepositoryFile, repositoryPath } from "./lib/contracts.mjs";

function failContract(message) {
  throw new ContractError(`CONTRIBUTING CI contract: ${message}`);
}

function markdownSection(document, heading) {
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

function fencedBashCommands(section, heading) {
  const match = section.match(/```bash\n([\s\S]*?)\n```/);
  if (!match) {
    failContract(`${JSON.stringify(heading)} must contain a bash code block`);
  }
  return match[1]
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

function ciCommand(job, prefix) {
  const command = job.steps
    .map((step) => step.run)
    .filter((run) => typeof run === "string")
    .find((candidate) => candidate.startsWith(prefix));
  if (!command) {
    failContract(`CI must define a ${prefix.trim()} command`);
  }
  return command;
}

function validate(root) {
  const contributing = readFileSync(path.join(root, "CONTRIBUTING.md"), "utf8");
  const cargoToml = readFileSync(path.join(root, "Cargo.toml"), "utf8");
  const workflow = parseYaml(readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8"));

  const rustVersion = cargoToml.match(/^rust-version\s*=\s*"([^"]+)"\s*$/m)?.[1];
  if (!rustVersion) {
    failContract("Cargo.toml must declare rust-version");
  }

  const gettingStarted = markdownSection(contributing, "Getting started");
  const expectedMsrv = `The minimum supported Rust version (MSRV) is Rust ${rustVersion}.`;
  if (!gettingStarted.includes(expectedMsrv)) {
    failContract(`Getting started must state ${JSON.stringify(expectedMsrv)}`);
  }
  if (!gettingStarted.includes("Run the [required local checks](#required-local-checks) below.")) {
    failContract("Getting started must point contributors to Required local checks");
  }
  const bootstrapCommands = fencedBashCommands(gettingStarted, "Getting started");
  if (bootstrapCommands.some((command) => command.startsWith("cargo "))) {
    failContract("Getting started must not duplicate cargo commands from Required local checks");
  }

  const jobs = workflow.jobs;
  const testJob = jobs.test;
  const testCommand = ciCommand(testJob, "cargo test ");
  if (!testCommand.includes("${{ matrix.args }}")) {
    failContract("CI test job must use a matrix command");
  }

  const testMatrix = testJob.strategy.matrix.include;
  const allFeatureArgs = testMatrix
    .map((entry) => entry.args)
    .filter((args) => args === "--all-features");
  if (allFeatureArgs.length === 0) {
    failContract("CI test matrix must include --all-features");
  }

  const mdxEntries = testMatrix.filter((entry) => entry.features === "mdx");
  const expectedMdxEntry = {
    os: "ubuntu-latest",
    rust: "stable",
    features: "mdx",
    args: "--no-default-features --features mdx",
  };
  if (mdxEntries.length !== 1 || !deepEqual(mdxEntries[0], expectedMdxEntry)) {
    failContract("CI test matrix must isolate the mdx feature on stable Ubuntu");
  }

  const msrvEntries = testMatrix.filter((entry) => entry.rust === rustVersion);
  if (msrvEntries.length === 0) {
    failContract(`CI test matrix must test Rust ${rustVersion}`);
  }
  const msrvArgs = msrvEntries.map((entry) => entry.args);
  if (!msrvArgs.includes("") || !msrvArgs.includes("--all-features")) {
    failContract(`CI Rust ${rustVersion} entries must cover default and --all-features`);
  }

  const expectedCommands = [
    testCommand.replace("${{ matrix.args }}", allFeatureArgs[0]),
    ciCommand(jobs.clippy, "cargo clippy "),
    ciCommand(testJob, "cargo test -p ferro-byte-search "),
    ciCommand(jobs.clippy, "cargo clippy -p ferro-byte-search "),
    ciCommand(jobs.fmt, "cargo fmt "),
  ];
  const requiredChecks = markdownSection(contributing, "Required local checks");
  const actualCommands = fencedBashCommands(requiredChecks, "Required local checks");
  if (!deepEqual(actualCommands, expectedCommands)) {
    failContract(
      `Required local checks commands must exactly match CI: expected ${JSON.stringify(expectedCommands)}, got ${JSON.stringify(actualCommands)}`,
    );
  }
  if (!requiredChecks.includes("[releasing guide](docs/releasing.md)")) {
    failContract("Required local checks must link the Node workspace release checks");
  }
}

function deepEqual(actual, expected) {
  try {
    assert.deepEqual(actual, expected);
    return true;
  } catch {
    return false;
  }
}

function withFixture(callback) {
  const fixtureRoot = mkdtempSync(path.join(tmpdir(), "ferromark-contributing-contract."));
  try {
    mkdirSync(path.join(fixtureRoot, ".github/workflows"), { recursive: true });
    for (const file of ["CONTRIBUTING.md", "Cargo.toml"]) {
      cpSync(repositoryPath(file), path.join(fixtureRoot, file));
    }
    cpSync(
      repositoryPath(".github/workflows/ci.yml"),
      path.join(fixtureRoot, ".github/workflows/ci.yml"),
    );
    callback(fixtureRoot);
  } finally {
    rmSync(fixtureRoot, { force: true, recursive: true });
  }
}

function mutateSectionCommand(document, heading, original, replacement) {
  const sectionStart = document.indexOf(`## ${heading}\n`);
  assert.notEqual(sectionStart, -1, `test fixture is missing ${JSON.stringify(heading)}`);
  const prefix = document.slice(0, sectionStart);
  const section = document.slice(sectionStart);
  assert.ok(section.includes(original), `test fixture is missing ${JSON.stringify(original)}`);
  return prefix + section.replace(original, replacement);
}

const contributingDocument = readRepositoryFile("CONTRIBUTING.md");
const requiredCommands = fencedBashCommands(
  markdownSection(contributingDocument, "Required local checks"),
  "Required local checks",
);
const requiredTest = requiredCommands.find((command) => command.startsWith("cargo test "));
const requiredClippy = requiredCommands.find((command) => command.startsWith("cargo clippy "));
const requiredFmt = requiredCommands.find((command) => command.startsWith("cargo fmt "));
const rustVersion = readRepositoryFile("Cargo.toml").match(
  /^rust-version\s*=\s*"([^"]+)"\s*$/m,
)?.[1];
const requiredMsrv = `The minimum supported Rust version (MSRV) is Rust ${rustVersion}.`;

describe("CONTRIBUTING CI contract", () => {
  it("documents the commands CI runs", () => {
    assert.ok(requiredTest, "test fixture is missing a required cargo test command");
    assert.ok(requiredClippy, "test fixture is missing a required cargo clippy command");
    assert.ok(requiredFmt, "test fixture is missing a required cargo fmt command");
    assert.ok(rustVersion, "test fixture is missing Cargo.toml rust-version");
    validate(repositoryPath());
  });

  for (const [label, [original, replacement]] of Object.entries({
    "required all-features test": [requiredTest, "cargo test --locked"],
    "required clippy command": [requiredClippy, "cargo clippy"],
    "required byte-search test": [
      "cargo test -p ferro-byte-search --locked",
      "cargo test -p ferro-byte-search",
    ],
    "required byte-search clippy": [
      "cargo clippy -p ferro-byte-search --all-targets --locked -- -D warnings",
      "cargo clippy -p ferro-byte-search",
    ],
    "required fmt command": [requiredFmt, "cargo fmt"],
  })) {
    it(`rejects a weakened ${label}`, () => {
      withFixture((fixtureRoot) => {
        const file = path.join(fixtureRoot, "CONTRIBUTING.md");
        writeFileSync(
          file,
          mutateSectionCommand(
            readFileSync(file, "utf8"),
            "Required local checks",
            original,
            replacement,
          ),
        );
        assert.throws(() => validate(fixtureRoot), ContractError);
      });
    });
  }

  it("rejects a stale MSRV statement", () => {
    withFixture((fixtureRoot) => {
      const file = path.join(fixtureRoot, "CONTRIBUTING.md");
      const document = readFileSync(file, "utf8");
      assert.ok(
        document.includes(requiredMsrv),
        `test fixture is missing ${JSON.stringify(requiredMsrv)}`,
      );
      writeFileSync(
        file,
        document.replace(
          requiredMsrv,
          "The minimum supported Rust version (MSRV) is Rust unknown.",
        ),
      );
      assert.throws(() => validate(fixtureRoot), ContractError);
    });
  });

  // The bootstrap section deliberately contains no cargo command, leaving
  // Required local checks as the single source for CI-equivalent commands.
  it("rejects a cargo command in the bootstrap section", () => {
    withFixture((fixtureRoot) => {
      const file = path.join(fixtureRoot, "CONTRIBUTING.md");
      writeFileSync(
        file,
        mutateSectionCommand(
          readFileSync(file, "utf8"),
          "Getting started",
          "cd ferromark",
          "cd ferromark\ncargo test --locked",
        ),
      );
      assert.throws(() => validate(fixtureRoot), ContractError);
    });
  });

  it("rejects a missing isolated mdx matrix entry", () => {
    withFixture((fixtureRoot) => {
      const file = path.join(fixtureRoot, ".github/workflows/ci.yml");
      const document = readFileSync(file, "utf8");
      const mdxEntry =
        '          - { os: ubuntu-latest, rust: stable, features: mdx, args: "--no-default-features --features mdx" }\n';
      assert.ok(
        document.includes(mdxEntry),
        "test fixture is missing the isolated mdx matrix entry",
      );
      writeFileSync(file, document.replace(mdxEntry, ""));
      assert.throws(() => validate(fixtureRoot), ContractError);
    });
  });
});
