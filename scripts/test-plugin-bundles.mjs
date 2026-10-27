import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import { NATIVE_TARGETS } from "../node/ferromark/native-target.mjs";

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
    NATIVE_TARGETS.map((entry) => entry.triple).sort(),
  );
  assert.equal(ci.jobs.native.needs, "native-matrix");
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
