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
  'linux-arm64-musl',
  'linux-x64-gnu',
  'linux-x64-musl',
  'win32-arm64-msvc',
  'win32-x64-msvc',
]
for (const target of targets) {
  const platformDir = path.join(packageDir, 'npm', target)
  const platformPackage = JSON.parse(
    await readFile(path.join(platformDir, 'package.json'), 'utf8'),
  )
  if (platformPackage.version !== packageJson.version) {
    throw new Error(
      `Platform package version mismatch: ${platformPackage.name}@${platformPackage.version}`,
    )
  }
  if (packageJson.optionalDependencies[platformPackage.name] !== packageJson.version) {
    throw new Error(`Main package does not pin ${platformPackage.name}@${packageJson.version}`)
  }
  const file = path.join(platformDir, `ferromark.${target}.node`)
  const info = await stat(file)
  if (!info.isFile() || info.size === 0) {
    throw new Error(`Invalid native binary: ${file}`)
  }
}

console.log(`Verified ferromark ${packageJson.version} with ${targets.length} platform packages`)
