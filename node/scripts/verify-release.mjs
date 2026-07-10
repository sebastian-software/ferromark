import { readFile, stat } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const workspace = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const root = path.resolve(workspace, '..')
const packageDir = path.join(workspace, 'ferromark')
const packageJson = JSON.parse(await readFile(path.join(packageDir, 'package.json'), 'utf8'))
const cargo = await readFile(path.join(root, 'Cargo.toml'), 'utf8')
const cargoVersion = cargo.match(/^version = "([^"]+)"/m)?.[1]

if (!cargoVersion || cargoVersion !== packageJson.version) {
  throw new Error(
    `Rust/npm version mismatch: Cargo=${cargoVersion ?? 'missing'}, npm=${packageJson.version}`,
  )
}

const targets = [
  'darwin-arm64',
  'darwin-x64',
  'linux-arm64-gnu',
  'linux-x64-gnu',
  'win32-arm64-msvc',
  'win32-x64-msvc',
]
for (const target of targets) {
  const file = path.join(packageDir, `ferromark.${target}.node`)
  const info = await stat(file)
  if (!info.isFile() || info.size === 0) {
    throw new Error(`Invalid native binary: ${file}`)
  }
}

console.log(`Verified ferromark ${packageJson.version} with ${targets.length} native binaries`)
