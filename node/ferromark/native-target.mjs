/**
 * The parts of a Node.js diagnostic report that identify the Linux C library.
 *
 * @typedef {{
 *   header?: { glibcVersionRuntime?: string }
 *   sharedObjects?: readonly string[]
 * }} DiagnosticReport
 */

/**
 * Resolve the Linux C library from a diagnostic report, with a loader fallback.
 *
 * Node.js built against glibc always records `glibcVersionRuntime`, so a
 * collected report without one describes a musl host. When no report can be
 * collected at all — non-Node runtimes, or a future removal of `process.report`
 * — only positive evidence selects musl, because misreporting a glibc host
 * sends the loader to a musl package that is not installed.
 *
 * Both inputs are supplied by the caller so the detection can be tested without
 * mutating process globals, collecting diagnostic reports, or reading files.
 *
 * @param {DiagnosticReport | undefined} report Collected diagnostic report, if any.
 * @param {() => string} readLoaderHelper Reads `/usr/bin/ldd`, `""` when unreadable.
 * @returns {"gnu" | "musl"} Detected C library.
 */
export function linuxLibc(report, readLoaderHelper) {
  if (report?.header?.glibcVersionRuntime) {
    return "gnu";
  }
  if (report?.sharedObjects?.some((sharedObject) => sharedObject.includes("ld-musl-"))) {
    return "musl";
  }
  if (report) {
    // A collected report without a glibc runtime version: a musl host.
    return "musl";
  }
  // No report at all: only the loader helper can still prove musl.
  return readLoaderHelper().includes("musl") ? "musl" : "gnu";
}

/**
 * Resolve a Node platform, architecture, and Linux libc to the native package suffix.
 *
 * `libc` is intentionally supplied by the caller so the target table can be
 * tested without mutating process globals or collecting diagnostic reports.
 * An unknown Linux libc selects gnu, the more widely installed runtime.
 *
 * @param {NodeJS.Platform} platform Current Node platform.
 * @param {string} arch Current CPU architecture.
 * @param {"gnu" | "musl"} [libc] Detected Linux C library.
 */
export function nativeTarget(platform, arch, libc) {
  const suffix = platform === "linux" ? `-${libc ?? "gnu"}` : "";
  const key = `${platform}-${arch}${suffix}`;
  /** @type {Record<string, string>} */
  const targets = {
    "darwin-arm64": "darwin-arm64",
    "darwin-x64": "darwin-x64",
    "linux-arm64-gnu": "linux-arm64-gnu",
    "linux-arm64-musl": "linux-arm64-musl",
    "linux-x64-gnu": "linux-x64-gnu",
    "linux-x64-musl": "linux-x64-musl",
    "win32-arm64": "win32-arm64-msvc",
    "win32-x64": "win32-x64-msvc",
  };

  const target = targets[key];
  if (!target) {
    throw new Error(`ferromark does not support ${platform}/${arch}`);
  }
  return target;
}
