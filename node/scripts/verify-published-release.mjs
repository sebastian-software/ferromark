import assert from 'node:assert/strict'
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'
import { releaseChannel } from './release-channel.mjs'

const version = process.argv[2]
const { tag, prerelease } = releaseChannel(version)
const pkg = JSON.parse(readFileSync(new URL('../ferromark/package.json', import.meta.url)))
assert.equal(pkg.version, version)
const before = JSON.parse(readFileSync(process.argv[3], 'utf8'))
for (const name of [...Object.keys(pkg.optionalDependencies), pkg.name]) {
  let metadata
  for (let attempt = 0; attempt < 8; attempt++) {
    const response = await fetch(`https://registry.npmjs.org/${encodeURIComponent(name)}`, { signal: AbortSignal.timeout(30_000), cache: 'no-store' })
    if (response.ok) {
      metadata = await response.json()
      if (metadata.versions?.[version] && metadata['dist-tags']?.[tag] === version) break
    }
    if (attempt < 7) await new Promise(resolve => setTimeout(resolve, 15_000))
  }
  assert.equal(metadata?.versions?.[version]?.name, name, `${name}: version is missing`)
  assert.equal(metadata?.['dist-tags']?.[tag], version, `${name}: incorrect release channel`)
  if (prerelease) assert.equal(metadata['dist-tags']?.latest, before[name], `${name}: RC changed latest`)
}
const consumer = mkdtempSync(join(tmpdir(), 'ferromark-registry-consumer-'))
try {
  writeFileSync(join(consumer, 'package.json'), JSON.stringify({ private: true, type: 'module' }))
  run('npm', ['install', '--ignore-scripts', '--no-audit', '--no-fund', `ferromark@${version}`])
  writeFileSync(join(consumer, 'smoke.mjs'), `import { toHtml } from 'ferromark'; if (toHtml('Hello, **world**!') !== '<p>Hello, <strong>world</strong>!</p>\\n') throw new Error('Unexpected output');`)
  run(process.execPath, ['smoke.mjs'])
  writeFileSync(join(consumer, 'Cargo.toml'), `[workspace]\n[package]\nname = "ferromark-registry-consumer"\nversion = "0.0.0"\nedition = "2024"\n[dependencies]\nferromark = "=${version}"\n[[bin]]\nname = "smoke"\npath = "smoke.rs"\n`)
  writeFileSync(join(consumer, 'smoke.rs'), 'fn main() { assert_eq!(ferromark::to_html("Hello, **world**!").unwrap(), "<p>Hello, <strong>world</strong>!</p>\\n"); }')
  run('cargo', ['run'])
  console.log(`Fresh npm and crates.io installations passed for ${version}`)
} finally {
  rmSync(consumer, { recursive: true, force: true })
}
function run(command, args) {
  const result = spawnSync(command, args, { cwd: consumer, stdio: 'inherit' })
  if (result.error) throw result.error
  assert.equal(result.status, 0, `${command} failed`)
}
