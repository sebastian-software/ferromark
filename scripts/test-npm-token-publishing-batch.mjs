import assert from "node:assert/strict";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";

const script = new URL("./npm-token-publishing-batch.sh", import.meta.url).pathname;
const mockNpm = [
  "#!/usr/bin/env node",
  "const fs = require('node:fs')",
  "const args = process.argv.slice(2)",
  "fs.appendFileSync(process.env.MOCK_NPM_LOG, JSON.stringify(args) + '\\n')",
  "if (args.includes('whoami')) { process.stdout.write('maintainer\\n'); process.exit(0) }",
  "if (args.includes('access') && args.includes('list')) {",
  "  const i = args.indexOf('packages')",
  "  const selector = args[i + 1] === '--json' ? '' : args[i + 1]",
  "  const file = selector ? process.env.MOCK_SCOPE_MAP : process.env.MOCK_ACCESS_MAP",
  "  process.stdout.write(fs.readFileSync(file, 'utf8')); process.exit(0)",
  "}",
  "if (args.includes('trust') && args.includes('list')) {",
  "  const pkg = args[args.indexOf('list') + 1]",
  "  const trustCalls = fs.readFileSync(process.env.MOCK_NPM_LOG, 'utf8').trim().split('\\n').filter((line) => line && JSON.parse(line).includes('trust')).length",
  "  const trustMap = JSON.parse(fs.readFileSync(process.env.MOCK_TRUST_MAP, 'utf8'))",
  "  if (Number(process.env.MOCK_CHANGE_TRUST_ON_CALL) === trustCalls) { trustMap[pkg] = (trustMap[pkg] || []).map((record) => ({ ...record, file: 'changed.yml' })); fs.writeFileSync(process.env.MOCK_TRUST_MAP, JSON.stringify(trustMap)) }",
  "  const records = trustMap[pkg] || []",
  "  for (const record of records) process.stdout.write(JSON.stringify(record, null, 2) + '\\n')",
  "  process.exit(0)",
  "}",
  "if (args.includes('access') && args.includes('set')) {",
  "  const pkg = args[args.indexOf('set') + 2]",
  "  fs.appendFileSync(process.env.MOCK_WRITE_LOG, pkg + '\\n')",
  "  if (pkg === process.env.MOCK_FAIL_PACKAGE) process.exit(9)",
  "  process.exit(0)",
  "}",
  "process.stderr.write('unexpected npm command: ' + JSON.stringify(args) + '\\n')",
  "process.exit(2)",
].join("\n");

const direct = {
  type: "github",
  repository: "team/release-one",
  file: "publish.yml",
  permissions: ["createPackage"],
};
const staged = {
  type: "github",
  repository: "team/release-one",
  file: "publish.yml",
  permissions: ["createStagedPackage"],
};
const accountPackages = {
  "pkg-direct": "read-write",
  "pkg-stage": "read-write",
  "pkg-untrusted": "read-write",
  "pkg-readonly": "read-only",
};
const scopePackages = { "pkg-scope": "read-write" };
const trustRecords = {
  "pkg-direct": [direct, staged],
  "pkg-stage": [staged],
  "pkg-untrusted": [],
  "pkg-readonly": [direct],
  "pkg-scope": [direct],
};

function harness(t) {
  const root = mkdtempSync(join(tmpdir(), "npm-token-publishing-test-"));
  const bin = join(root, "bin");
  mkdirSync(bin);
  const npmPath = join(bin, "npm");
  const accessPath = join(root, "access.json");
  const scopePath = join(root, "scope.json");
  const trustPath = join(root, "trust.json");
  const inventoryPath = join(root, "packages.txt");
  const logPath = join(root, "npm.log");
  const writePath = join(root, "writes.log");
  writeFileSync(npmPath, mockNpm);
  chmodSync(npmPath, 0o755);
  writeFileSync(accessPath, JSON.stringify(accountPackages));
  writeFileSync(scopePath, JSON.stringify(scopePackages));
  writeFileSync(trustPath, JSON.stringify(trustRecords));
  writeFileSync(
    inventoryPath,
    [
      "# Explicit inventory can span repositories.",
      "pkg-direct",
      "pkg-stage",
      "pkg-untrusted",
      "pkg-scope",
      "pkg-readonly",
      "pkg-direct",
      "",
    ].join("\n"),
  );
  const env = {
    ...process.env,
    PATH: bin + ":" + process.env.PATH,
    MOCK_NPM_LOG: logPath,
    MOCK_WRITE_LOG: writePath,
    MOCK_ACCESS_MAP: accessPath,
    MOCK_SCOPE_MAP: scopePath,
    MOCK_TRUST_MAP: trustPath,
  };
  t.after(() => rmSync(root, { recursive: true, force: true }));
  return {
    root,
    inventoryPath,
    logPath,
    writePath,
    trustPath,
    run(args, options = {}) {
      return spawnSync("/bin/bash", [script, ...args], {
        cwd: root,
        env: { ...env, ...options.env },
        input: options.input,
        encoding: "utf8",
      });
    },
  };
}

function writeValidated(path, names) {
  writeFileSync(path, names.join("\n") + (names.length ? "\n" : ""));
}

test("preview is read-only, deduplicates inventory, and leaves Valid status for human review", (t) => {
  const h = harness(t);
  const planPath = join(h.root, "review-plan.json");
  const defaultPreview = h.run(["--packages-file", h.inventoryPath]);
  assert.equal(defaultPreview.status, 0, defaultPreview.stderr);
  assert.equal(defaultPreview.stdout.includes("Review plan written"), false);
  assert.equal(existsSync(h.writePath), false);

  const result = h.run([
    "--owner",
    "@team",
    "--packages-file",
    h.inventoryPath,
    "--plan",
    planPath,
  ]);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /REVIEW pkg-direct: direct,staged OIDC publish/);
  assert.match(result.stdout, /SKIP pkg-untrusted: no Trusted Publisher/);
  assert.match(result.stdout, /SKIP pkg-readonly: npm account does not have read-write access/);
  assert.equal(existsSync(h.writePath), false);
  assert.match(readFileSync(h.logPath, "utf8"), /\"@team\"/);
  const plan = JSON.parse(readFileSync(planPath, "utf8"));
  assert.deepEqual(plan.owners, ["@team"]);
  assert.equal(plan.packages.filter((pkg) => pkg.name === "pkg-direct").length, 1);
  assert.equal(plan.packages.find((pkg) => pkg.name === "pkg-direct").trust_records.length, 2);
  assert.equal("review" in plan.packages.find((pkg) => pkg.name === "pkg-direct"), false);
});

test("apply skips unreviewed and changed publishers before writing", (t) => {
  const h = harness(t);
  writeFileSync(h.inventoryPath, "pkg-direct\n");
  const planPath = join(h.root, "review-plan.json");
  const validatedPath = join(h.root, "validated.txt");
  const statePath = join(h.root, "state.tsv");
  const preview = h.run(["--packages-file", h.inventoryPath, "--plan", planPath]);
  assert.equal(preview.status, 0, preview.stderr);

  writeValidated(validatedPath, []);
  const unreviewed = h.run(
    ["--apply", "--plan", planPath, "--validated-packages", validatedPath, "--state", statePath],
    { input: "APPLY\n" },
  );
  assert.equal(unreviewed.status, 0, unreviewed.stderr);
  assert.match(unreviewed.stdout, /No packages were marked as UI-validated/);
  assert.equal(existsSync(h.writePath), false);
  assert.equal(readFileSync(h.logPath, "utf8").includes('\"set\"'), false);

  writeValidated(validatedPath, ["pkg-direct"]);
  const changed = JSON.parse(readFileSync(h.trustPath, "utf8"));
  changed["pkg-direct"] = [{ ...direct, file: "other.yml" }];
  writeFileSync(h.trustPath, JSON.stringify(changed));
  const result = h.run(
    ["--apply", "--plan", planPath, "--validated-packages", validatedPath, "--state", statePath],
    { input: "APPLY\n" },
  );
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /pkg-direct: Trusted Publisher records changed after review/);
  assert.equal(existsSync(h.writePath), false);
  assert.equal(readFileSync(h.logPath, "utf8").includes('\"set\"'), false);
});

test("apply revalidates publisher records immediately before each write", (t) => {
  const h = harness(t);
  writeFileSync(h.inventoryPath, "pkg-direct\n");
  const planPath = join(h.root, "review-plan.json");
  const validatedPath = join(h.root, "validated.txt");
  const statePath = join(h.root, "state.tsv");
  const preview = h.run(["--packages-file", h.inventoryPath, "--plan", planPath]);
  assert.equal(preview.status, 0, preview.stderr);
  writeValidated(validatedPath, ["pkg-direct"]);

  const result = h.run(
    ["--apply", "--plan", planPath, "--validated-packages", validatedPath, "--state", statePath],
    {
      input: "APPLY\n",
      env: { MOCK_CHANGE_TRUST_ON_CALL: "3" },
    },
  );
  assert.notEqual(result.status, 0);
  assert.match(
    result.stderr,
    /Trusted Publisher records or permissions changed for pkg-direct; stopped before this write/,
  );
  assert.equal(existsSync(h.writePath), false);
});

test("apply stops on error and resumes successful packages without repeating their writes", (t) => {
  const h = harness(t);
  writeFileSync(h.inventoryPath, "pkg-direct\npkg-stage\n");
  const planPath = join(h.root, "review-plan.json");
  const validatedPath = join(h.root, "validated.txt");
  const statePath = join(h.root, "state.tsv");
  const preview = h.run(["--packages-file", h.inventoryPath, "--plan", planPath]);
  assert.equal(preview.status, 0, preview.stderr);
  writeValidated(validatedPath, ["pkg-direct", "pkg-stage"]);

  const first = h.run(
    ["--apply", "--plan", planPath, "--validated-packages", validatedPath, "--state", statePath],
    {
      input: "APPLY\n",
      env: { MOCK_FAIL_PACKAGE: "pkg-stage" },
    },
  );
  assert.notEqual(first.status, 0);
  assert.match(first.stderr, /npm command failed for pkg-stage; stopped here/);
  assert.match(readFileSync(statePath, "utf8"), /pkg-direct\t.*\tCOMMAND_OK/);
  assert.deepEqual(readFileSync(h.writePath, "utf8").trim().split("\n"), [
    "pkg-direct",
    "pkg-stage",
  ]);

  const resumed = h.run(
    ["--apply", "--plan", planPath, "--validated-packages", validatedPath, "--state", statePath],
    { input: "APPLY\n" },
  );
  assert.equal(resumed.status, 0, resumed.stderr);
  assert.match(resumed.stdout, /SKIP pkg-direct: command already succeeded/);
  assert.match(resumed.stdout, /COMMAND_OK pkg-stage/);
  assert.deepEqual(readFileSync(h.writePath, "utf8").trim().split("\n"), [
    "pkg-direct",
    "pkg-stage",
    "pkg-stage",
  ]);
  const logged = readFileSync(h.logPath, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const writes = logged.filter((args) => args.includes("set"));
  assert.deepEqual(
    writes.map((args) => args.slice(-2)),
    [
      ["mfa=publish", "pkg-direct"],
      ["mfa=publish", "pkg-stage"],
      ["mfa=publish", "pkg-stage"],
    ],
  );
  assert.ok(writes.every((args) => !args.some((arg) => arg.startsWith("--otp"))));
});
