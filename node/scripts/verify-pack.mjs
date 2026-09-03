import { mkdir, readFile, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import process from 'node:process'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const workspace = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const packageDir = path.join(workspace, 'ferromark')
const artifacts = path.join(workspace, 'artifacts')
const args = process.argv.slice(2)

if (args.length > 1 || (args.length === 1 && args[0] !== '--all-targets')) {
  throw new Error(`Usage: node ${path.basename(fileURLToPath(import.meta.url))} [--all-targets]`)
}

const packageJson = JSON.parse(await readFile(path.join(packageDir, 'package.json'), 'utf8'))
const platformTargets = Object.keys(packageJson.optionalDependencies)
  .map(name => name.slice(`${packageJson.name}-`.length))
const targets = args[0] === '--all-targets' ? platformTargets : [nativeTarget()]

await rm(artifacts, { force: true, recursive: true })
await mkdir(artifacts, { recursive: true })

const main = pack(packageDir)
const mainFiles = main.files.map(file => file.path).sort()
const allowedMain = [
  'LICENSE',
  'README.md',
  'index.d.mts',
  'index.mjs',
  'native-target.mjs',
  'native.d.ts',
  'package.json',
]
if (mainFiles.some(file => file.endsWith('.node'))) {
  throw new Error('Main package must not contain a native binary')
}
if (mainFiles.some(file => !allowedMain.includes(file))) {
  throw new Error(`Main package contains unexpected files:\n${mainFiles.join('\n')}`)
}
if (main.unpackedSize >= 100_000) {
  throw new Error(`Main package is unexpectedly large: ${main.unpackedSize} bytes unpacked`)
}

const platforms = targets.map((target) => {
  const result = pack(path.join(packageDir, 'npm', target))
  const files = result.files.map(file => file.path).sort()
  const expected = [
    'README.md',
    `ferromark.${target}.node`,
    'package.json',
  ]
  if (files.length !== expected.length || expected.some(file => !files.includes(file))) {
    throw new Error(`${target} package has invalid contents:\n${files.join('\n')}`)
  }
  return { filename: result.filename, files, unpackedSize: result.unpackedSize }
})

console.log(JSON.stringify({
  main: { filename: main.filename, files: mainFiles, unpackedSize: main.unpackedSize },
  platforms,
}, null, 2))

function pack(directory) {
  const packed = spawnSync(
    'npm',
    ['pack', '--json', '--pack-destination', artifacts],
    {
      cwd: directory,
      encoding: 'utf8',
      env: { ...process.env, npm_config_cache: path.join(tmpdir(), 'ferromark-npm-cache') },
    },
  )
  if (packed.status !== 0) {
    process.stderr.write(packed.stderr)
    process.exit(packed.status ?? 1)
  }
  return JSON.parse(packed.stdout)[0]
}

function nativeTarget() {
  const base = `${process.platform}-${process.arch}`
  if (process.platform !== 'linux') {
    return base === 'win32-arm64' || base === 'win32-x64' ? `${base}-msvc` : base
  }
  const report = process.report?.getReport?.()
  return `${base}-${report?.header?.glibcVersionRuntime ? 'gnu' : 'musl'}`
}
