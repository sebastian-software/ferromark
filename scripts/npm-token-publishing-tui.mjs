#!/usr/bin/env node
import { spawn } from "node:child_process";
import {
  closeSync,
  existsSync,
  fchmodSync,
  fsyncSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { homedir, tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";

const registry = "https://registry.npmjs.org";
const helper = fileURLToPath(new URL("./npm-token-publishing-batch.sh", import.meta.url));
const packageRoot = fileURLToPath(new URL("..", import.meta.url));

function usage() {
  process.stdout.write(
    [
      "Usage: node scripts/npm-token-publishing-tui.mjs [--owner USER_OR_SCOPE ...] [--packages-file FILE] [--state FILE]",
      "Discovers npm packages and interactively reviews configured Trusted Publishers.",
      "The default state file is stored outside the repository; preview never changes npm settings.",
    ].join("\n") + "\n",
  );
}

function parseArgs(argv) {
  const options = { owners: [], packagesFile: undefined, stateFile: defaultStateFile() };
  for (let index = 0; index < argv.length; ) {
    const option = argv[index];
    if (option === "--help" || option === "-h") {
      usage();
      process.exit(0);
    }
    if (option === "--owner") {
      const owner = argv[index + 1];
      if (!owner || owner.startsWith("-") || /[\s]/u.test(owner)) {
        throw new Error("--owner requires a npm user or scope without whitespace");
      }
      options.owners.push(owner);
      index += 2;
      continue;
    }
    if (option === "--packages-file" || option === "--state") {
      const value = argv[index + 1];
      if (!value || value.startsWith("--")) throw new Error(`${option} requires a path`);
      if (option === "--packages-file") options.packagesFile = resolve(value);
      else options.stateFile = resolve(value);
      index += 2;
      continue;
    }
    throw new Error(`unknown option: ${option}`);
  }
  if (options.packagesFile && !existsSync(options.packagesFile)) {
    throw new Error(`package inventory does not exist: ${options.packagesFile}`);
  }
  return options;
}

function defaultStateFile() {
  if (process.env.XDG_STATE_HOME) {
    return join(process.env.XDG_STATE_HOME, "ferromark", "npm-token-publishing", "state.tsv");
  }
  if (process.platform === "darwin") {
    return join(
      homedir(),
      "Library",
      "Application Support",
      "ferromark",
      "npm-token-publishing",
      "state.tsv",
    );
  }
  return join(homedir(), ".local", "state", "ferromark", "npm-token-publishing", "state.tsv");
}

function readState(path) {
  if (!existsSync(path)) return new Map();
  const state = new Map();
  const lines = readFileSync(path, "utf8").split(/\r?\n/u);
  if (lines.shift() !== "package\tfingerprint\tresult") {
    throw new Error(`unsupported state file format: ${path}`);
  }
  for (const line of lines) {
    if (!line) continue;
    const [name, fingerprint, result, ...extra] = line.split("\t");
    if (
      !name ||
      !/^[0-9a-f]{64}$/u.test(fingerprint ?? "") ||
      result !== "COMMAND_OK" ||
      extra.length
    ) {
      throw new Error(`malformed state row in ${path}`);
    }
    state.set(`${name}\t${fingerprint}`, result);
  }
  return state;
}

function checkNpmLogin() {
  const timeoutMs = Number(process.env.NPM_POLICY_WHOAMI_TIMEOUT_MS) || 15_000;
  return new Promise((resolvePromise) => {
    let output = "";
    let settled = false;
    const child = spawn("npm", [`--registry=${registry}`, "whoami"], {
      stdio: ["ignore", "pipe", "ignore"],
    });
    child.stdout.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      output += chunk;
    });
    const finish = (account) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      resolvePromise(account);
    };
    const timer = setTimeout(() => {
      child.kill("SIGTERM");
      const forceTimer = setTimeout(() => child.kill("SIGKILL"), 1_000);
      forceTimer.unref();
      finish("");
    }, timeoutMs);
    child.once("error", () => finish(""));
    child.once("close", (code) => finish(code === 0 ? output.trim() : ""));
  });
}

function runNpmLogin() {
  return new Promise((resolvePromise) => {
    const child = spawn("npm", [`--registry=${registry}`, "login"], { stdio: "inherit" });
    child.once("error", (error) => {
      process.stderr.write(`Could not start npm login: ${error.message}\n`);
      resolvePromise(false);
    });
    child.once("close", (code) => resolvePromise(code === 0));
  });
}

function runPreview(options, planFile) {
  const args = [helper, "--plan", planFile];
  for (const owner of options.owners) args.push("--owner", owner);
  if (options.packagesFile) args.push("--packages-file", options.packagesFile);
  return new Promise((resolvePromise) => {
    let stdout = "";
    const child = spawn("bash", args, {
      cwd: packageRoot,
      stdio: ["inherit", "pipe", "inherit"],
    });
    child.stdout.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.once("error", (error) => resolvePromise({ status: 127, stdout: `${error.message}\n` }));
    child.once("close", (status) => resolvePromise({ status: status ?? 1, stdout }));
  });
}

function publisherText(record) {
  const type = record.type ?? "unknown";
  const repository = record.repository ?? "unknown repository";
  const file = record.file ?? "unknown workflow";
  const permissions = Array.isArray(record.permissions) ? record.permissions.join(", ") : "unknown";
  return `type=${type}; repository=${repository}; workflow=${file}; permissions=${permissions}`;
}

function isEligible(packageInfo) {
  return packageInfo.access === "read-write" && packageInfo.permission_summary.length > 0;
}

function stateSummary(packageInfo, state) {
  const key = `${packageInfo.name}\t${packageInfo.trust_fingerprint}`;
  if (state.has(key)) return "COMMAND_OK (npm CLI succeeded; registry readback UNKNOWN)";
  const oldSuccess = [...state.keys()].some((item) => item.startsWith(`${packageInfo.name}\t`));
  return oldSuccess
    ? "not applied for this publisher config (older success exists)"
    : "not attempted";
}

function render(plan, selected, acknowledged, state, statePath) {
  process.stdout.write("\n=== npm token publishing policy ===\n");
  process.stdout.write(`npm account: ${plan.account}\n`);
  process.stdout.write(
    "Configured OIDC identity/permissions come from npm CLI. npm UI Valid status: UNKNOWN; current Publishing access: UNKNOWN.\n",
  );
  process.stdout.write(`Resumable command state: ${statePath}\n\n`);
  for (const [index, packageInfo] of plan.packages.entries()) {
    const eligible = isEligible(packageInfo);
    const marker = selected.has(packageInfo.name) ? "[x]" : "[ ]";
    const oidc = packageInfo.permission_summary || "none";
    const review = acknowledged.has(packageInfo.name) ? "UNKNOWN acknowledged" : "not reviewed";
    process.stdout.write(
      `${index + 1}. ${eligible ? marker : "[-]"} ${packageInfo.name} | OIDC: ${oidc} | Valid: UNKNOWN | review: ${review} | apply: ${stateSummary(packageInfo, state)}\n`,
    );
    if (!eligible) process.stdout.write(`   skipped: ${packageInfo.reason}\n`);
  }
  process.stdout.write(
    "\nUse `show N` or `review N` to inspect publisher claims. Commands: toggle N, review N, apply, refresh, reset N, status, help, quit\n",
  );
}

function packageAt(plan, selector) {
  const index = Number(selector) - 1;
  if (!Number.isInteger(index) || index < 0 || index >= plan.packages.length) return undefined;
  return plan.packages[index];
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const tempDirectory = mkdtempSync(join(tmpdir(), "ferromark-npm-policy-ui-"));
  let planFile = "";
  let planNumber = 0;
  let rl = createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: Boolean(process.stdin.isTTY),
  });
  let closed = false;
  let plan;
  const selected = new Set();
  const acknowledged = new Set();
  let state = new Map();

  function closeInput() {
    if (closed) return;
    closed = true;
    rl.close();
  }

  function question(prompt) {
    return new Promise((resolvePromise) => {
      if (closed) return resolvePromise(undefined);
      const onClose = () => resolvePromise(undefined);
      rl.once("close", onClose);
      rl.question(prompt, (answer) => {
        rl.off("close", onClose);
        resolvePromise(answer);
      });
    });
  }

  function reopenInput() {
    closed = false;
    rl = createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: Boolean(process.stdin.isTTY),
    });
  }

  async function ensureNpmLogin() {
    process.stdout.write("Checking npm CLI authentication...\n");
    let account = await checkNpmLogin();
    if (account) return account;
    process.stdout.write(
      "The npm CLI is not authenticated. A browser login does not sign in this terminal.\n",
    );
    const answer = await question("Run npm login interactively now? [y/N] ");
    if (answer?.toLowerCase() !== "y" && answer?.toLowerCase() !== "yes") return "";
    closeInput();
    const loginSucceeded = await runNpmLogin();
    reopenInput();
    if (!loginSucceeded) {
      process.stderr.write(
        "npm login did not complete successfully. No registry settings were changed.\n",
      );
      return "";
    }
    account = await checkNpmLogin();
    if (!account)
      process.stderr.write(
        "npm whoami still fails after login. No registry settings were changed.\n",
      );
    return account;
  }

  async function refresh() {
    const candidatePlanFile = join(tempDirectory, `plan-${++planNumber}.json`);
    process.stdout.write("Discovering npm packages and configured Trusted Publishers...\n");
    closeInput();
    let result;
    try {
      result = await runPreview(options, candidatePlanFile);
    } finally {
      reopenInput();
    }
    if (result.status !== 0) {
      if (result.stdout) process.stdout.write(result.stdout);
      rmSync(candidatePlanFile, { force: true });
      throw new Error("npm package discovery failed; no settings were changed");
    }
    const refreshedPlan = JSON.parse(readFileSync(candidatePlanFile, "utf8"));
    if (!Array.isArray(refreshedPlan.packages) || typeof refreshedPlan.account !== "string") {
      throw new Error("npm package discovery returned an invalid plan");
    }
    const refreshedState = readState(options.stateFile);
    planFile = candidatePlanFile;
    plan = refreshedPlan;
    selected.clear();
    acknowledged.clear();
    state = refreshedState;
    render(plan, selected, acknowledged, state, options.stateFile);
  }

  async function apply() {
    const packages = plan.packages.filter((item) => selected.has(item.name));
    if (packages.length === 0) {
      process.stdout.write("Select at least one eligible package before applying.\n");
      return;
    }
    const notAcknowledged = packages.filter((item) => !acknowledged.has(item.name));
    if (notAcknowledged.length) {
      process.stdout.write(
        `Review and acknowledge unknown status for: ${notAcknowledged.map((item) => item.name).join(", ")}\n`,
      );
      return;
    }
    mkdirSync(dirname(options.stateFile), { recursive: true, mode: 0o700 });
    const acknowledgedFile = join(tempDirectory, "acknowledged-packages.txt");
    writeFileSync(acknowledgedFile, packages.map((item) => item.name).join("\n") + "\n", {
      mode: 0o600,
    });
    process.stdout.write(
      "Applying the selected packages. The final confirmation will state that npm UI validity and saved policy remain unknown.\n",
    );
    closeInput();
    await new Promise((resolvePromise) => {
      const child = spawn(
        "bash",
        [
          helper,
          "--apply",
          "--plan",
          planFile,
          "--acknowledged-packages",
          acknowledgedFile,
          "--state",
          options.stateFile,
        ],
        { cwd: packageRoot, stdio: "inherit" },
      );
      child.once("error", (error) => {
        process.stderr.write(`Could not start the npm update helper: ${error.message}\n`);
        resolvePromise(127);
      });
      child.once("close", (code) => resolvePromise(code ?? 1));
    }).then((code) => {
      process.stdout.write(
        `\nBatch command exited with status ${code}. Check each package's npm settings to verify the saved option.\n`,
      );
    });
    reopenInput();
    state = readState(options.stateFile);
    render(plan, selected, acknowledged, state, options.stateFile);
  }

  async function resetState(packageInfo) {
    if (!existsSync(options.stateFile)) {
      process.stdout.write("There is no saved command result to reset.\n");
      return;
    }
    const key = `${packageInfo.name}\t${packageInfo.trust_fingerprint}`;
    if (!state.has(key)) {
      process.stdout.write(
        "There is no successful command result for this publisher configuration.\n",
      );
      return;
    }
    const answer = await question(
      `Reset removes this package's COMMAND_OK marker. Confirm in npm settings that mfa=publish did not save, then type RESET ${packageInfo.name}: `,
    );
    if (answer === `RESET ${packageInfo.name}`) saveReset(packageInfo);
    else process.stdout.write("Reset confirmation did not match; state was unchanged.\n");
  }

  function saveReset(packageInfo) {
    const lines = readFileSync(options.stateFile, "utf8").split(/\r?\n/u);
    const key = `${packageInfo.name}\t${packageInfo.trust_fingerprint}\tCOMMAND_OK`;
    const filtered = lines.filter((line, index) => index === 0 || line !== key);
    const temporary = `${options.stateFile}.${process.pid}.tmp`;
    const descriptor = openSync(temporary, "w", 0o600);
    try {
      fchmodSync(descriptor, 0o600);
      writeFileSync(descriptor, filtered.join("\n").replace(/\n*$/u, "\n"));
      fsyncSync(descriptor);
    } finally {
      closeSync(descriptor);
    }
    renameSync(temporary, options.stateFile);
    state = readState(options.stateFile);
    process.stdout.write(
      `Removed the resume marker for ${packageInfo.name}. The next apply may run its npm command again.\n`,
    );
  }

  try {
    if (!process.stdin.isTTY && process.env.NPM_POLICY_TUI_TEST !== "1") {
      process.stderr.write("This review flow needs an interactive terminal.\n");
      process.exitCode = 2;
      return;
    }
    const account = await ensureNpmLogin();
    if (!account) {
      process.exitCode = 1;
      return;
    }
    await refresh();

    while (true) {
      const line = await question("npm-policy> ");
      if (line === undefined) break;
      const [command = "", selector = ""] = line.trim().split(/\s+/u);
      if (command === "quit" || command === "exit") {
        break;
      } else if (command === "help") {
        process.stdout.write(
          "Use `review N` to inspect claims, acknowledge unknown npm UI status, and select a package. `toggle N` checks or unchecks an eligible package; `apply` runs the selected npm commands. `show N` prints the configured publisher claims. `refresh` rebuilds discovery; `reset N` clears a CLI-success resume marker only after confirming in npm settings that the policy did not save.\n",
        );
      } else if (command === "status") {
        state = readState(options.stateFile);
        render(plan, selected, acknowledged, state, options.stateFile);
      } else if (command === "show") {
        const packageInfo = packageAt(plan, selector);
        if (!packageInfo) process.stdout.write("Use a valid package number.\n");
        else
          process.stdout.write(
            `${JSON.stringify(packageInfo, null, 2)}\nUI Valid status: UNKNOWN; current Publishing access setting: UNKNOWN.\n`,
          );
      } else if (command === "toggle") {
        const packageInfo = packageAt(plan, selector);
        if (!packageInfo) process.stdout.write("Use a valid package number.\n");
        else if (!isEligible(packageInfo))
          process.stdout.write(
            `${packageInfo.name} is not eligible because it lacks current npm write access or a configured publish-capable OIDC permission.\n`,
          );
        else if (selected.has(packageInfo.name)) selected.delete(packageInfo.name);
        else selected.add(packageInfo.name);
        render(plan, selected, acknowledged, state, options.stateFile);
      } else if (command === "review") {
        const packageInfo = packageAt(plan, selector);
        if (!packageInfo) process.stdout.write("Use a valid package number.\n");
        else if (!isEligible(packageInfo))
          process.stdout.write(`${packageInfo.name} is not eligible for an update.\n`);
        else {
          process.stdout.write(`Configured publisher claims for ${packageInfo.name}:\n`);
          for (const [publisherIndex, record] of packageInfo.trust_records.entries()) {
            process.stdout.write(`  ${publisherIndex + 1}. ${publisherText(record)}\n`);
          }
          process.stdout.write(
            "  npm UI Valid status: UNKNOWN; configuration expiry: UNKNOWN; current Publishing access: UNKNOWN.\n",
          );
          const answer = await question(
            "Accept these unknowns and permit an update attempt? Type ACK UNKNOWN: ",
          );
          if (answer === "ACK UNKNOWN") {
            acknowledged.add(packageInfo.name);
            selected.add(packageInfo.name);
            process.stdout.write(
              `${packageInfo.name}: unknown npm UI status acknowledged and selected; this does not mark it Valid.\n`,
            );
          } else {
            process.stdout.write(`${packageInfo.name}: remains unacknowledged.\n`);
          }
          render(plan, selected, acknowledged, state, options.stateFile);
        }
      } else if (command === "apply") {
        await apply();
      } else if (command === "refresh") {
        try {
          await refresh();
        } catch (error) {
          process.stderr.write(`${error.message}\n`);
        }
      } else if (command === "reset") {
        const packageInfo = packageAt(plan, selector);
        if (!packageInfo) process.stdout.write("Use a valid package number.\n");
        else await resetState(packageInfo);
      } else if (line.trim()) {
        process.stdout.write("Unknown command. Type `help` for available commands.\n");
      }
    }
  } finally {
    closeInput();
    rmSync(tempDirectory, { recursive: true, force: true });
  }
}

main().catch((error) => {
  process.stderr.write(`${error.message}\n`);
  process.exitCode = 1;
});
