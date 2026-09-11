import assert from "node:assert/strict";
import { statSync } from "node:fs";
import { describe, it } from "node:test";

import {
  ContractError,
  deepCopy,
  readRepositoryFile,
  readYaml,
  repositoryPath,
} from "./lib/contracts.mjs";

const CHECKOUT_ACTION = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const RUST_TOOLCHAIN_ACTION = "dtolnay/rust-toolchain@2c7215f132e9ebf062739d9130488b56d53c060c";
const RUST_CACHE_ACTION = "Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae";
const CACHE_RESTORE_ACTION = "actions/cache/restore@55cc8345863c7cc4c66a329aec7e433d2d1c52a9";
const BENCHMARK_ACTION =
  "benchmark-action/github-action-benchmark@52576c92bccf6ac60c8223ec7eb2565637cae9ba";
const CACHE_SAVE_ACTION = "actions/cache/save@55cc8345863c7cc4c66a329aec7e433d2d1c52a9";
const CACHE_KEY = "ferromark-benchmark-${{ runner.os }}-main-${{ github.sha }}";
const PATHOLOGICAL_BENCHMARKS = [
  "bracket_explosion_64k",
  "unmatched_backticks_64k",
  "reference_definitions_64k",
  "invalid_html_starts_64k",
];

function failContract(message) {
  throw new ContractError(`Benchmark CI contract: ${message}`);
}

function assertDeepEqual(actual, expected, message) {
  try {
    assert.deepEqual(actual, expected);
  } catch {
    failContract(message);
  }
}

// The YAML 1.1 parser Ruby used turned an unquoted `on` key into true; the
// 1.2 parser keeps the string. Accept whichever the parser produced.
function triggers(workflow) {
  const events = workflow.on ?? workflow[true];
  if (!events) {
    failContract("workflow triggers are missing");
  }
  return events;
}

function validate(
  workflow,
  {
    benchmarkSource = readRepositoryFile("benches/parsing.rs"),
    largeFixtureBytes = statSync(repositoryPath("benches/fixtures/commonmark-1m.md")).size,
  } = {},
) {
  assertDeepEqual(
    workflow.permissions,
    { contents: "read" },
    "workflow permissions must grant contents: read only",
  );

  const events = triggers(workflow);
  const push = events.push;
  const pullRequest = events.pull_request;
  assertDeepEqual(push.branches, ["main"], "push must target main");
  if (!push.paths.includes("src/**")) {
    failContract("push must run for src changes");
  }
  if (!pullRequest.paths.includes("src/**")) {
    failContract("pull requests must run benchmarks for src changes");
  }
  if (!("workflow_dispatch" in events)) {
    failContract("manual benchmark runs must remain available");
  }

  const job = workflow.jobs.benchmark;
  if (job["runs-on"] !== "ubuntu-latest") {
    failContract("benchmark job must use ubuntu-latest");
  }

  const steps = job.steps;
  const actions = steps.map((step) => step.uses).filter((uses) => uses !== undefined);
  assertDeepEqual(
    actions,
    [
      CHECKOUT_ACTION,
      RUST_TOOLCHAIN_ACTION,
      RUST_CACHE_ACTION,
      CHECKOUT_ACTION,
      CACHE_RESTORE_ACTION,
      BENCHMARK_ACTION,
      CACHE_SAVE_ACTION,
    ],
    "workflow actions must remain pinned and ordered",
  );

  const restore = steps.find((step) => step.uses === CACHE_RESTORE_ACTION);
  if (restore.if !== "github.event_name != 'pull_request'") {
    failContract("pull requests must not use historical runner timings");
  }
  const restoreInputs = restore.with;
  if (restoreInputs.key !== CACHE_KEY) {
    failContract("benchmark history cache key changed");
  }
  if (!restoreInputs["restore-keys"].includes("ferromark-benchmark-${{ runner.os }}-main-")) {
    failContract("main must restore the latest benchmark history");
  }

  const base = steps.find((step) => step.with?.path === ".benchmark-base");
  if (
    base?.if !== "github.event_name == 'pull_request'" ||
    base.with.ref !== "${{ github.event.pull_request.base.sha }}"
  ) {
    failContract("pull requests must check out their base revision on the same runner");
  }
  const pairedComparison = steps.find(
    (step) => step.name === "Compare with PR base measured on this runner",
  );
  if (
    pairedComparison?.if !== "github.event_name == 'pull_request'" ||
    pairedComparison["continue-on-error"] ||
    !pairedComparison.run.includes("python3 scripts/compare-ci-benchmarks.py") ||
    !pairedComparison.run.includes("benchmark-base-output.txt benchmark-output.txt")
  ) {
    failContract("pull requests must fail when the same-runner comparison fails");
  }

  const command = steps.find((step) => step.name === "Run representative benchmarks").run;
  for (const fragment of [
    "set -o pipefail",
    "cargo bench --locked --bench parsing",
    "cargo bench --manifest-path .benchmark-base/Cargo.toml --locked --bench parsing",
    "tee benchmark-base-output.txt",
    "tee benchmark-output.txt",
    "--output-format bencher",
    "--warm-up-time 1",
    "--measurement-time 3",
    "--sample-size 30",
  ]) {
    if (!command.includes(fragment)) {
      failContract(`benchmark command must include ${fragment}`);
    }
  }

  const history = steps.find((step) => step.uses === BENCHMARK_ACTION);
  if (history.if !== "github.event_name != 'pull_request'") {
    failContract("historical comparisons must not gate pull requests");
  }
  const compare = history.with;
  assertDeepEqual(
    compare,
    {
      name: "ferromark parser",
      tool: "cargo",
      "output-file-path": "benchmark-output.txt",
      "external-data-json-path": ".benchmark-cache/benchmark-data.json",
      "save-data-file":
        "${{ github.ref == 'refs/heads/main' && github.event_name != 'pull_request' }}",
      "alert-threshold": "120%",
      "fail-threshold": "120%",
      "fail-on-alert": false,
      "summary-always": true,
    },
    "main must record history with the 20% alert threshold",
  );

  const save = steps.find((step) => step.uses === CACHE_SAVE_ACTION);
  const expectedCondition =
    "github.ref == 'refs/heads/main' && github.event_name != 'pull_request' && steps.benchmark-history.outputs.cache-hit != 'true'";
  if (save.if !== expectedCondition) {
    failContract("benchmark history must only be saved outside pull requests");
  }
  if (save.with?.key !== CACHE_KEY) {
    failContract("saved benchmark history must reuse the restore key");
  }

  if (!benchmarkSource.includes("pub const PATHOLOGICAL_BYTES: usize = 64 * 1024;")) {
    failContract("pathological benchmarks must share a fixed 64 KiB byte budget");
  }
  for (const benchmarkName of PATHOLOGICAL_BENCHMARKS) {
    if (!benchmarkSource.includes(`bench_function("${benchmarkName}"`)) {
      failContract(`pathological benchmark ${benchmarkName} is missing`);
    }
  }
  if (
    !benchmarkSource.includes('include_str!("fixtures/commonmark-1m.md")') ||
    !benchmarkSource.includes('bench_function("commonmark_1m"')
  ) {
    failContract("the 1 MiB CommonMark fixture must be benchmarked");
  }
  if (largeFixtureBytes < 1024 * 1024) {
    failContract("large CommonMark fixture must be at least 1 MiB");
  }
}

function assertRejected(mutate, options = {}) {
  const copy = deepCopy(workflow);
  mutate(copy);
  assert.throws(() => validate(copy, options), ContractError);
}

const workflow = readYaml(".github", "workflows", "benchmarks.yml");

describe("benchmark CI contract", () => {
  it("accepts the current workflow", () => {
    validate(workflow);
  });

  it("rejects a write-scoped token", () => {
    assertRejected((copy) => {
      copy.permissions.contents = "write";
    });
  });

  it("rejects a missing source trigger", () => {
    assertRejected((copy) => {
      const events = triggers(copy);
      events.pull_request.paths = events.pull_request.paths.filter((path) => path !== "src/**");
    });
  });

  it("rejects a mutable benchmark action", () => {
    assertRejected((copy) => {
      const step = copy.jobs.benchmark.steps.find(
        (candidate) => candidate.uses === BENCHMARK_ACTION,
      );
      step.uses = "benchmark-action/github-action-benchmark@v1";
    });
  });

  it("rejects a relaxed regression threshold", () => {
    assertRejected((copy) => {
      const step = copy.jobs.benchmark.steps.find(
        (candidate) => candidate.uses === BENCHMARK_ACTION,
      );
      step.with["fail-threshold"] = "150%";
    });
  });

  it("rejects a pull request cache write", () => {
    assertRejected((copy) => {
      const step = copy.jobs.benchmark.steps.find(
        (candidate) => candidate.uses === CACHE_SAVE_ACTION,
      );
      step.if = "always()";
    });
  });

  it("rejects a comparison against a moving base revision", () => {
    assertRejected((copy) => {
      copy.jobs.benchmark.steps.find((step) => step.with?.path === ".benchmark-base").with.ref =
        "main";
    });
  });

  it("rejects an advisory-only pull request comparison", () => {
    assertRejected((copy) => {
      copy.jobs.benchmark.steps.find(
        (step) => step.name === "Compare with PR base measured on this runner",
      )["continue-on-error"] = true;
    });
  });

  it("rejects a missing bracket stress case", () => {
    assertRejected(() => {}, {
      benchmarkSource: readRepositoryFile("benches/parsing.rs").replace(
        'bench_function("bracket_explosion_64k"',
        'bench_function("removed"',
      ),
    });
  });

  it("rejects an undersized large fixture", () => {
    assertRejected(() => {}, { largeFixtureBytes: 1024 * 1024 - 1 });
  });
});
