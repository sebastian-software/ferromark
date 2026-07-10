import { spawnSync } from 'node:child_process'
import process from 'node:process'

const pnpm = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm'
const args = [
  'exec',
  'napi',
  'build',
  '--platform',
  '--release',
  '--manifest-path',
  '../native/Cargo.toml',
  '--dts',
  'native.d.ts',
  '--no-js',
  '--output-dir',
  '.',
]

if (process.env.FERROMARK_RUST_TARGET) {
  args.push('--target', process.env.FERROMARK_RUST_TARGET)
}

args.push('--', '--locked')

const result = spawnSync(pnpm, args, {
  cwd: new URL('..', import.meta.url),
  stdio: 'inherit',
})

if (result.error) {
  throw result.error
}

process.exit(result.status ?? 1)
