import { spawnSync } from 'node:child_process'
import { rm } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const pnpm = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm'
const outputDir = process.env.FERROMARK_NATIVE_OUTPUT_DIR ?? '.'
const packageDir = fileURLToPath(new URL('..', import.meta.url))
const buildOutputDir = outputDir === '.' ? '.napi-artifacts' : outputDir

if (outputDir === '.') {
  await rm(path.join(packageDir, buildOutputDir), { force: true, recursive: true })
}

const args = [
  'exec',
  'napi',
  'build',
  '--platform',
  '--profile',
  'release-node',
  '--manifest-path',
  '../native/Cargo.toml',
  '--dts',
  'native.d.ts',
  '--no-js',
  '--output-dir',
  buildOutputDir,
]

if (process.env.FERROMARK_RUST_TARGET) {
  args.push('--target', process.env.FERROMARK_RUST_TARGET)
  if (process.env.FERROMARK_RUST_TARGET.includes('-unknown-linux-gnu')) {
    args.push('--use-napi-cross')
  }
  else if (process.env.FERROMARK_RUST_TARGET.endsWith('-musl')) {
    args.push('--cross-compile')
  }
}

if (process.env.FERROMARK_NAPI_FEATURES) {
  args.push('--features', process.env.FERROMARK_NAPI_FEATURES)
}

args.push('--', '--locked')

const result = spawnSync(pnpm, args, {
  cwd: new URL('..', import.meta.url),
  // Windows command shims such as pnpm.cmd require cmd.exe for spawning.
  shell: process.platform === 'win32',
  stdio: 'inherit',
})

if (result.error) {
  throw result.error
}
if (result.status !== 0) {
  process.exit(result.status ?? 1)
}

// Normal builds keep the local addon for development and copy it into the
// matching optional package. Isolated verification builds use a temporary
// output directory and must not modify package artifacts.
if (outputDir === '.') {
  const artifacts = spawnSync(
    pnpm,
    ['exec', 'napi', 'artifacts', '--output-dir', buildOutputDir, '--npm-dir', 'npm'],
    {
      cwd: new URL('..', import.meta.url),
      shell: process.platform === 'win32',
      stdio: 'inherit',
    },
  )
  if (artifacts.error) {
    throw artifacts.error
  }
  await rm(path.join(packageDir, buildOutputDir), { force: true, recursive: true })
  process.exit(artifacts.status ?? 1)
}
