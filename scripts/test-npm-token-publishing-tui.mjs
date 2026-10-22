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
import { dirname, join } from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

const script = fileURLToPath(new URL("./npm-token-publishing-tui.mjs", import.meta.url));
const mockNpm = [
  "#!/usr/bin/env node",
  "const fs = require('node:fs')",
  "const args = process.argv.slice(2)",
  "fs.appendFileSync(process.env.MOCK_NPM_LOG, JSON.stringify(args) + '\\n')",
  "if (args.includes('login')) { fs.writeFileSync(process.env.MOCK_AUTH, 'logged-in'); process.stdout.write('mock npm login complete\\n'); process.exit(0) }",
  "if (args.includes('whoami')) { if (process.env.MOCK_REQUIRE_LOGIN === '1' && !fs.existsSync(process.env.MOCK_AUTH)) process.exit(1); process.stdout.write('maintainer\\n'); process.exit(0) }",
  "if (args.includes('access') && args.includes('list')) { process.stdout.write(fs.readFileSync(process.env.MOCK_ACCESS_MAP, 'utf8')); process.exit(0) }",
  "if (args.includes('trust') && args.includes('list')) { const pkg = args[args.indexOf('list') + 1]; for (const record of (JSON.parse(fs.readFileSync(process.env.MOCK_TRUST_MAP, 'utf8'))[pkg] || [])) process.stdout.write(JSON.stringify(record, null, 2) + '\\n'); process.exit(0) }",
  "if (args.includes('access') && args.includes('set')) { const pkg = args[args.indexOf('set') + 2]; fs.appendFileSync(process.env.MOCK_WRITE_LOG, pkg + '\\n'); process.exit(0) }",
  "process.stderr.write('unexpected npm command: ' + JSON.stringify(args) + '\\n'); process.exit(2)",
].join("\n");

const direct = {
  type: "github",
  repository: "team/release-one",
  file: ".github/workflows/publish.yml",
  permissions: ["createPackage"],
};

function harness(t, { requireLogin = false } = {}) {
  const root = mkdtempSync(join(tmpdir(), "npm-token-publishing-tui-test-"));
  const bin = join(root, "bin");
  mkdirSync(bin);
  const npmPath = join(bin, "npm");
  const accessPath = join(root, "access.json");
  const trustPath = join(root, "trust.json");
  const npmLog = join(root, "npm.log");
  const writeLog = join(root, "writes.log");
  const authPath = join(root, "auth");
  const stateHome = join(root, "state-home");
  writeFileSync(npmPath, mockNpm);
  chmodSync(npmPath, 0o755);
  writeFileSync(
    accessPath,
    JSON.stringify({
      "pkg-direct": "read-write",
      "pkg-untrusted": "read-write",
      "pkg-readonly": "read-only",
    }),
  );
  writeFileSync(
    trustPath,
    JSON.stringify({ "pkg-direct": [direct], "pkg-readonly": [direct], "pkg-untrusted": [] }),
  );
  const env = {
    ...process.env,
    PATH: `${bin}:${process.env.PATH}`,
    XDG_STATE_HOME: stateHome,
    NPM_POLICY_TUI_TEST: "1",
    MOCK_NPM_LOG: npmLog,
    MOCK_WRITE_LOG: writeLog,
    MOCK_ACCESS_MAP: accessPath,
    MOCK_TRUST_MAP: trustPath,
    MOCK_AUTH: authPath,
    MOCK_REQUIRE_LOGIN: requireLogin ? "1" : "0",
  };
  t.after(() => rmSync(root, { recursive: true, force: true }));
  return {
    root,
    npmLog,
    writeLog,
    authPath,
    stateFile: join(stateHome, "ferromark", "npm-token-publishing", "state.tsv"),
    start(args = []) {
      const child = spawn(process.execPath, [script, ...args], {
        cwd: root,
        env,
        stdio: ["pipe", "pipe", "pipe"],
      });
      t.after(() => {
        if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
      });
      const session = { child, stdout: "", stderr: "" };
      child.stdout.setEncoding("utf8");
      child.stderr.setEncoding("utf8");
      child.stdout.on("data", (chunk) => {
        session.stdout += chunk;
      });
      child.stderr.on("data", (chunk) => {
        session.stderr += chunk;
      });
      session.send = (line) => child.stdin.write(`${line}\n`);
      session.waitFor = (text, from = 0) =>
        new Promise((resolvePromise, reject) => {
          const find = () => {
            const position = session.stdout.indexOf(text, from);
            if (position !== -1) {
              cleanup();
              resolvePromise(position);
              return true;
            }
            return false;
          };
          const onExit = (code) => {
            cleanup();
            reject(
              new Error(
                `process exited ${code} before output ${JSON.stringify(text)}\n${session.stdout}\n${session.stderr}`,
              ),
            );
          };
          const onData = () => find();
          const timer = setTimeout(() => {
            cleanup();
            reject(
              new Error(
                `timed out waiting for ${JSON.stringify(text)}\n${session.stdout}\n${session.stderr}`,
              ),
            );
          }, 8000);
          const cleanup = () => {
            clearTimeout(timer);
            child.off("exit", onExit);
            child.stdout.off("data", onData);
          };
          if (!find()) {
            child.once("exit", onExit);
            child.stdout.on("data", onData);
          }
        });
      session.exit = () => {
        if (child.exitCode !== null)
          return Promise.resolve({ code: child.exitCode, signal: child.signalCode });
        return new Promise((resolvePromise, reject) => {
          const timer = setTimeout(() => {
            child.kill("SIGKILL");
            reject(new Error(`process did not exit\n${session.stdout}\n${session.stderr}`));
          }, 8000);
          child.once("exit", (code, signal) => {
            clearTimeout(timer);
            resolvePromise({ code, signal });
          });
        });
      };
      return session;
    },
  };
}

test("interactive preview shows configured claims and unknown status without changing npm settings", async (t) => {
  const h = harness(t);
  const session = h.start();
  await session.waitFor("npm-policy> ");
  assert.match(session.stdout, /OIDC: direct/);
  assert.match(session.stdout, /Valid: UNKNOWN/);
  assert.match(session.stdout, /current Publishing access: UNKNOWN/);
  assert.match(session.stdout, /\[-\] pkg-untrusted/);
  assert.match(session.stdout, /skipped: no Trusted Publisher/);

  session.send("show 1");
  await session.waitFor("team/release-one");
  session.send("toggle 2");
  await session.waitFor("pkg-untrusted is not eligible");
  session.send("quit");
  assert.deepEqual(await session.exit(), { code: 0, signal: null });
  assert.equal(existsSync(h.writeLog), false);
  assert.equal(existsSync(h.stateFile), false);
  const calls = readFileSync(h.npmLog, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  assert.ok(calls.some((args) => args.includes("trust") && args.includes("list")));
  assert.ok(calls.every((args) => !args.some((arg) => arg.startsWith("--otp"))));
});

test("per-package unknown acknowledgement gates apply and successful commands resume", async (t) => {
  const h = harness(t);
  const session = h.start();
  await session.waitFor("npm-policy> ");
  session.send("toggle 1");
  await session.waitFor("1. [x] pkg-direct");
  session.send("apply");
  await session.waitFor("Review and acknowledge unknown status for: pkg-direct");
  assert.equal(existsSync(h.writeLog), false);

  session.send("review 1");
  await session.waitFor("Type ACK UNKNOWN:");
  session.send("ACK VALID");
  await session.waitFor("pkg-direct: remains unacknowledged");
  session.send("apply");
  await session.waitFor("Review and acknowledge unknown status for: pkg-direct");

  const secondReviewFrom = session.stdout.length;
  session.send("review 1");
  await session.waitFor("Type ACK UNKNOWN:", secondReviewFrom);
  session.send("ACK UNKNOWN");
  await session.waitFor("unknown npm UI status acknowledged and selected");
  session.send("apply");
  await session.waitFor("Type APPLY UNKNOWN to continue:");
  session.send("APPLY UNKNOWN");
  await session.waitFor("COMMAND_OK pkg-direct");
  await session.waitFor("registry readback UNKNOWN");
  assert.deepEqual(readFileSync(h.writeLog, "utf8").trim().split("\n"), ["pkg-direct"]);
  assert.match(readFileSync(h.stateFile, "utf8"), /pkg-direct\t.*\tCOMMAND_OK/);
  session.send("quit");
  assert.deepEqual(await session.exit(), { code: 0, signal: null });
});

test("interactive login is offered when npm CLI authentication is missing", async (t) => {
  const h = harness(t, { requireLogin: true });
  const session = h.start();
  await session.waitFor("Run npm login interactively now?");
  session.send("y");
  await session.waitFor("mock npm login complete");
  await session.waitFor("npm-policy> ");
  assert.equal(existsSync(h.authPath), true);
  session.send("quit");
  assert.deepEqual(await session.exit(), { code: 0, signal: null });
  assert.equal(existsSync(h.writeLog), false);
});

test("malformed persisted state fails closed before any npm write", async (t) => {
  const h = harness(t);
  mkdirSync(dirname(h.stateFile), { recursive: true });
  writeFileSync(h.stateFile, "package\tfingerprint\tresult\npkg-direct\tbroken\tCOMMAND_OK\n");
  const session = h.start();
  assert.deepEqual(await session.exit(), { code: 1, signal: null });
  assert.match(session.stderr, /malformed state row/);
  assert.equal(existsSync(h.writeLog), false);
});
