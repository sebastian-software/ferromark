import { readFileSync } from 'node:fs'
import { mkdir, readFile, rm } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { gunzipSync } from 'node:zlib'

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
  'LICENSE-APACHE',
  'LICENSE-MIT',
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
    'LICENSE',
    'LICENSE-MIT',
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

// pnpm rather than npm: the facade references its native sidecars with the
// workspace protocol, and only pnpm resolves `workspace:*` to the sidecar's
// version while packing. An npm-packed facade would ship `workspace:*`, which
// no registry consumer can install. The published archives are these tarballs.
function pack(directory) {
  const packed = spawnSync(
    process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm',
    ['pack', '--json', '--pack-destination', artifacts],
    { cwd: directory, encoding: 'utf8', shell: process.platform === 'win32' },
  )
  if (packed.status !== 0) {
    process.stderr.write(packed.stderr)
    process.exit(packed.status ?? 1)
  }
  const result = JSON.parse(packed.stdout)
  const filename = path.basename(result.filename)
  return {
    filename,
    files: result.files,
    // `pnpm pack --json` reports no unpacked size, so it is read back from the
    // archive the check is about to approve.
    unpackedSize: unpackedSize(path.join(artifacts, filename)),
  }
}

/** Sum of the member sizes in a gzipped tar, read from its ustar headers. */
function unpackedSize(archive) {
  const tar = gunzipSync(readFileSync(archive))
  let total = 0
  for (let offset = 0; offset + 512 <= tar.length; ) {
    // Two consecutive zero blocks end the archive.
    if (tar[offset] === 0) break
    const size = Number.parseInt(tar.toString('utf8', offset + 124, offset + 136).replace(/\0.*$/, '').trim(), 8)
    if (!Number.isFinite(size)) {
      throw new Error(`Unreadable tar header at offset ${offset} in ${archive}`)
    }
    if (tar.toString('utf8', offset + 156, offset + 157) === '0') total += size
    offset += 512 + Math.ceil(size / 512) * 512
  }
  return total
}

function nativeTarget() {
  const base = `${process.platform}-${process.arch}`
  if (process.platform !== 'linux') {
    return base === 'win32-arm64' || base === 'win32-x64' ? `${base}-msvc` : base
  }
  const report = process.report?.getReport?.()
  return `${base}-${report?.header?.glibcVersionRuntime ? 'gnu' : 'musl'}`
}
