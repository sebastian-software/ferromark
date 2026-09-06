import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { ContractError, escapeRegExp, readRepositoryFile } from './lib/contracts.mjs'

const MIGRATION_GUIDE = '[0.4–0.7 migration guide](docs/migration-0.4.md)'
const PROJECT_STRUCTURE_FILES = ['highlight.rs', 'events.rs', 'strict.rs']

function failContract(message) {
  throw new ContractError(`README structure contract: ${message}`)
}

function section(document, heading) {
  const marker = `## ${heading}\n`
  const start = document.indexOf(marker)
  if (start === -1) {
    failContract(`must contain a ${JSON.stringify(heading)} section`)
  }
  const contentStart = start + marker.length
  const following = document.slice(contentStart).search(/^## /m)
  return following === -1
    ? document.slice(contentStart)
    : document.slice(contentStart, contentStart + following)
}

function lockedBenchmarkVersion(packageName) {
  const lockfile = readRepositoryFile('benchmarks/md4c-comparison/Cargo.lock')
  const match = lockfile.match(
    new RegExp(`^name = "${escapeRegExp(packageName)}"\\nversion = "([^"]+)"$`, 'm'),
  )
  if (!match) {
    failContract(`benchmark lockfile must contain ${packageName}`)
  }
  return match[1]
}

function pinnedMd4cRevision() {
  const buildScript = readRepositoryFile('benchmarks/md4c-comparison/build.rs')
  const revision = buildScript.match(/^const MD4C_REVISION: &str = "([0-9a-f]{40})";$/m)?.[1]
  if (!revision) {
    failContract('benchmark build must declare a full MD4C_REVISION')
  }
  return revision
}

function validate(
  document,
  {
    contributing = readRepositoryFile('CONTRIBUTING.md'),
    performancePlan = readRepositoryFile('docs/arch/ARCH-PLAN-001-performance-opportunities.md'),
  } = {},
) {
  const headings = [...document.matchAll(/^## (.+)$/gm)].map(match => match[1])
  if (new Set(headings).size !== headings.length) {
    failContract('must not repeat top-level headings')
  }

  for (const heading of headings) {
    if (section(document, heading).trim().length === 0) {
      failContract(`${JSON.stringify(heading)} must not be empty`)
    }
  }

  const cliStart = document.indexOf('## CLI\n')
  const configurationStart = document.indexOf('## Markdown configuration\n')
  if (cliStart === -1 || configurationStart === -1 || cliStart >= configurationStart) {
    failContract('CLI must precede Markdown configuration so each section owns its content')
  }

  const cli = section(document, 'CLI')
  const configuration = section(document, 'Markdown configuration')
  if (!cli.includes('cargo install ferromark')) {
    failContract('CLI must document installation')
  }
  if (!cli.includes('--trusted')) {
    failContract('CLI must document trusted mode')
  }
  if (!configuration.includes('Options::minimal()')) {
    failContract('Markdown configuration must describe presets')
  }
  if (!configuration.includes('`Options` is non-exhaustive')) {
    failContract('Markdown configuration must document Options construction')
  }
  if (!document.includes(MIGRATION_GUIDE)) {
    failContract('README must preserve the migration guide link')
  }

  const benchmarks = section(document, 'Benchmarks')
  if (!benchmarks.includes('These rankings are Apple Silicon results only')) {
    failContract('Benchmarks must scope published rankings to Apple Silicon')
  }
  if (!benchmarks.includes('has not been re-measured') || !benchmarks.includes('x86-64')) {
    failContract('Benchmarks must disclose the missing x86-64 comparison')
  }
  for (const packageName of ['pulldown-cmark', 'comrak']) {
    const lockedVersion = lockedBenchmarkVersion(packageName)
    const pattern = new RegExp(`${escapeRegExp(packageName)}\\s+${escapeRegExp(lockedVersion)}`)
    if (!pattern.test(benchmarks)) {
      failContract(`Benchmarks must state locked ${packageName} ${lockedVersion}`)
    }
  }

  const shortRevision = pinnedMd4cRevision().slice(0, 7)
  if (!benchmarks.includes(`md4c @ ${shortRevision}`)) {
    failContract(`Benchmarks must state pinned md4c revision ${shortRevision}`)
  }
  if (
    !benchmarks.includes(`checkout --detach ${shortRevision}`) ||
    !contributing.includes(`checkout --detach ${shortRevision}`)
  ) {
    failContract(`README and CONTRIBUTING must check out pinned md4c revision ${shortRevision}`)
  }
  for (const instructions of [benchmarks, contributing]) {
    if (
      !instructions.includes('cargo bench --locked') ||
      !instructions.includes('--manifest-path benchmarks/md4c-comparison/Cargo.toml')
    ) {
      failContract('README and CONTRIBUTING must use the locked isolated benchmark manifest')
    }
  }
  if (performancePlan.includes('PERF_ATTEMPTS.md')) {
    failContract('Performance plan must not reference the removed PERF_ATTEMPTS.md')
  }
  const comparisonCommands = [
    ...performancePlan.matchAll(/`([^`\n]*cargo bench[^`\n]*comparison[^`\n]*)`/g),
  ].map(match => match[1])
  if (comparisonCommands.length === 0) {
    failContract('Performance plan must document the comparison benchmark command')
  }
  if (
    !comparisonCommands.every(
      command =>
        command.includes('MD4C_DIR=/path/to/md4c cargo bench --locked') &&
        command.includes('--manifest-path benchmarks/md4c-comparison/Cargo.toml'),
    )
  ) {
    failContract('Every performance-plan comparison command must use the locked isolated benchmark')
  }
  if (!document.includes('baseline SSE2 (x86-64)')) {
    failContract('README must describe the x86-64 inline SIMD path')
  }

  const projectStructure = section(document, 'Project structure')
  for (const filename of PROJECT_STRUCTURE_FILES) {
    if (!projectStructure.includes(filename)) {
      failContract(`Project structure must include ${filename}`)
    }
  }
}

const document = readRepositoryFile('README.md')

describe('README structure contract', () => {
  it('accepts the current README', () => {
    validate(document)
  })

  it('rejects an empty Markdown configuration section', () => {
    assert.throws(
      () =>
        validate(
          document.replace(
            '## Markdown configuration\n\nStart from',
            '## Markdown configuration\n\n## Configuration details\n\nStart from',
          ),
        ),
      ContractError,
    )
  })

  it('rejects misowned Markdown configuration content', () => {
    assert.throws(() => validate(document.replace('## Markdown configuration\n', '')), ContractError)
  })

  it('rejects renamed CLI trusted guidance', () => {
    assert.throws(() => validate(document.replaceAll('--trusted', '--safe')), ContractError)
  })

  it('rejects a missing migration guide link', () => {
    assert.throws(() => validate(document.replace(MIGRATION_GUIDE, 'migration guide')), ContractError)
  })

  it('rejects an unscoped Apple Silicon benchmark claim', () => {
    assert.throws(
      () =>
        validate(
          document.replace(
            'These rankings are Apple Silicon results only',
            'These rankings apply everywhere',
          ),
        ),
      ContractError,
    )
  })

  it('rejects a dropped x86-64 benchmark caveat', () => {
    assert.throws(
      () => validate(document.replace('has not been re-measured', 'has been re-measured')),
      ContractError,
    )
  })

  it('rejects a stale locked comrak version', () => {
    const lockedVersion = lockedBenchmarkVersion('comrak')
    assert.throws(
      () =>
        validate(
          document.replace(
            new RegExp(`comrak\\s+${escapeRegExp(lockedVersion)}`),
            'comrak 0.0.0',
          ),
        ),
      ContractError,
    )
  })

  it('rejects an unpinned md4c contributor checkout', () => {
    const revision = pinnedMd4cRevision().slice(0, 7)
    assert.throws(
      () =>
        validate(document, {
          contributing: readRepositoryFile('CONTRIBUTING.md').replace(
            `checkout --detach ${revision}`,
            'checkout --detach main',
          ),
        }),
      ContractError,
    )
  })

  it('rejects a reference to the removed performance evidence file', () => {
    assert.throws(
      () =>
        validate(document, {
          performancePlan: `References PERF_ATTEMPTS.md\n${readRepositoryFile('docs/arch/ARCH-PLAN-001-performance-opportunities.md')}`,
        }),
      ContractError,
    )
  })

  it('rejects a non-isolated performance benchmark command', () => {
    assert.throws(
      () =>
        validate(document, {
          performancePlan: readRepositoryFile(
            'docs/arch/ARCH-PLAN-001-performance-opportunities.md',
          ).replaceAll('MD4C_DIR=/path/to/md4c cargo bench --locked', 'cargo bench'),
        }),
      ContractError,
    )
  })

  for (const filename of PROJECT_STRUCTURE_FILES) {
    it(`rejects a project structure without ${filename}`, () => {
      assert.throws(
        () => validate(document.replace(filename, 'omitted-source-file.rs')),
        ContractError,
      )
    })
  }
})
