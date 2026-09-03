import assert from 'node:assert/strict'
import { readFile, stat } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const workspace = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const packageDir = path.join(workspace, 'ferromark')
const [target, ...extraArgs] = process.argv.slice(2)

if (!target || extraArgs.length > 0 || !/^[a-z0-9-]+$/.test(target)) {
  throw new Error(`Usage: node ${path.basename(fileURLToPath(import.meta.url))} <platform-target>`)
}

const [mainPackage, platformPackage] = await Promise.all([
  readFile(path.join(packageDir, 'package.json'), 'utf8').then(JSON.parse),
  readFile(path.join(packageDir, 'npm', target, 'package.json'), 'utf8').then(JSON.parse),
])
const dependency = `${mainPackage.name}-${target}`
const binary = path.join(packageDir, 'npm', target, `ferromark.${target}.node`)
const binaryInfo = await stat(binary)

assert.equal(platformPackage.name, dependency)
assert.equal(platformPackage.version, mainPackage.version)
assert.equal(mainPackage.optionalDependencies[dependency], mainPackage.version)
assert.ok(binaryInfo.isFile() && binaryInfo.size > 0, `Invalid native binary: ${binary}`)

console.log(`Verified ${dependency}@${platformPackage.version} (${binaryInfo.size} bytes)`)
