import assert from "node:assert/strict";
import { test } from "node:test";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { readRepositoryFile, readYaml } from "./lib/contracts.mjs";

const workflow = readYaml(".github", "workflows", "blacksmith-benchmarks.yml");
const native = readYaml(".github", "workflows", "native-comparison.yml");

test("Blacksmith jobs require manual dispatch and stay in the requested organization", () => {
  assert.deepEqual(Object.keys(workflow.on).sort(), ["workflow_call", "workflow_dispatch"]);
  assert.equal(workflow.permissions.contents, "read");
  assert.equal(workflow.jobs.compare.if, "github.repository == 'sebastian-software/ferromark'");
  assert.equal(workflow.on.workflow_dispatch.inputs.mode.default, "probe");
  assert.equal(native.on.workflow_dispatch.inputs.runner_provider.default, "github");
  assert.ok(native.jobs.blacksmith.if.includes("github.event_name == 'workflow_dispatch'"));
  assert.ok(native.jobs.blacksmith.if.includes("inputs.runner_provider == 'blacksmith'"));
  assert.equal(native.jobs.compare.if, "inputs.runner_provider != 'blacksmith'");
});

test("resource gates finish before any build and only explicit measurement mode times", () => {
  const job = workflow.jobs.compare;
  assert.equal(job.strategy["max-parallel"], 1);
  const guard = job.steps.findIndex((step) => step.run?.includes("runner_context.py"));
  const setup = job.steps.findIndex((step) => step.uses?.startsWith("actions/setup-node@"));
  assert.ok(guard >= 0 && guard < setup);
  const timing = job.steps.filter((step) => step.run?.includes("benchmark-comparison run "));
  assert.equal(timing.length, 1);
  assert.equal(timing[0].if, "inputs.mode == 'measure'");
  assert.ok(timing[0].run.includes('publish "$BENCH_OUTPUT/suite" --check'));
  assert.equal(
    job.steps.find((step) => step.uses?.startsWith("actions/checkout@")).with["fetch-depth"],
    0,
  );
  const artifact = job.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(artifact.if, "always()");
  assert.ok(artifact.with.path.includes("/suite/evidence/"));
  assert.ok(artifact.with.path.includes("/runner-context.json"));
  assert.ok(
    !job.steps.some(
      (step) =>
        step.run?.includes("git push") ||
        (/publish.*--check/.test(step.run || "") === false &&
          /benchmark-comparison publish/.test(step.run || "")),
    ),
  );
});

test("workflow runner labels match the enforced profiles", () => {
  const context = readRepositoryFile("benchmarks", "manual-comparison", "runner_context.py");
  for (const label of ["blacksmith-6vcpu-macos-26", "blacksmith-4vcpu-ubuntu-2404"]) {
    assert.ok(workflow.jobs.compare["runs-on"].includes(label));
    assert.ok(context.includes(label));
  }
});

test("managed and per-pair workflows select main or extended candidates without emulation", () => {
  const candidates = JSON.parse(
    readRepositoryFile("benchmarks", "markdown-ecosystem", "comparisons.json"),
  );
  const pairs = readYaml(".github", "workflows", "markdown-ecosystem.yml");
  assert.equal(pairs.jobs.compare.needs, "plan");
  assert.equal(pairs.jobs.compare.strategy.matrix, "${{ fromJSON(needs.plan.outputs.matrix) }}");
  assert.equal(workflow.on.workflow_dispatch.inputs.scope.default, "main");
  assert.equal(pairs.on.workflow_dispatch.inputs.scope.default, "main");
  assert.equal(native.jobs.blacksmith.with.scope, "${{ inputs.blacksmith_scope }}");
  const planner = pairs.jobs.plan.steps.find((step) => step.id === "matrix");
  assert.ok(planner.run.includes('contracts.py --scope "$SCOPE"'));
  assert.equal(planner.env.SCOPE, "${{ inputs.scope || 'main' }}");
  const script = fileURLToPath(
    new URL("../benchmarks/markdown-ecosystem/contracts.py", import.meta.url),
  );
  for (const scope of ["main", "extended"]) {
    const matrix = JSON.parse(
      execFileSync("python3", [script, "--scope", scope], { encoding: "utf8" }),
    );
    assert.deepEqual(
      new Set(matrix.include.map(({ track, engine }) => `${track}/${engine}`)),
      new Set(
        candidates
          .filter(({ lane, optional }) => lane === "pair" && (scope === "extended" || !optional))
          .map(({ track, id }) => `${track}/${id}`),
      ),
    );
    assert.equal(matrix.include.length, scope === "main" ? 16 : 19);
  }
  for (const step of workflow.jobs.compare.steps.filter((step) =>
    /benchmark-comparison (prepare|run) /.test(step.run || ""),
  )) {
    assert.ok(step.run.includes('--scope "$SCOPE"'));
  }
  for (const job of [workflow.jobs.compare, pairs.jobs.compare]) {
    const go = job.steps.find((step) => step.uses?.startsWith("actions/setup-go@"));
    assert.ok(go);
    assert.equal(go.with["go-version"], "1.27.1");
    assert.equal(go.with.cache, false);
    assert.ok(!go.with.architecture);
  }
});
