import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import TOML from "@iarna/toml";

const read = (name) => readFileSync(new URL(`../${name}`, import.meta.url), "utf8");
const ci = parse(read(".github/workflows/ci.yml"));

test("CI covers the v2 workspace, toolchain floor, Node package, and site", () => {
  const rustVersion = read("Cargo.toml").match(/^rust-version = "([^"]+)"/m)[1];
  assert.ok(ci.jobs.test.strategy.matrix.rust.includes(rustVersion.replace(/\.0$/, "")));
  assert.deepEqual(ci.on.push.branches, ["main", "codex/v2"]);
  for (const command of [
    "cargo test --workspace --all-features --locked",
    "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
    "cargo bench --workspace --no-run --locked",
    "cargo fmt --all --check",
  ]) {
    assert.ok(read("CONTRIBUTING.md").includes(command));
    assert.ok(
      Object.values(ci.jobs).some((job) => job.steps?.some((step) => step.run === command)),
    );
  }
  for (const job of [
    "node",
    "node-floor",
    "native",
    "npm-packages",
    "rust-packages",
    "homepage",
    "standards",
    "cargo-deny",
    "coverage",
  ])
    assert.ok(ci.jobs[job]);
  assert.equal(ci.jobs.native.strategy.matrix.include.length, 8);
  assert.equal(ci.jobs["npm-packages"].needs, "native");
  assert.ok(
    ci.jobs["npm-packages"].steps.some(
      (step) => step.run === "node ./scripts/verify-pack.mjs --all-targets",
    ),
  );
  assert.ok(
    ci.jobs["rust-packages"].steps.some((step) =>
      step.run?.startsWith("python3 scripts/rehearse-rust-packages.py "),
    ),
  );
  assert.ok(ci.jobs.test.steps.some((step) => step.run === 'rustup override set "$TOOLCHAIN"'));
});

test("coverage retains the report before enforcing the floor", () => {
  const coverage = ci.jobs.coverage;
  const generate = coverage.steps.find((step) => step.name === "Generate LCOV coverage report");
  const upload = coverage.steps.find((step) => step.name === "Upload LCOV coverage report");
  const link = coverage.steps.find((step) => step.name === "Link retained coverage report");
  const enforce = coverage.steps.find((step) => step.name === "Enforce line coverage floor");

  assert.equal(generate.run.trim(), "cargo llvm-cov report --lcov --output-path lcov.info");
  assert.equal(upload.if, "${{ always() }}");
  assert.match(upload.uses, /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(upload.with, {
    name: "rust-coverage",
    path: "lcov.info",
    "if-no-files-found": "error",
    "retention-days": 30,
  });
  assert.equal(link.if, "${{ always() && steps.upload-coverage.outputs.artifact-url != '' }}");
  assert.match(link.run, /steps\.upload-coverage\.outputs\.artifact-url/);
  assert.equal(enforce.run, 'cargo llvm-cov report --fail-under-lines "$COVERAGE_FLOOR"');

  const generateIndex = coverage.steps.indexOf(generate);
  const uploadIndex = coverage.steps.indexOf(upload);
  const enforceIndex = coverage.steps.indexOf(enforce);
  assert.ok(generateIndex < uploadIndex && uploadIndex < enforceIndex);
  assert.equal(coverage.env.COVERAGE_FLOOR, "90");
  assert.match(read("CONTRIBUTING.md"), /fail-under-lines 90/);
});

test("release packages target public registries and publication is main-only", () => {
  assert.match(read("Cargo.toml"), /^publish = \["crates-io"\]$/m);
  const packageJson = JSON.parse(read("node/ferromark/package.json"));
  assert.equal(packageJson.private, false);
  assert.equal(packageJson.version, read("Cargo.toml").match(/^version = "([^"]+)"/m)[1]);
  for (const dependency of Object.keys(packageJson.optionalDependencies)) {
    const suffix = dependency.replace(/^ferromark-/, "");
    assert.equal(JSON.parse(read(`node/ferromark/npm/${suffix}/package.json`)).private, false);
  }
  const publisher = parse(read(".github/workflows/publish.yml"));
  assert.equal(publisher.jobs.publish.if, "github.ref == 'refs/heads/main'");
  assert.deepEqual(Object.keys(publisher.on), ["workflow_dispatch"]);
  assert.match(read("node/native/Cargo.toml"), /^publish = false$/m);
  const deploy = parse(read(".github/workflows/deploy-homepage.yml"));
  assert.equal(deploy.jobs.build.if, "github.ref == 'refs/heads/main'");
});

test("Node native declarations follow the v2 option surface", () => {
  const declarations = read("node/ferromark/native.d.ts");
  for (const name of [
    "tableColgroup",
    "tableColumnNames",
    "tableAttributes",
    "headingAttributes",
    "wikiLinks",
    "cjkEmphasis",
    "mdx",
    "highlight",
    "inlineFootnotes",
    "allowLinkRefs",
  ])
    assert.ok(declarations.includes(`${name}?`));
  for (const name of ["tableColumnWidths", "indentedCodeBlocks"])
    assert.ok(!declarations.includes(`${name}?`));
  assert.ok(!declarations.includes("CodeCallback"), "callback types must be self-contained");
});

test("only ferromark is public and no path-only dependency exceptions remain", () => {
  const policy = TOML.parse(read("deny.toml"));
  assert.equal(policy.bans.wildcards, "deny");
  assert.notEqual(policy.bans["allow-wildcard-paths"], true);
  const root = TOML.parse(read("Cargo.toml"));
  const workspace = root.workspace;
  // `release-type: rust` updates the root `[package]`, the members below it and
  // their explicit path requirements, so `ferromark` is the root package.
  assert.equal(root.package.name, "ferromark");
  assert.deepEqual(workspace.members, ["node/native"]);
  const unversioned = [];
  for (const member of [".", ...workspace.members]) {
    const manifest = member === "." ? root : TOML.parse(read(`${member}/Cargo.toml`));
    for (const section of ["dependencies", "dev-dependencies", "build-dependencies"]) {
      for (const [name, dependency] of Object.entries(manifest[section] ?? {})) {
        const spec = dependency.workspace ? workspace.dependencies[name] : dependency;
        if (spec.path && !spec.version) unversioned.push(`${member}:${section}:${name}`);
      }
    }
  }
  assert.deepEqual(unversioned, []);
});
