import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { ContractError, deepCopy, readRepositoryFile, readYaml } from "./lib/contracts.mjs";

const CHECKOUT_ACTION = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const RUST_TOOLCHAIN_ACTION = "dtolnay/rust-toolchain@2c7215f132e9ebf062739d9130488b56d53c060c";
const INSTALL_ACTION = "taiki-e/install-action@e67fa11c4b9316fa714ddf0abed07a0c3143b95b";
const RUST_CACHE_ACTION = "Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae";
const CARGO_DENY_ACTION =
  "EmbarkStudios/cargo-deny-action@3c6349835b2b7b196a839186cb8b78e02f7b5f25";
const RUSTDOC_COMMAND = "RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features --locked";
const COVERAGE_COMMAND = "cargo llvm-cov --all-features --locked --no-report";
const COVERAGE_REPORT_COMMAND =
  'cargo llvm-cov report --lcov --output-path lcov.info --fail-under-lines "$COVERAGE_FLOOR"';
// Coverage is gated by this repository's own CI, not by an external service.
const EXTERNAL_COVERAGE_SERVICE = /codecov/i;

function failContract(message) {
  throw new ContractError(`CI hardening contract: ${message}`);
}

function readContributing() {
  return readRepositoryFile("CONTRIBUTING.md");
}

function readDependencyPolicy() {
  return readRepositoryFile("deny.toml");
}

function readReadme() {
  return readRepositoryFile("README.md");
}

function validate(
  workflow,
  {
    contributing = readContributing(),
    dependencyPolicy = readDependencyPolicy(),
    readme = readReadme(),
  } = {},
) {
  if (EXTERNAL_COVERAGE_SERVICE.test(JSON.stringify(workflow))) {
    failContract("coverage must stay in this workflow, not in an external coverage service");
  }

  assertDeepEqual(workflow.permissions, { contents: "read" }, () =>
    failContract("top-level permissions must grant contents: read only"),
  );

  const jobs = workflow.jobs;
  assertDeepEqual(jobs.rustsec?.permissions, { contents: "read", checks: "write" }, () =>
    failContract("rustsec permissions must grant only contents: read and checks: write"),
  );

  const fmtCommands = jobs.fmt.steps.map((step) => step.run).filter((run) => run !== undefined);
  if (!fmtCommands.includes(RUSTDOC_COMMAND)) {
    failContract("fmt job must reject rustdoc warnings for all features");
  }

  const coverage = jobs.coverage;
  if (coverage["runs-on"] !== "ubuntu-latest") {
    failContract("coverage job must run on ubuntu-latest");
  }

  const threshold = coverage.env?.COVERAGE_FLOOR;
  if (typeof threshold !== "string" || !/^\d+$/.test(threshold)) {
    failContract("the coverage job must carry the floor as a plain percentage in COVERAGE_FLOOR");
  }

  const steps = coverage.steps;
  const expectedActions = [
    CHECKOUT_ACTION,
    RUST_TOOLCHAIN_ACTION,
    INSTALL_ACTION,
    RUST_CACHE_ACTION,
  ];
  const actualActions = steps.map((step) => step.uses).filter((uses) => uses !== undefined);
  assertDeepEqual(actualActions, expectedActions, () =>
    failContract(
      `coverage actions must be pinned and ordered as ${JSON.stringify(expectedActions)}`,
    ),
  );

  const toolchain = steps.find((step) => step.uses === RUST_TOOLCHAIN_ACTION).with;
  assertDeepEqual(toolchain, { toolchain: "stable", components: "llvm-tools-preview" }, () =>
    failContract("coverage must install stable Rust with llvm-tools-preview"),
  );

  const installer = steps.find((step) => step.uses === INSTALL_ACTION).with;
  assertDeepEqual(installer, { tool: "cargo-llvm-cov" }, () =>
    failContract("coverage must install cargo-llvm-cov"),
  );

  const commands = steps.map((step) => step.run).filter((run) => run !== undefined);
  if (commands.length !== 3 || commands[0] !== COVERAGE_COMMAND) {
    failContract("coverage must run cargo llvm-cov for all features with the lockfile");
  }

  // The run summary is the transparency half of the gate: it states the
  // measured percentage and the floor, and it runs before the gate can fail.
  const summaryCommand = commands[1];
  for (const fragment of ["$GITHUB_STEP_SUMMARY", "Line coverage:", "$COVERAGE_FLOOR"]) {
    if (!summaryCommand.includes(fragment)) {
      failContract("coverage must summarize the measured percentage and the floor for the run");
    }
  }

  if (commands[2] !== COVERAGE_REPORT_COMMAND) {
    failContract("coverage must report lcov.info and fail under the line coverage floor");
  }

  if (!contributing.includes(`${threshold}% line coverage`)) {
    failContract(`CONTRIBUTING.md must document the ${threshold}% line coverage floor`);
  }
  if (!contributing.includes(`--fail-under-lines ${threshold}`)) {
    failContract(`CONTRIBUTING.md must show how to run the ${threshold}% gate locally`);
  }

  const badge = `[![coverage gate ≥ ${threshold}%](`;
  if (!readme.includes(badge)) {
    failContract(`README.md must carry a coverage gate badge for ${threshold}%`);
  }
  const badgeLine = readme.split("\n").find((line) => line.startsWith(badge));
  if (!badgeLine.endsWith("/.github/workflows/ci.yml)")) {
    failContract("the coverage gate badge must link the CI workflow file");
  }

  const dependencyJob = jobs["cargo-deny"];
  if (!dependencyJob) {
    failContract("a cargo-deny job must enforce the dependency policy");
  }
  const dependencySteps = dependencyJob.steps;
  const dependencyActions = dependencySteps
    .map((step) => step.uses)
    .filter((uses) => uses !== undefined);
  assertDeepEqual(dependencyActions, [CHECKOUT_ACTION, CARGO_DENY_ACTION], () =>
    failContract(
      `cargo-deny actions must be pinned and ordered as ${JSON.stringify([CHECKOUT_ACTION, CARGO_DENY_ACTION])}`,
    ),
  );

  const policyInputs = dependencySteps.find((step) => step.uses === CARGO_DENY_ACTION).with;
  assertDeepEqual(policyInputs, { command: "check", arguments: "" }, () =>
    failContract("cargo-deny must run every check with the arguments from deny.toml"),
  );

  if (!dependencyPolicy.includes('yanked = "deny"')) {
    failContract("deny.toml must reject yanked crates");
  }
}

function assertDeepEqual(actual, expected, onFailure) {
  try {
    assert.deepEqual(actual, expected);
  } catch {
    onFailure();
  }
}

function assertRejected(mutate, options = {}) {
  const copy = deepCopy(workflow);
  mutate(copy);
  assert.throws(() => validate(copy, options), ContractError);
}

const workflow = readYaml(".github", "workflows", "ci.yml");

describe("CI hardening contract", () => {
  it("accepts the current workflow", () => {
    validate(workflow);
  });

  it("rejects a write-scoped top-level token", () => {
    assertRejected((copy) => {
      copy.permissions.contents = "write";
    });
  });

  it("rejects a missing rustsec checks permission", () => {
    assertRejected((copy) => {
      delete copy.jobs.rustsec.permissions.checks;
    });
  });

  it("rejects a missing rustdoc warning gate", () => {
    assertRejected((copy) => {
      copy.jobs.fmt.steps = copy.jobs.fmt.steps.filter((step) => step.run !== RUSTDOC_COMMAND);
    });
  });

  it("rejects a mutable coverage action", () => {
    assertRejected((copy) => {
      const step = copy.jobs.coverage.steps.find((candidate) => candidate.uses === INSTALL_ACTION);
      step.uses = "taiki-e/install-action@v2";
    });
  });

  it("rejects coverage without all features", () => {
    assertRejected((copy) => {
      const step = copy.jobs.coverage.steps.find((candidate) => candidate.run === COVERAGE_COMMAND);
      step.run = "cargo llvm-cov --locked --no-report";
    });
  });

  it("rejects a reintroduced external coverage service", () => {
    assertRejected((copy) => {
      copy.jobs.coverage.steps.push({
        uses: "codecov/codecov-action@fb8b3582c8e4def4969c97caa2f19720cb33a72f",
        with: { files: "lcov.info" },
      });
    });
  });

  it("rejects coverage without a floor", () => {
    assertRejected((copy) => {
      const step = copy.jobs.coverage.steps.find(
        (candidate) => candidate.run === COVERAGE_REPORT_COMMAND,
      );
      step.run = "cargo llvm-cov report --lcov --output-path lcov.info";
    });
  });

  it("rejects a floor that is not a plain percentage", () => {
    assertRejected((copy) => {
      copy.jobs.coverage.env.COVERAGE_FLOOR = "ninety";
    });
  });

  it("rejects a coverage run that stays silent about the measurement", () => {
    assertRejected((copy) => {
      const step = copy.jobs.coverage.steps.find((candidate) =>
        candidate.run?.includes("$GITHUB_STEP_SUMMARY"),
      );
      step.run = "cargo llvm-cov report --summary-only";
    });
  });

  it("rejects an undocumented coverage floor", () => {
    assertRejected(() => {}, {
      contributing: readContributing().replace(/\d+% line coverage/g, "an unstated% line coverage"),
    });
  });

  it("rejects a local gate command with a different floor", () => {
    assertRejected(() => {}, {
      contributing: readContributing().replace(/--fail-under-lines \d+/g, "--fail-under-lines 1"),
    });
  });

  it("rejects a README without the coverage gate badge", () => {
    assertRejected(() => {}, {
      readme: readReadme().replace(/\[!\[coverage gate [^\]]+\]\([^\n]+\)/g, ""),
    });
  });

  it("rejects a coverage gate badge that does not link the workflow", () => {
    assertRejected(() => {}, {
      readme: readReadme().replace(
        /(\[!\[coverage gate [^\]]+\]\([^)]+\)\]\()[^)]+\)/,
        "$1https://example.com/coverage)",
      ),
    });
  });

  it("rejects a missing dependency policy job", () => {
    assertRejected((copy) => {
      delete copy.jobs["cargo-deny"];
    });
  });

  it("rejects a mutable cargo-deny action", () => {
    assertRejected((copy) => {
      const step = copy.jobs["cargo-deny"].steps.find(
        (candidate) => candidate.uses === CARGO_DENY_ACTION,
      );
      step.uses = "EmbarkStudios/cargo-deny-action@v2";
    });
  });

  it("rejects a dependency policy that allows yanked crates", () => {
    assertRejected(() => {}, {
      dependencyPolicy: readDependencyPolicy().replace('yanked = "deny"', 'yanked = "warn"'),
    });
  });
});
