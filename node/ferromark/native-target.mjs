import { readFileSync } from "node:fs";
// Generated consumer copies are owned by ferramenta-native. Do not edit them.
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

export const NATIVE_TARGETS = Object.freeze(
  [
    {
      id: "linux-x64-gnu",
      triple: "x86_64-unknown-linux-gnu",
      os: "linux",
      cpu: "x64",
      libc: "glibc",
    },
    {
      id: "linux-arm64-gnu",
      triple: "aarch64-unknown-linux-gnu",
      os: "linux",
      cpu: "arm64",
      libc: "glibc",
    },
    {
      id: "linux-x64-musl",
      triple: "x86_64-unknown-linux-musl",
      os: "linux",
      cpu: "x64",
      libc: "musl",
    },
    {
      id: "linux-arm64-musl",
      triple: "aarch64-unknown-linux-musl",
      os: "linux",
      cpu: "arm64",
      libc: "musl",
    },
    { id: "darwin-arm64", triple: "aarch64-apple-darwin", os: "darwin", cpu: "arm64" },
    { id: "win32-x64-msvc", triple: "x86_64-pc-windows-msvc", os: "win32", cpu: "x64" },
    { id: "win32-arm64-msvc", triple: "aarch64-pc-windows-msvc", os: "win32", cpu: "arm64" },
  ].map((target) => Object.freeze(target)),
);

export function linuxLibc(report, readLoaderHelper) {
  if (report?.header?.glibcVersionRuntime) return "gnu";
  if (report?.sharedObjects?.some((item) => item.includes("ld-musl-"))) return "musl";
  if (report) return "musl";
  return readLoaderHelper().includes("musl") ? "musl" : "gnu";
}

export function diagnosticReport() {
  const report = process.report;
  if (!report?.getReport) return;
  const previous = report.excludeNetwork;
  try {
    report.excludeNetwork = true;
    return report.getReport();
  } catch {
    // A failed report leaves the loader helper as the remaining evidence.
  } finally {
    report.excludeNetwork = previous;
  }
}

export function detectLinuxLibc(platform = process.platform) {
  if (platform !== "linux") return;
  return linuxLibc(diagnosticReport(), () => {
    try {
      return readFileSync("/usr/bin/ldd", "latin1");
    } catch {
      return "";
    }
  });
}

export function nativeTarget(platform, arch, libc = "gnu") {
  const suffix = platform === "linux" ? `-${libc}` : platform === "win32" ? "-msvc" : "";
  const id = `${platform}-${arch}${suffix}`;
  if (!NATIVE_TARGETS.some((target) => target.id === id)) {
    throw new Error(
      `Unsupported native target ${platform}/${arch}${platform === "linux" ? `/${libc}` : ""}`,
    );
  }
  return id;
}

function loadHint(target) {
  if (target.endsWith("-gnu"))
    return "Check that glibc 2.17 or newer is available and that no required shared library is missing.";
  if (target.endsWith("-musl"))
    return "Check that the musl runtime is compatible and that no required shared library is missing.";
  if (target.startsWith("win32-"))
    return "Install or repair the Microsoft Visual C++ Redistributable and verify the binary architecture.";
  return "Check the macOS version and binary architecture, and whether quarantine or code-signing policy blocked the addon.";
}

function loadCandidate(require, candidate, context) {
  let resolved;
  try {
    resolved = require.resolve(candidate.file);
  } catch (error) {
    if (error.code !== "MODULE_NOT_FOUND") throw error;
    return { error };
  }
  try {
    return { binding: require(resolved) };
  } catch (error) {
    if (error.code !== "ERR_DLOPEN_FAILED") throw error;
    throw new Error(
      `${context.name} could not load native binary ${context.filename} from ${candidate.source} for ${process.platform}/${process.arch} (ERR_DLOPEN_FAILED). ${loadHint(context.target)}`,
      { cause: error },
    );
  }
}

function loadCandidates(require, candidates, context) {
  const errors = [];
  for (const candidate of candidates) {
    const result = loadCandidate(require, candidate, context);
    if (result.binding) return result.binding;
    errors.push(result.error);
  }
  throw new Error(
    `${context.name} could not load the optional native package ${context.packageName} for ${process.platform}/${process.arch}. No native binary for ${context.target}. Install the optional platform package ${context.packageName}.`,
    {
      cause: new AggregateError(errors, `No native binary found for ${context.target}`),
    },
  );
}

export function createNativeLoader(parentUrl, config) {
  const require = createRequire(parentUrl);
  let binding;
  return () => {
    if (binding) return binding;
    const target = nativeTarget(process.platform, process.arch, detectLinuxLibc());
    const filename = config.binaryPattern.replace("{target}", target);
    const packageName = config.packagePattern.replace("{target}", target);
    const local = config.localPatterns ?? [config.binaryPattern];
    const candidates = [
      ...local.map((pattern) => ({
        file: fileURLToPath(new URL(pattern.replace("{target}", target), parentUrl)),
        source: "the local package",
      })),
      { file: packageName, source: `the optional package ${packageName}` },
    ];
    binding = loadCandidates(require, candidates, {
      name: config.name,
      target,
      filename,
      packageName,
    });
    return binding;
  };
}
