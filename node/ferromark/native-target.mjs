/**
 * Resolve a Node platform, architecture, and Linux libc to the native package suffix.
 *
 * `glibcVersionRuntime` is intentionally supplied by the caller so the target table
 * can be tested without mutating process globals or collecting diagnostic reports.
 *
 * @param {NodeJS.Platform} platform
 * @param {string} arch
 * @param {string | undefined} [glibcVersionRuntime]
 */
export function nativeTarget(platform, arch, glibcVersionRuntime) {
  const libc = platform === 'linux'
    ? glibcVersionRuntime ? '-gnu' : '-musl'
    : ''
  const key = `${platform}-${arch}${libc}`
  /** @type {Record<string, string>} */
  const targets = {
    'darwin-arm64': 'darwin-arm64',
    'darwin-x64': 'darwin-x64',
    'linux-arm64-gnu': 'linux-arm64-gnu',
    'linux-arm64-musl': 'linux-arm64-musl',
    'linux-x64-gnu': 'linux-x64-gnu',
    'linux-x64-musl': 'linux-x64-musl',
    'win32-arm64': 'win32-arm64-msvc',
    'win32-x64': 'win32-x64-msvc',
  }

  const target = targets[key]
  if (!target) {
    throw new Error(`ferromark does not support ${platform}/${arch}`)
  }
  return target
}
