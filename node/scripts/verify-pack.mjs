import { mkdir, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import process from 'node:process'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const workspace = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const packageDir = path.join(workspace, 'ferromark')
const artifacts = path.join(workspace, 'artifacts')

await rm(artifacts, { force: true, recursive: true })
await mkdir(artifacts, { recursive: true })

const packed = spawnSync(
  'npm',
  ['pack', '--json', '--pack-destination', artifacts],
  {
    cwd: packageDir,
    encoding: 'utf8',
    env: { ...process.env, npm_config_cache: path.join(tmpdir(), 'ferromark-npm-cache') },
  },
)
if (packed.status !== 0) {
  process.stderr.write(packed.stderr)
  process.exit(packed.status ?? 1)
}

const [result] = JSON.parse(packed.stdout)
const allowed = [
  /^LICENSE$/,
  /^README\.md$/,
  /^ferromark\.[a-z0-9-]+\.node$/,
  /^index\.d\.mts$/,
  /^index\.mjs$/,
  /^native\.d\.ts$/,
  /^package\.json$/,
]
const files = result.files.map(file => file.path).sort()
const unexpected = files.filter(file => !allowed.some(pattern => pattern.test(file)))
if (unexpected.length > 0) {
  throw new Error(`Packed package contains unexpected files:\n${unexpected.join('\n')}`)
}
if (!files.some(file => /^ferromark\.[a-z0-9-]+\.node$/.test(file))) {
  throw new Error('Packed package does not contain a native binary')
}

console.log(JSON.stringify({ filename: result.filename, files }, null, 2))
