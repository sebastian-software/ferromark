import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

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

test("development packages cannot publish and homepage deployment is main-only", () => {
  assert.match(read("Cargo.toml"), /^publish = false$/m);
  const packageJson = JSON.parse(read("node/ferromark/package.json"));
  assert.equal(packageJson.private, true);
  assert.equal(packageJson.version, read("Cargo.toml").match(/^version = "([^"]+)"/m)[1]);
  for (const dependency of Object.keys(packageJson.optionalDependencies)) {
    const suffix = dependency.replace(/^ferromark-/, "");
    assert.equal(JSON.parse(read(`node/ferromark/npm/${suffix}/package.json`)).private, true);
  }
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
