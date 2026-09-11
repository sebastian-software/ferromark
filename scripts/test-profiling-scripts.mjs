import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import {
  chmodSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import process from "node:process";
import { describe, it } from "node:test";
import { setTimeout as delay } from "node:timers/promises";

import { readRepositoryFile, repositoryPath, repositoryRoot } from "./lib/contracts.mjs";

const SIMPLE = readRepositoryFile("scripts/profile_simple.sh");
const COMMONMARK = readRepositoryFile("scripts/profile_commonmark50k.sh");
const SHARED = readRepositoryFile("scripts/profile_common.sh");
const SIMPLE_SCRIPT = repositoryPath("scripts/profile_simple.sh");
const COMMONMARK_SCRIPT = repositoryPath("scripts/profile_commonmark50k.sh");
// Cargo separates CARGO_ENCODED_RUSTFLAGS entries with a unit separator.
const UNIT_SEPARATOR = "\u001f";

const FAKE_HARNESS = `#!/bin/sh
printf '%s\\n' "$$" >"$FAKE_CHILD_PID"
if [ "\${FAKE_HARNESS_STATUS:-0}" -ne 0 ]; then
  exit "$FAKE_HARNESS_STATUS"
fi
exec sleep 1000
`;

const FAKE_CARGO = `#!/bin/sh
pwd >"$FAKE_CARGO_CWD"
printf 'args=%s\\n' "$*" >"$FAKE_CARGO_ARGS"
printf 'target=%s\\n' "$CARGO_TARGET_DIR" >>"$FAKE_CARGO_ARGS"
printf 'encoded=%s\\n' "$CARGO_ENCODED_RUSTFLAGS" >>"$FAKE_CARGO_ARGS"
printf 'rustflags=%s\\n' "$RUSTFLAGS" >>"$FAKE_CARGO_ARGS"
printf 'md4c=%s\\n' "$MD4C_DIR" >>"$FAKE_CARGO_ARGS"
if [ "\${FAKE_CARGO_STATUS:-0}" -ne 0 ]; then
  echo 'fake cargo diagnostic' >&2
  exit "$FAKE_CARGO_STATUS"
fi
printf '{"reason":"compiler-artifact","target":{"name":"%s","kind":["%s"]},"executable":"%s"}\\n' "$FAKE_PROFILE_TARGET" "$FAKE_PROFILE_KIND" "$FAKE_PROFILE_BIN"
printf '{"reason":"compiler-artifact","target":{"name":"unrelated","kind":["example"]},"executable":"/bin/false"}\\n'
`;

const FAKE_SAMPLE = `#!/bin/sh
for arg; do output="$arg"; done
: >"$output"
if [ "\${FAKE_SAMPLE_SLEEP:-0}" -ne 0 ]; then
  printf '%s\\n' "$$" >"$FAKE_SAMPLE_PID"
  exec sleep "$FAKE_SAMPLE_SLEEP"
fi
if [ "\${FAKE_SAMPLE_STATUS:-0}" -ne 0 ]; then
  exit "$FAKE_SAMPLE_STATUS"
fi
exit 0
`;

// A freshly written executable may take longer than the production 100 ms
// startup pause to reach its first instruction (for example on macOS). This
// test-only clock waits for the requested early exit before advancing the pause.
const FAKE_SLEEP = `#!/bin/sh
if [ "\${FAKE_WAIT_FOR_HARNESS_EXIT:-0}" -eq 1 ] && [ "$1" = "0.1" ]; then
  attempts=0
  while [ "$attempts" -lt 200 ]; do
    if [ -f "$FAKE_CHILD_PID" ]; then
      state=$(ps -o stat= -p "$(cat "$FAKE_CHILD_PID")" 2>/dev/null || true)
      case "$state" in ""|*Z*) break ;; esac
    fi
    attempts=$((attempts + 1))
    /bin/sleep 0.01
  done
  if [ "$attempts" -eq 200 ]; then
    echo 'fake harness did not exit before the test deadline' >&2
    exit 1
  fi
fi
exec /bin/sleep "$@"
`;

function requireText(document, text) {
  assert.ok(document.includes(text), `Profiling script contract: missing ${JSON.stringify(text)}`);
}

function processAlive(pid) {
  try {
    process.kill(Number.parseInt(pid, 10), 0);
    return true;
  } catch {
    return false;
  }
}

function createFixture() {
  const directory = realpathSync(mkdtempSync(path.join(tmpdir(), "ferromark-profiling-contract.")));
  const files = {
    harness: path.join(directory, "fake-profile-harness"),
    cargo: path.join(directory, "cargo"),
    sample: path.join(directory, "sample"),
    sleep: path.join(directory, "sleep"),
  };
  writeFileSync(files.harness, FAKE_HARNESS);
  writeFileSync(files.cargo, FAKE_CARGO);
  writeFileSync(files.sample, FAKE_SAMPLE);
  writeFileSync(files.sleep, FAKE_SLEEP);
  for (const file of Object.values(files)) {
    chmodSync(file, 0o755);
  }
  const env = {
    ...process.env,
    PATH: `${directory}:${process.env.PATH}`,
    FAKE_PROFILE_BIN: files.harness,
    FAKE_PROFILE_TARGET: "profile_harness",
    FAKE_PROFILE_KIND: "example",
    FAKE_CARGO_CWD: path.join(directory, "cargo.cwd"),
    FAKE_CARGO_ARGS: path.join(directory, "cargo.args"),
    FAKE_CHILD_PID: path.join(directory, "child.pid"),
    FAKE_SAMPLE_PID: path.join(directory, "sample.pid"),
    PROFILE_OUTPUT_DIR: directory,
    CARGO_TARGET_DIR: "relative-target",
  };
  return { directory, env };
}

function withFixture(callback) {
  const fixture = createFixture();
  try {
    return callback(fixture);
  } finally {
    rmSync(fixture.directory, { force: true, recursive: true });
  }
}

function runProfile(script, args, { directory, env }, overrides = {}, removed = []) {
  const childEnv = { ...env, ...overrides };
  for (const key of removed) {
    delete childEnv[key];
  }
  return spawnSync(script, args, { cwd: directory, env: childEnv, encoding: "utf8" });
}

function cargoLog(directory) {
  return readFileSync(path.join(directory, "cargo.args"), "utf8");
}

async function waitFor(condition, message, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      if (condition()) {
        return;
      }
    } catch {
      // The profiling script may not have written its pid files yet.
    }
    await delay(50);
  }
  assert.fail(message);
}

async function withTimeout(promise, message, timeoutMs = 5000) {
  const result = await Promise.race([promise, delay(timeoutMs, "timeout")]);
  if (result === "timeout") {
    assert.fail(message);
  }
  return result;
}

describe("profiling script contract", () => {
  it("keeps the documented shared helpers and guards", () => {
    requireText(SIMPLE, "profile_build_harness");
    requireText(SIMPLE, "non-pgo");
    requireText(COMMONMARK, "profile_build_harness");
    requireText(COMMONMARK, "profile_build_comparison");
    requireText(SHARED, "--message-format=json-render-diagnostics");
    requireText(SHARED, "json.loads");
    requireText(SHARED, "MD4C_DIR");
    requireText(SHARED, "trap cleanup_profile_child EXIT");
    requireText(SHARED, "trap 'exit 130' INT");
    requireText(SHARED, "trap 'exit 143' TERM");
    requireText(SHARED, "trap 'exit 129' HUP");
    requireText(SHARED, "Failed to build ferromark profile_harness.");
    requireText(SHARED, "Failed to build the isolated cross-parser comparison bench.");
    assert.ok(
      !SIMPLE.includes("cargo bench --bench comparison"),
      "simple script still builds the removed root comparison bench",
    );
    assert.ok(
      SIMPLE.includes("BASH_SOURCE") && COMMONMARK.includes("BASH_SOURCE"),
      "scripts must resolve their repository from their own location",
    );
  });

  it("builds from the repository root and forwards the target directory", () => {
    withFixture((fixture) => {
      const result = runProfile(SIMPLE_SCRIPT, ["non-pgo", "0.1", "1"], fixture);
      assert.equal(
        result.status,
        0,
        `ferromark-only smoke failed: ${result.stdout}\n${result.stderr}`,
      );
      assert.equal(
        readFileSync(path.join(fixture.directory, "cargo.cwd"), "utf8").trimEnd(),
        repositoryRoot,
        "build did not run from the repository root",
      );
      const log = cargoLog(fixture.directory);
      assert.ok(
        log.includes(path.join(fixture.directory, "relative-target")),
        "custom CARGO_TARGET_DIR was not forwarded",
      );
      assert.ok(
        existsSync(path.join(fixture.directory, "ferromark-simple.sample.txt")),
        "sample output was not created",
      );
      assert.ok(log.includes("encoded=\n"), "non-PGO mode did not clear Cargo encoded flags");
      assert.ok(log.includes("rustflags=\n"), "non-PGO mode did not clear RUSTFLAGS");
    });
  });

  it("reports a failing cargo build", () => {
    withFixture((fixture) => {
      const result = runProfile(SIMPLE_SCRIPT, ["non-pgo", "0.1", "1"], fixture, {
        FAKE_CARGO_STATUS: "42",
      });
      assert.notEqual(result.status, 0, "cargo build failure was swallowed");
      requireText(result.stderr, "Failed to build ferromark profile_harness");
      requireText(result.stderr, "fake cargo diagnostic");
    });
  });

  it("stops the benchmark child when sampling fails", () => {
    withFixture((fixture) => {
      const result = runProfile(SIMPLE_SCRIPT, ["non-pgo", "0.1", "1"], fixture, {
        FAKE_SAMPLE_STATUS: "7",
      });
      assert.notEqual(result.status, 0, "sample failure was swallowed");
      requireText(result.stderr, "sample failed for PID");
      assert.ok(
        !processAlive(readFileSync(path.join(fixture.directory, "child.pid"), "utf8")),
        "sample failure left the benchmark child running",
      );
    });
  });

  it("rejects a benchmark that exits before sampling", () => {
    withFixture((fixture) => {
      const result = runProfile(SIMPLE_SCRIPT, ["non-pgo", "0.1", "1"], fixture, {
        FAKE_HARNESS_STATUS: "23",
        FAKE_WAIT_FOR_HARNESS_EXIT: "1",
      });
      assert.notEqual(
        result.status,
        0,
        `early benchmark exit was not rejected: ${result.stdout} ${result.stderr}`,
      );
      requireText(result.stderr, "Profiling child exited before sampling");
      assert.ok(
        !processAlive(readFileSync(path.join(fixture.directory, "child.pid"), "utf8")),
        "early benchmark exit left the child running",
      );
    });
  });

  it("stops every child on SIGTERM", async () => {
    const fixture = createFixture();
    try {
      const child = spawn(SIMPLE_SCRIPT, ["non-pgo", "10", "20"], {
        cwd: fixture.directory,
        env: { ...fixture.env, FAKE_SAMPLE_SLEEP: "1000" },
        stdio: "ignore",
      });
      const exited = new Promise((resolve) => {
        child.on("exit", (code, signal) => resolve({ code, signal }));
      });
      await waitFor(
        () => existsSync(path.join(fixture.directory, "sample.pid")),
        "SIGTERM test could not start the sampler",
      );
      child.kill("SIGTERM");
      const status = await withTimeout(exited, "SIGTERM did not stop the profiling script");
      assert.notEqual(status.code, 0, "SIGTERM was incorrectly reported as success");
      await waitFor(
        () =>
          !processAlive(readFileSync(path.join(fixture.directory, "child.pid"), "utf8")) &&
          !processAlive(readFileSync(path.join(fixture.directory, "sample.pid"), "utf8")),
        "SIGTERM left a benchmark or sample child running",
      );
    } finally {
      rmSync(fixture.directory, { force: true, recursive: true });
    }
  });

  it("rejects an empty or oversized sample budget", () => {
    withFixture((fixture) => {
      const zero = runProfile(SIMPLE_SCRIPT, ["non-pgo", "0", "1"], fixture);
      assert.notEqual(zero.status, 0, "zero sample duration was not rejected");
      requireText(zero.stderr, "sample_seconds must be a positive number");

      const overflow = runProfile(SIMPLE_SCRIPT, ["non-pgo", "2", "1"], fixture);
      assert.notEqual(overflow.status, 0, "sample budget overflow was not rejected");
      requireText(overflow.stderr, "must not exceed measurement_seconds");
    });
  });

  it("encodes PGO flags and requires the profile data", () => {
    withFixture((fixture) => {
      const profdata = path.join(fixture.directory, "profile data.profdata");
      writeFileSync(profdata, "profile");
      const result = runProfile(SIMPLE_SCRIPT, ["pgo", "0.1", "1"], fixture, {
        PGO_PROFDATA: "profile data.profdata",
      });
      assert.equal(result.status, 0, `PGO smoke failed: ${result.stdout}\n${result.stderr}`);
      const log = cargoLog(fixture.directory);
      const encoded = log
        .split("\n")
        .find((line) => line.startsWith("encoded="))
        .slice("encoded=".length);
      assert.ok(
        encoded.includes(`-Cprofile-use=${realpathSync(profdata)}`) &&
          encoded.includes(UNIT_SEPARATOR),
        "PGO flags were not encoded for Cargo",
      );
      assert.ok(
        log.includes("rustflags=\n"),
        "PGO mode did not clear the legacy RUSTFLAGS channel",
      );

      const missing = runProfile(SIMPLE_SCRIPT, ["pgo", "0.1", "1"], fixture, {}, ["PGO_PROFDATA"]);
      assert.notEqual(missing.status, 0, "missing PGO data was not rejected");
      requireText(missing.stderr, "PGO mode requires PGO_PROFDATA");
    });
  });

  it("profiles cross-parser comparisons only with md4c sources", () => {
    withFixture((fixture) => {
      const missing = runProfile(
        COMMONMARK_SCRIPT,
        ["5k", "pulldown-cmark", "0.1", "1", "non-pgo"],
        fixture,
        {},
        ["MD4C_DIR"],
      );
      assert.notEqual(
        missing.status,
        0,
        "cross-parser profiling without MD4C_DIR was not rejected",
      );
      requireText(missing.stderr, "Cross-parser profiling requires MD4C_DIR");

      const result = runProfile(
        COMMONMARK_SCRIPT,
        ["5k", "pulldown-cmark", "0.1", "1", "non-pgo"],
        fixture,
        { MD4C_DIR: ".", FAKE_PROFILE_TARGET: "comparison", FAKE_PROFILE_KIND: "bench" },
      );
      assert.equal(
        result.status,
        0,
        `cross-parser smoke failed: ${result.stdout}\n${result.stderr}`,
      );
      const log = cargoLog(fixture.directory);
      requireText(log, "benchmarks/md4c-comparison/Cargo.toml");
      assert.ok(
        log.includes(`md4c=${fixture.directory}\n`),
        "relative MD4C_DIR was not canonicalized before changing cwd",
      );
      assert.ok(
        existsSync(
          path.join(fixture.directory, "ferromark-commonmark5k-pulldown-cmark-non-pgo.sample.txt"),
        ),
        "cross-parser smoke did not create sample output",
      );
    });
  });
});
