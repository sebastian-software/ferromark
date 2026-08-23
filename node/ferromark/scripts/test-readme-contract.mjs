import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'

const readmeUrl = new URL('../README.md', import.meta.url)
const packageUrl = new URL('../package.json', import.meta.url)
const loaderUrl = new URL('../index.mjs', import.meta.url)
const declarationUrl = new URL('../index.d.mts', import.meta.url)
const nativeOptionsUrl = new URL('../../native/src/lib.rs', import.meta.url)
const coreOptionsUrl = new URL('../../../src/lib.rs', import.meta.url)

const [readme, packageJson, loader, declarations, nativeOptions, coreOptions] = await Promise.all([
  readFile(readmeUrl, 'utf8'),
  readFile(packageUrl, 'utf8').then(JSON.parse),
  readFile(loaderUrl, 'utf8'),
  readFile(declarationUrl, 'utf8'),
  readFile(nativeOptionsUrl, 'utf8'),
  readFile(coreOptionsUrl, 'utf8'),
])

function assertReadmeContract(candidate) {
  assert.match(candidate, /^## Install$/m, 'README must have an install section')
  assert.match(candidate, new RegExp(`npm install ${packageJson.name}`), 'README must show npm install')
  assert.match(candidate, new RegExp(`pnpm add ${packageJson.name}`), 'README must show pnpm add')

  const minimumNode = packageJson.engines.node.match(/^>=(\d+)/)?.[1]
  assert.ok(minimumNode, 'package.json must declare a minimum Node major version')
  assert.match(
    candidate,
    new RegExp(`Node\\.js ${minimumNode} or newer`),
    'README must state the package Node.js requirement',
  )

  assert.match(candidate, /^## Untrusted by default$/m, 'README must explain the default trust boundary')
  assert.match(declarations, /export type RenderPolicy = 'untrusted' \| 'trusted'/, 'declarations must expose both render policies')
  assert.match(nativeOptions, /let mut options = CoreOptions::default\(\)/, 'Node options must start from core defaults')
  const coreDefaultOptions = coreOptions.match(/impl Default for Options \{.*?\n\}/s)?.[0]
  assert.ok(coreDefaultOptions, 'core must define default options')
  assert.match(coreDefaultOptions, /render_policy: RenderPolicy::Untrusted/, 'core default policy must be untrusted')
  assert.match(candidate, /renderPolicy: 'untrusted'/, 'README must name the default render policy')
  assert.match(candidate, /raw HTML\s+is escaped/i, 'README must describe raw HTML escaping')
  assert.match(candidate, /unsafe link and image URL schemes/i, 'README must describe URL filtering')
  assert.match(candidate, /renderPolicy: 'trusted'/, 'README must show the trusted opt-in')
  assert.match(candidate, /only when the Markdown source is trusted/i, 'README must scope trusted mode')
  assert.match(
    candidate,
    /'<p><span class="note">Internal note<\/span><\/p>\\n'/,
    'README must show the actual trusted inline HTML output',
  )
  assert.match(candidate, /\[`Options`\]\(\.\/index\.d\.mts\)/, 'README must link the typed options')

  assert.match(candidate, /^## Troubleshooting native loading$/m, 'README must explain lazy loader failures')
  assert.match(candidate, /first call to `toHtml\(\)`, `transform\(\)`, or a\s+highlighter helper/i)

  assert.match(loader, /musl is not supported/, 'loader must retain its musl diagnostic')
  assert.match(candidate, /Alpine or another musl Linux environment/, 'README must cover the musl loader error')
  assert.match(loader, /does not support \$\{process\.platform\}/, 'loader must retain unsupported-target diagnostics')
  assert.match(candidate, /Unsupported platform or architecture/, 'README must cover unsupported targets')
  assert.match(loader, /does not include a native binary/, 'loader must retain missing-binary diagnostics')
  assert.match(candidate, /`does not include a native binary`/, 'README must cover missing binaries')

  const loaderTargets = [...loader.matchAll(/'([a-z0-9]+-(?:arm64|x64))': '([a-z0-9-]+)'/g)]
    .map(([, host, target]) => ({ host, target }))
  const packageTargets = packageJson.napi.targets.map(packageTargetToBindingTarget)
  assert.deepEqual(
    loaderTargets.map(({ target }) => target).sort(),
    packageTargets.slice().sort(),
    'loader targets and published native targets must stay aligned',
  )
  assert.match(candidate, /glibc\s+Linux, macOS, and Windows on x64 and arm64/, 'README must state published platforms')
  assert.match(candidate, /no WASM fallback/, 'README must state the missing fallback')
}

function packageTargetToBindingTarget(target) {
  const match = target.match(/^(aarch64|x86_64)-(apple|unknown-linux|pc-windows)-(darwin|gnu|msvc)$/)
  assert.ok(match, `unsupported napi target format: ${target}`)

  const [, architecture, platform, abi] = match
  const hostPlatform = {
    apple: 'darwin',
    'unknown-linux': 'linux',
    'pc-windows': 'win32',
  }[platform]
  const hostArchitecture = architecture === 'aarch64' ? 'arm64' : 'x64'
  const binding = `${hostPlatform}-${hostArchitecture}`
  return abi === 'darwin' ? binding : `${binding}-${abi}`
}

assertReadmeContract(readme)

assert.throws(
  () => assertReadmeContract(readme.replace(`npm install ${packageJson.name}`, 'npm install')),
  /npm install/,
  'install documentation contract must reject a missing package name',
)
assert.throws(
  () => assertReadmeContract(readme.replace("renderPolicy: 'untrusted'", "renderPolicy: 'trusted'")),
  /default render policy/,
  'security documentation contract must reject a missing untrusted default',
)
assert.throws(
  () => assertReadmeContract(readme.replace('<p><span class="note">Internal note</span></p>\\n', '<span class="note">Internal note</span>')),
  /actual trusted inline HTML output/,
  'security documentation contract must reject incorrect trusted output',
)
assert.throws(
  () => assertReadmeContract(readme.replace('Alpine or another musl Linux environment', 'Linux environment')),
  /musl loader error/,
  'loader troubleshooting contract must reject missing musl guidance',
)

console.log('Node README package, security, and loader contract checks passed')
