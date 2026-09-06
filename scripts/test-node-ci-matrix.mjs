import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { readYaml } from "./lib/contracts.mjs";

const ZIG_ACTION = "mlugg/setup-zig@d1434d08867e3ee9daa34448df10607b98908d29";
const INSTALL_ACTION = "taiki-e/install-action@e67fa11c4b9316fa714ddf0abed07a0c3143b95b";
const EXPECTED_TARGETS = [
  {
    os: "macos-latest",
    rust_target: "aarch64-apple-darwin",
    artifact: "darwin-arm64",
    runtime_test: true,
  },
  {
    os: "macos-15-intel",
    rust_target: "x86_64-apple-darwin",
    artifact: "darwin-x64",
    runtime_test: true,
  },
  {
    os: "ubuntu-24.04-arm",
    rust_target: "aarch64-unknown-linux-gnu",
    artifact: "linux-arm64-gnu",
    runtime_test: true,
  },
  {
    os: "ubuntu-latest",
    rust_target: "x86_64-unknown-linux-gnu",
    artifact: "linux-x64-gnu",
    runtime_test: true,
  },
  {
    os: "ubuntu-latest",
    rust_target: "x86_64-unknown-linux-musl",
    artifact: "linux-x64-musl",
    runtime_test: false,
  },
  {
    os: "ubuntu-latest",
    rust_target: "aarch64-unknown-linux-musl",
    artifact: "linux-arm64-musl",
    runtime_test: false,
  },
  {
    os: "windows-latest",
    rust_target: "x86_64-pc-windows-msvc",
    artifact: "win32-x64-msvc",
    runtime_test: true,
  },
  {
    os: "windows-11-arm",
    rust_target: "aarch64-pc-windows-msvc",
    artifact: "win32-arm64-msvc",
    runtime_test: true,
  },
];

const workflow = readYaml(".github", "workflows", "ci.yml");
const jobs = workflow.jobs;

describe("native CI matrix", () => {
  it("tests the current Node release lines", () => {
    assert.deepEqual(
      jobs.node.strategy.matrix.node,
      ["22", "24"],
      "Node CI matrix must test the current Node 22 and 24 release lines",
    );
  });

  it("builds on the workspace runtime before testing the floor", () => {
    const steps = jobs["node-floor"].steps;
    const workspaceSetup = steps.findIndex((step) => step.with?.["node-version"] === "22.13.0");
    const build = steps.findIndex((step) => step.run === "pnpm build");
    const floorSetup = steps.findIndex((step) => step.with?.["node-version"] === "22.12.0");
    const floorTest = steps.findIndex(
      (step) => step.run === "node --test ferromark/test/index.test.mjs",
    );
    assert.ok(
      workspaceSetup !== -1 &&
        build !== -1 &&
        floorSetup !== -1 &&
        floorTest !== -1 &&
        workspaceSetup < build &&
        build < floorSetup &&
        floorSetup < floorTest,
      "Node floor CI must build on the workspace runtime before testing on Node 22.12.0",
    );
  });

  it("runs each native target on its own operating system", () => {
    assert.equal(
      jobs.native["runs-on"],
      "${{ matrix.os }}",
      "native CI job must run on the operating system selected by the matrix",
    );
    assert.deepEqual(
      jobs.native.permissions,
      { contents: "read" },
      "native CI job must use contents: read permissions only",
    );
    assert.deepEqual(
      jobs.native.strategy.matrix.include,
      EXPECTED_TARGETS,
      "native CI matrix must build every non-host platform, including Linux musl",
    );
  });

  it("builds, tests, and verifies every platform package", () => {
    const steps = jobs.native.steps;
    const build = steps.findIndex((step) => step.run === "pnpm build:native");
    const test = steps.findIndex((step) => step.run === "pnpm test");
    const verify = steps.findIndex(
      (step) => step.run === "node ./scripts/verify-platform-artifact.mjs ${{ matrix.artifact }}",
    );
    const zig = steps.findIndex((step) => step.uses === ZIG_ACTION);
    const zigbuild = steps.findIndex((step) => step.uses === INSTALL_ACTION);

    assert.notEqual(build, -1, "native CI job must build the N-API binding");
    assert.deepEqual(
      steps[build].env,
      { FERROMARK_RUST_TARGET: "${{ matrix.rust_target }}" },
      "native CI job must build the N-API binding for the matrix target",
    );
    assert.ok(
      test !== -1 && test > build,
      "native CI job must run pnpm test after building the N-API binding",
    );
    assert.equal(
      steps[test].if,
      "${{ matrix.runtime_test }}",
      "native CI job must skip runtime tests only for cross-compiled musl targets",
    );
    assert.ok(
      verify !== -1 && verify > build,
      "native CI job must verify the generated platform package",
    );
    assert.ok(zig !== -1, "native CI job must install Zig for musl targets");
    assert.equal(
      steps[zig].if,
      "${{ contains(matrix.rust_target, '-musl') }}",
      "native CI job must install Zig for musl targets",
    );
    assert.ok(zigbuild !== -1, "native CI job must install cargo-zigbuild for musl targets");
    assert.deepEqual(
      steps[zigbuild].with,
      { tool: "cargo-zigbuild" },
      "native CI job must install cargo-zigbuild for musl targets",
    );
  });
});
