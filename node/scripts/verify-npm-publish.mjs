import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

export const MAX_ATTEMPTS = 8
export const RETRY_DELAY_MS = 15_000
export const REQUEST_TIMEOUT_MS = 10_000

export function registryVersionUrl(packageName, version) {
  return `https://registry.npmjs.org/${encodeURIComponent(packageName)}/${encodeURIComponent(version)}`
}

const sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds))

export async function verifyNpmPublication({
  packageName,
  version,
  publishResult,
  fetchImpl = fetch,
  sleepImpl = sleep,
}) {
  const registryUrl = registryVersionUrl(packageName, version)
  let published = false
  let lastObservation = 'the registry did not return the expected package metadata'

  for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt += 1) {
    try {
      const response = await fetchImpl(registryUrl, {
        cache: 'no-store',
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      })

      if (response.ok) {
        const metadata = await response.json()
        if (metadata.name === packageName && metadata.version === version) {
          published = true
          console.log(`npm registry confirmed ${packageName}@${version} on attempt ${attempt}`)
          break
        }
        lastObservation = `registry returned ${metadata.name ?? 'unknown'}@${metadata.version ?? 'unknown'}`
      } else {
        lastObservation = `registry returned HTTP ${response.status}`
      }
    } catch (error) {
      lastObservation = `registry request failed: ${error.message}`
    }

    console.log(
      `npm release verification attempt ${attempt}/${MAX_ATTEMPTS}: ${lastObservation}`,
    )
    if (attempt < MAX_ATTEMPTS) {
      await sleepImpl(RETRY_DELAY_MS)
    }
  }

  const failures = []
  if (publishResult !== 'success') {
    failures.push(`publish-npm concluded ${publishResult}`)
  }
  if (!published) {
    failures.push(`expected ${packageName}@${version} at ${registryUrl}; ${lastObservation}`)
  }
  if (failures.length > 0) {
    throw new Error(`npm release verification failed: ${failures.join('; ')}`)
  }
}

async function main() {
  const scriptsDirectory = path.dirname(fileURLToPath(import.meta.url))
  const packageJsonPath = path.join(scriptsDirectory, '..', 'ferromark', 'package.json')
  const packageJson = JSON.parse(await readFile(packageJsonPath, 'utf8'))

  if (typeof packageJson.name !== 'string' || typeof packageJson.version !== 'string') {
    throw new Error(`Invalid npm package manifest: ${packageJsonPath}`)
  }

  await verifyNpmPublication({
    packageName: packageJson.name,
    version: packageJson.version,
    publishResult: process.env.NPM_PUBLISH_RESULT ?? 'unknown',
  })
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(error.message)
    process.exitCode = 1
  })
}
