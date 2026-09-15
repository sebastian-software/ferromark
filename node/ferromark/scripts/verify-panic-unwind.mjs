import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { mkdtemp, readFile, readdir, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = new URL('../../../', import.meta.url)
const buildScriptUrl = new URL('./build-native.mjs', import.meta.url)
const [cargoToml, buildScript] = await Promise.all([
  readFile(new URL('Cargo.toml', root), 'utf8'),
  readFile(buildScriptUrl, 'utf8'),
])

const profile = cargoToml.match(
  /^\[profile\.release-node\]$([\s\S]*?)(?=^\[|(?![\s\S]))/m,
)

assert.ok(profile, 'Cargo.toml must define the release-node profile at the workspace root')
assert.match(profile[1], /^inherits = "release"$/m)
assert.match(profile[1], /^panic = "unwind"$/m)
assert.match(
  buildScript,
  /['"]--profile['"],\s*['"]release-node['"],/,
  'the native package builder must select Cargo\'s release-node profile',
)

const cargo = spawnSync(
  'cargo',
  [
    'rustc',
    '--locked',
    '--package',
    'ferromark-node',
    '--profile',
    'release-node',
    '--lib',
    '--',
    '--print',
    'cfg',
  ],
  { cwd: root, encoding: 'utf8' },
)

assert.ifError(cargo.error)
assert.equal(
  cargo.status,
  0,
  `Could not inspect the Node cdylib panic strategy:\n${cargo.stderr}`,
)
assert.match(
  cargo.stdout,
  /^panic="unwind"$/m,
  'the release-node cdylib must compile with panic=unwind',
)

const outputDir = await mkdtemp(join(tmpdir(), 'ferromark-panic-unwind-'))

try {
  const build = spawnSync(process.execPath, [fileURLToPath(buildScriptUrl)], {
    cwd: fileURLToPath(new URL('./', buildScriptUrl)),
    encoding: 'utf8',
    env: {
      ...process.env,
      FERROMARK_NAPI_FEATURES: 'panic-test',
      FERROMARK_NATIVE_OUTPUT_DIR: outputDir,
    },
  })

  assert.ifError(build.error)
  assert.equal(
    build.status,
    0,
    `Could not build the panic-unwind verification addon:\n${build.stderr}`,
  )

  const nativeBinding = (await readdir(outputDir)).find(file => file.endsWith('.node'))
  assert.ok(nativeBinding, 'the panic-unwind verification build did not produce a .node addon')

  const panicCheck = spawnSync(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      `import { createRequire } from 'node:module';\nconst addon = createRequire(import.meta.url)(${JSON.stringify(join(outputDir, nativeBinding))});\ntry { addon.testPanicUnwind(); } catch (error) { if (error instanceof Error && error.message.includes('ferromark N-API panic-unwind verification')) process.exit(0); }\nprocess.exit(1);`,
    ],
    { encoding: 'utf8' },
  )

  assert.ifError(panicCheck.error)
  assert.equal(
    panicCheck.status,
    0,
    `a panic in the Node cdylib was not translated to a JavaScript Error:\n${panicCheck.stderr}`,
  )
} finally {
  await rm(outputDir, { force: true, recursive: true })
}
