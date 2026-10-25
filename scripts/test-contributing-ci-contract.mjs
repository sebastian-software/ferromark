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
    "node-musl",
    "rust-packages",
    "homepage",
    "cargo-deny",
    "coverage",
  ])
    assert.ok(ci.jobs[job]);
  assert.equal(ci.jobs.native.strategy.matrix.include.length, 7);
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

test("release packages target public registries", () => {
  assert.match(read("Cargo.toml"), /^publish = \["crates-io"\]$/m);
  const transform = TOML.parse(read("transforms/Cargo.toml"));
  const coreVersion = read("Cargo.toml").match(/^version = "([^"]+)"/m)[1];
  assert.equal(transform.package.name, "ferromark-transforms");
  assert.equal(transform.package.version, coreVersion);
  assert.deepEqual(transform.package.publish, ["crates-io"]);
  assert.deepEqual(transform.dependencies.ferromark, { version: `=${coreVersion}`, path: ".." });
  const packageJson = JSON.parse(read("node/ferromark/package.json"));
  assert.equal(packageJson.private, false);
  assert.equal(packageJson.version, coreVersion);
  for (const dependency of Object.keys(packageJson.optionalDependencies)) {
    const suffix = dependency.replace(/^ferromark-/, "");
    assert.equal(JSON.parse(read(`node/ferromark/npm/${suffix}/package.json`)).private, false);
  }
  assert.match(read("node/native/Cargo.toml"), /^publish = false$/m);
  const deploy = parse(read(".github/workflows/deploy-homepage.yml"));
  assert.equal(deploy.jobs.build.if, "github.ref == 'refs/heads/main'");
});

test("publication validates one immutable release source before publishing", () => {
  const publisher = parse(read(".github/workflows/publish.yml"));
  assert.deepEqual(Object.keys(publisher.on), ["push", "workflow_dispatch"]);
  assert.deepEqual(publisher.on.push.branches, ["main"]);
  assert.equal(publisher.jobs["release-please"].if, "${{ github.event_name == 'push' }}");
  assert.equal(publisher.on.workflow_dispatch.inputs.tag.required, true);
  assert.equal(
    publisher.jobs["release-please"].outputs.releases_created.includes("releases_created"),
    true,
  );
  assert.equal(publisher.jobs["release-please"].outputs.tag_name.includes("tag_name"), true);
  const releaseBranchCheckout = publisher.jobs["release-please"].steps.find(
    (step) => step.name === "Check out the Release Please branch",
  );
  const gha = (body) => "$" + "{{ " + body + " }}";
  assert.equal(releaseBranchCheckout.if, gha("steps.release.outputs.prs_created == 'true'"));
  assert.equal(
    releaseBranchCheckout.with.ref,
    gha("fromJSON(steps.release.outputs.pr || '{}').headBranchName"),
  );
  const pinStep = publisher.jobs["release-please"].steps.find(
    (step) => step.name === "Restore the exact transform dependency pin",
  );
  assert.equal(pinStep.if, gha("steps.release.outputs.prs_created == 'true'"));
  assert.equal(pinStep.env.RELEASE_BRANCH, releaseBranchCheckout.with.ref);
  assert.match(pinStep.run, /preserve-exact-transform-pin\.mjs/);
  assert.match(pinStep.run, /git push origin/);

  const validation = publisher.jobs["validate-release"];
  assert.equal(validation.needs, "release-please");
  assert.match(validation.if, /releases_created == 'true'/);
  assert.match(validation.if, /workflow_dispatch/);
  assert.equal(validation.permissions.contents, "read");
  assert.equal(validation.permissions["id-token"], undefined);
  assert.equal(validation.outputs.tag_name, "${{ steps.source.outputs.tag_name }}");
  assert.equal(validation.outputs.source_sha, "${{ steps.source.outputs.source_sha }}");

  const steps = validation.steps;
  const workflowCheckout = steps.find((step) => step.uses?.startsWith("actions/checkout@"));
  assert.equal(workflowCheckout.with.ref, gha("github.workflow_sha"));
  assert.equal(workflowCheckout.with["fetch-depth"], 0);
  const source = steps.find((step) => step.id === "source");
  const releaseRecord = steps.find(
    (step) => step.name === "Require a published GitHub release for the tag",
  );
  assert.match(source.run, /resolve-release-source\.mjs/);
  assert.equal(source.env.RELEASE_TAG, gha("inputs.tag || needs.release-please.outputs.tag_name"));
  assert.match(releaseRecord.run, /verify-github-release\.mjs/);
  assert.ok(steps.indexOf(source) < steps.indexOf(releaseRecord));
  assert.match(read("scripts/resolve-release-source.mjs"), /refs\/heads\/main/);
  assert.match(read("scripts/resolve-release-source.mjs"), /merge-base/);
  assert.match(read("scripts/verify-github-release.mjs"), /isDraft/);

  const versionGate = publisher.jobs["validate-release-version"];
  assert.equal(versionGate.needs, "validate-release");
  assert.equal(versionGate.permissions.contents, "read");
  assert.equal(versionGate.permissions["id-token"], undefined);
  assert.match(versionGate.if, /needs\.validate-release\.result == 'success'/);
  const versionCheck = versionGate.steps.find(
    (step) => step.name === "Verify the release tag matches the package versions",
  );
  assert.match(versionCheck.run, /verify-release\.mjs --metadata-only/);
  assert.match(versionCheck.run, /--root release-source/);
  const versionCheckouts = versionGate.steps.filter((step) =>
    step.uses?.startsWith("actions/checkout@"),
  );
  assert.equal(versionCheckouts.length, 2);
  assert.equal(versionCheckouts[0].with.ref, gha("github.workflow_sha"));
  assert.equal(versionCheckouts[0].with.path, "release-tools");
  assert.equal(versionCheckouts[0].with["persist-credentials"], false);
  assert.equal(versionCheckouts[1].with.ref, gha("needs.validate-release.outputs.source_sha"));
  assert.equal(versionCheckouts[1].with.path, "release-source");
  assert.equal(versionCheckouts[1].with["persist-credentials"], false);

  const sourceCheckoutRef = gha("needs.validate-release.outputs.source_sha");
  const workflowRef = gha("github.workflow_sha");
  for (const name of ["publish-crates", "build-native", "publish-npm", "verify-npm-publish"]) {
    const job = publisher.jobs[name];
    const checkouts = job.steps.filter((step) => step.uses?.startsWith("actions/checkout@"));
    const sourceCheckouts = checkouts.filter((step) => !step.with.path);
    assert.ok(sourceCheckouts.length > 0, name + ": checks out the validated release source");
    for (const step of sourceCheckouts) {
      assert.equal(step.with.ref, sourceCheckoutRef, name + ": uses the immutable source SHA");
    }

    const helperCheckouts = checkouts.filter((step) => step.with.path === "release-tools");
    const shouldLoadHelpers = name === "build-native" || name === "publish-npm";
    assert.equal(
      helperCheckouts.length,
      shouldLoadHelpers ? 1 : 0,
      name + ": helper checkout count",
    );
    for (const step of helperCheckouts) {
      assert.equal(step.with.ref, workflowRef, name + ": helper comes from the workflow commit");
      assert.equal(step.with["persist-credentials"], false);
      if (name === "build-native") {
        assert.equal(
          step.env.GIT_CONFIG_COUNT,
          gha("runner.os == 'Windows' && '1' || '0'"),
          name + ": disable NTFS protection only for the Windows checkout",
        );
        assert.equal(step.env.GIT_CONFIG_KEY_0, "core.protectNTFS");
        assert.equal(step.env.GIT_CONFIG_VALUE_0, "false");
      } else {
        assert.equal(step.env, undefined, name + ": no NTFS override on the Ubuntu checkout");
      }
    }
  }

  for (const name of [
    "publish-crates",
    "native-matrix",
    "build-native",
    "publish-npm",
    "verify-npm-publish",
  ]) {
    const needs = publisher.jobs[name].needs;
    const dependencies = Array.isArray(needs) ? needs : [needs];
    assert.ok(dependencies.includes("validate-release"), name + ": waits for source validation");
    assert.ok(
      dependencies.includes("validate-release-version"),
      name + ": waits for version validation",
    );
    assert.match(publisher.jobs[name].if, /needs\.validate-release\.result == 'success'/, name);
    assert.match(
      publisher.jobs[name].if,
      /needs\.validate-release-version\.result == 'success'/,
      name,
    );
  }
  for (const name of ["publish-crates", "publish-npm"]) {
    assert.equal(publisher.jobs[name].permissions["id-token"], "write");
    assert.match(publisher.jobs[name].if, /needs\.validate-release\.result == 'success'/);
    assert.match(publisher.jobs[name].if, /needs\.validate-release-version\.result == 'success'/);
  }

  const cratePublisher = publisher.jobs["publish-crates"].steps.find((step) =>
    step.uses?.includes("/publish-crates@"),
  );
  assert.deepEqual(cratePublisher.with.crates.trim().split(/\s+/), [
    "ferromark",
    "ferromark-transforms",
  ]);

  const uses = Object.values(publisher.jobs).flatMap((job) =>
    job.steps.map((step) => step.uses).filter(Boolean),
  );
  for (const action of ["publish-crates", "napi-matrix", "publish-npm"]) {
    assert.ok(
      uses.some(
        (entry) =>
          entry.startsWith("sebastian-software/standards/.github/actions/" + action + "@") &&
          /^[a-f0-9]{40}$/.test(entry.slice(entry.lastIndexOf("@") + 1)),
      ),
      action + " must come from the standards repository, pinned by commit",
    );
  }
  assert.equal(
    publisher.jobs["build-native"].strategy.matrix,
    "$" + "{{ fromJSON(needs.native-matrix.outputs.matrix) }}",
  );

  assert.equal(publisher.jobs["build-native"].strategy.matrix.include, undefined);
  const build = publisher.jobs["build-native"];
  const stage = build.steps.find(
    (step) => step.name === "Bind the native artifact to the validated source SHA",
  );
  const upload = build.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(stage.env.SOURCE_SHA, sourceCheckoutRef);
  assert.match(upload.with.path, /ferromark-source-/);

  const publishSteps = publisher.jobs["publish-npm"].steps;
  const provenance = publishSteps.findIndex(
    (step) => step.name === "Verify native artifact source identities",
  );
  const assemble = publishSteps.findIndex((step) => step.name === "Assemble all platform packages");
  const publish = publishSteps.findIndex((step) => step.uses?.includes("/publish-npm@"));
  assert.ok(provenance >= 0 && provenance < assemble);
  assert.ok(assemble < publish);
});

test("the release native matrix matches supported packages and CI targets", () => {
  const publisher = parse(read(".github/workflows/publish.yml"));
  const packageJson = JSON.parse(read("node/ferromark/package.json"));
  const matrixAction = publisher.jobs["native-matrix"].steps.find((step) =>
    step.uses?.includes("/napi-matrix@"),
  );
  assert.equal(matrixAction.with.package, packageJson.napi.packageName);

  const sidecars = Object.keys(packageJson.optionalDependencies)
    .map((name) => name.replace(/^ferromark-/, ""))
    .sort();
  const publishedPlatforms = matrixAction.with.platforms
    .trim()
    .split(/[\s,]+/)
    .sort();
  assert.deepEqual(
    publishedPlatforms,
    sidecars,
    "the release matrix must select exactly the facade's published sidecars",
  );

  const ciTargets = ci.jobs.native.strategy.matrix.include;
  assert.deepEqual(
    ciTargets.map(({ rust_target }) => rust_target).sort(),
    [...packageJson.napi.targets].sort(),
    "native CI targets must match the facade's supported N-API targets",
  );
  assert.deepEqual(
    ciTargets.map(({ artifact }) => artifact).sort(),
    sidecars,
    "native CI artifacts must match the facade's published sidecars",
  );
});

test("the packed musl addon is loaded on Alpine before merge and before publishing", () => {
  // The musl addons are cross-compiled, so only a musl container proves they
  // load. The rehearsal job installs the archives the assembly job uploaded.
  const musl = ci.jobs["node-musl"];
  const smoke = "node ./scripts/clean-install-smoke.mjs --target linux-x64-musl --package-tests";
  assert.equal(musl.needs, "npm-packages");
  assert.equal(musl.container, "node:22-alpine");
  assert.equal(musl.defaults.run.shell, "sh");
  const upload = ci.jobs["npm-packages"].steps.find((step) =>
    step.uses?.startsWith("actions/upload-artifact@"),
  );
  const download = musl.steps.find((step) => step.uses?.startsWith("actions/download-artifact@"));
  assert.equal(download.with.name, upload.with.name);
  assert.equal(download.with.path, "node/artifacts");
  assert.ok(musl.steps.some((step) => step.run === smoke));

  // The release runs the same check before the first registry call.
  const steps = parse(read(".github/workflows/publish.yml")).jobs["publish-npm"].steps;
  const alpine = steps.findIndex((step) => step.run?.includes(smoke));
  const publish = steps.findIndex((step) => step.uses?.includes("/publish-npm@"));
  assert.ok(alpine !== -1, "the release must load the musl addon on Alpine");
  assert.ok(alpine < publish, "the Alpine check must precede the npm publish");
});

test("Node native declarations follow the v2 option surface", () => {
  const declarations = read("node/ferromark/native.d.ts");
  for (const name of [
    "tableColgroup",
    "tableColumnNames",
    "tableAttributes",
    "headingAttributes",
    "cjkEmphasis",
    "mdx",
    "highlight",
    "inlineFootnotes",
    "allowLinkRefs",
  ])
    assert.ok(declarations.includes(`${name}?`));
  for (const name of ["tableColumnWidths", "indentedCodeBlocks", "wikiLinks"])
    assert.ok(!declarations.includes(`${name}?`));
  assert.ok(!declarations.includes("CodeCallback"), "callback types must be self-contained");
});

test("the core and transform packages are public with release-managed path dependencies", () => {
  const policy = TOML.parse(read("deny.toml"));
  assert.equal(policy.bans.wildcards, "deny");
  assert.notEqual(policy.bans["allow-wildcard-paths"], true);
  const root = TOML.parse(read("Cargo.toml"));
  const workspace = root.workspace;
  // `release-type: rust` updates the root `[package]`, the members below it and
  // their explicit path requirements, so `ferromark` is the root package.
  assert.equal(root.package.name, "ferromark");
  assert.deepEqual(workspace.members, [
    "node/native",
    "node/plugin-fixture",
    "plugins/ferrigura",
    "transforms",
  ]);
  const publicMembers = workspace.members
    .map((member) => [member, TOML.parse(read(`${member}/Cargo.toml`))])
    .filter(([, manifest]) => manifest.package.publish !== false)
    .map(([, manifest]) => manifest.package.name);
  assert.deepEqual(publicMembers, ["ferromark-transforms"]);
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

test("Node and homepage CI enforce their declared formatter and lint scripts", () => {
  for (const workspace of ["node", "homepage"]) {
    const { scripts, devDependencies } = JSON.parse(read(`${workspace}/package.json`));
    assert.match(scripts["format:check"], /\boxfmt\s+--check\b/);
    assert.match(scripts.lint, /\boxlint\b/);
    assert.match(scripts.lint, /\beslint\b/);
    assert.ok(devDependencies.oxfmt);
    assert.ok(devDependencies.oxlint);
    assert.ok(devDependencies["eslint-config-setup"]);
    for (const command of ["pnpm format:check", "pnpm lint"]) {
      assert.ok(
        ci.jobs[workspace].steps.some((step) => step.run === command),
        `${workspace}: ${command}`,
      );
    }
  }
});
