import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

const read = (file) => readFileSync(new URL(`../${file}`, import.meta.url), "utf8");

test("custom bundles cover the existing target matrix and test installed native packages", () => {
  const workflow = parse(read(".github/workflows/plugin-bundles.yml"));
  const ci = parse(read(".github/workflows/ci.yml"));
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.deepEqual(workflow.on.workflow_dispatch.inputs.plugins.options, [
    "none",
    "ferriki",
    "ferrigura",
    "all",
  ]);
  assert.deepEqual(
    workflow.jobs.build.strategy.matrix.include.map((entry) => entry.rust_target).sort(),
    ci.jobs.native.strategy.matrix.include.map((entry) => entry.rust_target).sort(),
  );
  const steps = workflow.jobs.build.steps;
  assert.ok(steps.some((step) => step.run?.includes("build-plugin-bundle.mjs")));
  assert.ok(steps.some((step) => step.run?.includes("verify-plugin-bundle.mjs")));
  assert.ok(
    workflow.jobs["musl-runtime"].steps.some((step) =>
      step.run?.includes("verify-plugin-bundle.mjs"),
    ),
  );
  assert.ok(!read(".github/workflows/plugin-bundles.yml").includes("npm publish"));
});
