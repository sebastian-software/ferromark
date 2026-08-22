import { spawnSync } from 'node:child_process'
import process from 'node:process'

const pnpm = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm'
const outputDir = process.env.FERROMARK_NATIVE_OUTPUT_DIR ?? '.'
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
  outputDir,
]

if (process.env.FERROMARK_RUST_TARGET) {
  args.push('--target', process.env.FERROMARK_RUST_TARGET)
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

process.exit(result.status ?? 1)
