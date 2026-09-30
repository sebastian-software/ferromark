import assert from "node:assert/strict";
import { test } from "node:test";
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
