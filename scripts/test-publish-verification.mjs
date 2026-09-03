import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const workflowPath = path.join(root, '.github', 'workflows', 'publish.yml')
const workflow = await readFile(workflowPath, 'utf8')
const { MAX_ATTEMPTS, registryVersionUrl, verifyNpmPublication } = await import(
  pathToFileURL(path.join(root, 'node', 'scripts', 'verify-npm-publish.mjs')).href,
)

function verifierJob(source) {
  const marker = '\n  verify-npm-publish:\n'
  const start = source.indexOf(marker)
  assert.notEqual(start, -1, 'publish workflow must define verify-npm-publish')
  const remaining = source.slice(start + marker.length)
  const nextJob = remaining.search(/\n  [a-z0-9-]+:\n/)
  return nextJob === -1 ? remaining : remaining.slice(0, nextJob)
}

function assertValidVerifierJob(source) {
  const job = verifierJob(source)
  assert.match(job, /needs:\n      - release-please\n      - publish-npm/)
  assert.match(
    job,
    /if: \$\{\{ always\(\) && needs\.release-please\.outputs\.releases_created == 'true' \}\}/,
  )
  assert.match(job, /timeout-minutes: 5/)
  assert.match(job, /NPM_PUBLISH_RESULT: \$\{\{ needs\.publish-npm\.result \}\}/)
  assert.match(job, /run: node \.\/node\/scripts\/verify-npm-publish\.mjs/)
}

function assertCratePublicationWaitsForVerification(source) {
  const crateJob = source.slice(
    source.indexOf('\n  publish-crate:\n'),
    source.indexOf('\n  build-native:\n'),
  )
  assert.match(
    crateJob,
    /needs:\n      - release-please\n      - verify-npm-publish/,
    'crate publication must wait for successful npm registry verification',
  )
  assert.doesNotMatch(crateJob, /- publish-npm/)
}

function assertPlatformPackagesArePublished(source) {
  for (const artifact of [
    'darwin-arm64',
    'darwin-x64',
    'linux-arm64-gnu',
    'linux-arm64-musl',
    'linux-x64-gnu',
    'linux-x64-musl',
    'win32-arm64-msvc',
    'win32-x64-msvc',
  ]) {
    assert.match(source, new RegExp(`artifact: ${artifact}\\n`))
  }

  const assembleIndex = source.indexOf(
    'run: pnpm --dir ferromark exec napi artifacts --output-dir .napi-artifacts --npm-dir npm',
  )
  const verifyIndex = source.indexOf('run: node ./scripts/verify-release.mjs')
  const platformPublishIndex = source.indexOf('run: node ./scripts/publish-platform-packages.mjs')
  const mainPublishIndex = source.indexOf('run: npm publish --access public --provenance')
  assert.ok(assembleIndex !== -1 && assembleIndex < verifyIndex)
  assert.ok(verifyIndex < platformPublishIndex)
  assert.ok(platformPublishIndex < mainPublishIndex)
}

assertValidVerifierJob(workflow)
assertCratePublicationWaitsForVerification(workflow)
assertPlatformPackagesArePublished(workflow)
assert.throws(
  () => assertValidVerifierJob(workflow.replace('\n  verify-npm-publish:\n', '\n  npm-check:\n')),
  /verify-npm-publish/,
)
assert.throws(
  () =>
    assertValidVerifierJob(
      workflow.replace(
        '  verify-npm-publish:\n    needs:\n      - release-please\n      - publish-npm\n',
        '  verify-npm-publish:\n    needs:\n      - release-please\n      - build-native\n',
      ),
    ),
  /publish-npm/,
)
assert.throws(
  () => assertValidVerifierJob(workflow.replace('always() && ', '')),
  /always/,
)
assert.throws(
  () => assertValidVerifierJob(workflow.replace('timeout-minutes: 5', 'timeout-minutes: 30')),
  /timeout-minutes: 5/,
)
assert.throws(
  () =>
    assertCratePublicationWaitsForVerification(
      workflow.replace('- verify-npm-publish', '- publish-npm'),
    ),
  /crate publication must wait for successful npm registry verification/,
)

assert.equal(registryVersionUrl('ferromark', '0.7.0'), 'https://registry.npmjs.org/ferromark/0.7.0')

let attempts = 0
await verifyNpmPublication({
  packageName: 'ferromark',
  version: '0.7.0',
  publishResult: 'success',
  fetchImpl: async () => {
    attempts += 1
    return {
      ok: attempts === 2,
      status: attempts === 2 ? 200 : 404,
      json: async () => ({ name: 'ferromark', version: '0.7.0' }),
    }
  },
  sleepImpl: async () => {},
})
assert.equal(attempts, 2)

let wrongVersionAttempts = 0
await assert.rejects(
  verifyNpmPublication({
    packageName: 'ferromark',
    version: '0.7.0',
    publishResult: 'success',
    fetchImpl: async () => {
      wrongVersionAttempts += 1
      return {
        ok: true,
        status: 200,
        json: async () => ({ name: 'ferromark', version: '0.6.0' }),
      }
    },
    sleepImpl: async () => {},
  }),
  /expected ferromark@0\.7\.0/,
)
assert.equal(wrongVersionAttempts, MAX_ATTEMPTS, 'registry polling must stay bounded')

await assert.rejects(
  verifyNpmPublication({
    packageName: 'ferromark',
    version: '0.7.0',
    publishResult: 'failure',
    fetchImpl: async () => ({
      ok: true,
      status: 200,
      json: async () => ({ name: 'wrong-package', version: '0.7.0' }),
    }),
    sleepImpl: async () => {},
  }),
  (error) => {
    assert.match(error.message, /publish-npm concluded failure/)
    assert.match(error.message, /registry returned wrong-package@0\.7\.0/)
    return true
  },
)

console.log('Publish verification workflow and retry contract checks passed')
