import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { ContractError, deepCopy, readRepositoryFile, readYaml } from './lib/contracts.mjs'

const CHECKOUT_ACTION = 'actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1'
const RUST_TOOLCHAIN_ACTION = 'dtolnay/rust-toolchain@2c7215f132e9ebf062739d9130488b56d53c060c'
const INSTALL_ACTION = 'taiki-e/install-action@e67fa11c4b9316fa714ddf0abed07a0c3143b95b'
const RUST_CACHE_ACTION = 'Swatinem/rust-cache@42dc69e1aa15d09112580998cf2ef0119e2e91ae'
const CODECOV_ACTION = 'codecov/codecov-action@fb8b3582c8e4def4969c97caa2f19720cb33a72f'
const CARGO_DENY_ACTION = 'EmbarkStudios/cargo-deny-action@3c6349835b2b7b196a839186cb8b78e02f7b5f25'
const RUSTDOC_COMMAND = "RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features --locked"
const COVERAGE_COMMAND = 'cargo llvm-cov --all-features --locked --no-report'
const COVERAGE_REPORT_PREFIX =
  'cargo llvm-cov report --lcov --output-path lcov.info --fail-under-lines '
const CODECOV_INPUTS = {
  files: 'lcov.info',
  fail_ci_if_error: false,
  token: '${{ secrets.CODECOV_TOKEN }}',
}

function failContract(message) {
  throw new ContractError(`CI hardening contract: ${message}`)
}

function readContributing() {
  return readRepositoryFile('CONTRIBUTING.md')
}

function readDependencyPolicy() {
  return readRepositoryFile('deny.toml')
}

function validate(
  workflow,
  { contributing = readContributing(), dependencyPolicy = readDependencyPolicy() } = {},
) {
  assertDeepEqual(workflow.permissions, { contents: 'read' }, () =>
    failContract('top-level permissions must grant contents: read only'),
  )

  const jobs = workflow.jobs
  assertDeepEqual(jobs.rustsec?.permissions, { contents: 'read', checks: 'write' }, () =>
    failContract('rustsec permissions must grant only contents: read and checks: write'),
  )

  const fmtCommands = jobs.fmt.steps.map(step => step.run).filter(run => run !== undefined)
  if (!fmtCommands.includes(RUSTDOC_COMMAND)) {
    failContract('fmt job must reject rustdoc warnings for all features')
  }

  const coverage = jobs.coverage
  if (coverage['runs-on'] !== 'ubuntu-latest') {
    failContract('coverage job must run on ubuntu-latest')
  }

  const steps = coverage.steps
  const expectedActions = [
    CHECKOUT_ACTION,
    RUST_TOOLCHAIN_ACTION,
    INSTALL_ACTION,
    RUST_CACHE_ACTION,
    CODECOV_ACTION,
  ]
  const actualActions = steps.map(step => step.uses).filter(uses => uses !== undefined)
  assertDeepEqual(actualActions, expectedActions, () =>
    failContract(`coverage actions must be pinned and ordered as ${JSON.stringify(expectedActions)}`),
  )

  const toolchain = steps.find(step => step.uses === RUST_TOOLCHAIN_ACTION).with
  assertDeepEqual(toolchain, { toolchain: 'stable', components: 'llvm-tools-preview' }, () =>
    failContract('coverage must install stable Rust with llvm-tools-preview'),
  )

  const installer = steps.find(step => step.uses === INSTALL_ACTION).with
  assertDeepEqual(installer, { tool: 'cargo-llvm-cov' }, () =>
    failContract('coverage must install cargo-llvm-cov'),
  )

  const commands = steps.map(step => step.run).filter(run => run !== undefined)
  if (commands.length !== 2 || commands[0] !== COVERAGE_COMMAND) {
    failContract('coverage must run cargo llvm-cov for all features with the lockfile')
  }

  const reportCommand = commands[1]
  if (!reportCommand.startsWith(COVERAGE_REPORT_PREFIX)) {
    failContract('coverage must report lcov.info and fail under a line coverage floor')
  }

  const threshold = reportCommand.slice(COVERAGE_REPORT_PREFIX.length)
  if (!/^\d+$/.test(threshold)) {
    failContract('the coverage floor must be a plain percentage')
  }

  const upload = steps.find(step => step.uses === CODECOV_ACTION).with
  assertDeepEqual(upload, CODECOV_INPUTS, () =>
    failContract(`coverage must upload lcov.info as ${JSON.stringify(CODECOV_INPUTS)}`),
  )

  if (!contributing.includes(`${threshold}% line coverage`)) {
    failContract(`CONTRIBUTING.md must document the ${threshold}% line coverage floor`)
  }

  const dependencyJob = jobs['cargo-deny']
  if (!dependencyJob) {
    failContract('a cargo-deny job must enforce the dependency policy')
  }
  const dependencySteps = dependencyJob.steps
  const dependencyActions = dependencySteps.map(step => step.uses).filter(uses => uses !== undefined)
  assertDeepEqual(dependencyActions, [CHECKOUT_ACTION, CARGO_DENY_ACTION], () =>
    failContract(
      `cargo-deny actions must be pinned and ordered as ${JSON.stringify([CHECKOUT_ACTION, CARGO_DENY_ACTION])}`,
    ),
  )

  const policyInputs = dependencySteps.find(step => step.uses === CARGO_DENY_ACTION).with
  assertDeepEqual(policyInputs, { command: 'check', arguments: '' }, () =>
    failContract('cargo-deny must run every check with the arguments from deny.toml'),
  )

  if (!dependencyPolicy.includes('yanked = "deny"')) {
    failContract('deny.toml must reject yanked crates')
  }
}

function assertDeepEqual(actual, expected, onFailure) {
  try {
    assert.deepEqual(actual, expected)
  } catch {
    onFailure()
  }
}

function assertRejected(mutate, options = {}) {
  const copy = deepCopy(workflow)
  mutate(copy)
  assert.throws(() => validate(copy, options), ContractError)
}

const workflow = readYaml('.github', 'workflows', 'ci.yml')

describe('CI hardening contract', () => {
  it('accepts the current workflow', () => {
    validate(workflow)
  })

  it('rejects a write-scoped top-level token', () => {
    assertRejected(copy => {
      copy.permissions.contents = 'write'
    })
  })

  it('rejects a missing rustsec checks permission', () => {
    assertRejected(copy => {
      delete copy.jobs.rustsec.permissions.checks
    })
  })

  it('rejects a missing rustdoc warning gate', () => {
    assertRejected(copy => {
      copy.jobs.fmt.steps = copy.jobs.fmt.steps.filter(step => step.run !== RUSTDOC_COMMAND)
    })
  })

  it('rejects a mutable coverage action', () => {
    assertRejected(copy => {
      const step = copy.jobs.coverage.steps.find(candidate => candidate.uses === INSTALL_ACTION)
      step.uses = 'taiki-e/install-action@v2'
    })
  })

  it('rejects coverage without all features', () => {
    assertRejected(copy => {
      const step = copy.jobs.coverage.steps.find(candidate => candidate.run === COVERAGE_COMMAND)
      step.run = 'cargo llvm-cov --locked --no-report'
    })
  })

  it('rejects a missing coverage upload', () => {
    assertRejected(copy => {
      copy.jobs.coverage.steps = copy.jobs.coverage.steps.filter(
        step => step.uses !== CODECOV_ACTION,
      )
    })
  })

  it('rejects a mutable coverage upload action', () => {
    assertRejected(copy => {
      const step = copy.jobs.coverage.steps.find(candidate => candidate.uses === CODECOV_ACTION)
      step.uses = 'codecov/codecov-action@v7'
    })
  })

  it('rejects coverage without a floor', () => {
    assertRejected(copy => {
      const step = copy.jobs.coverage.steps.find(candidate =>
        candidate.run?.startsWith(COVERAGE_REPORT_PREFIX),
      )
      step.run = 'cargo llvm-cov report --lcov --output-path lcov.info'
    })
  })

  it('rejects an undocumented coverage floor', () => {
    assertRejected(() => {}, {
      contributing: readContributing().replace(/\d+% line coverage/g, 'an unstated% line coverage'),
    })
  })

  it('rejects a missing dependency policy job', () => {
    assertRejected(copy => {
      delete copy.jobs['cargo-deny']
    })
  })

  it('rejects a mutable cargo-deny action', () => {
    assertRejected(copy => {
      const step = copy.jobs['cargo-deny'].steps.find(
        candidate => candidate.uses === CARGO_DENY_ACTION,
      )
      step.uses = 'EmbarkStudios/cargo-deny-action@v2'
    })
  })

  it('rejects a dependency policy that allows yanked crates', () => {
    assertRejected(() => {}, {
      dependencyPolicy: readDependencyPolicy().replace('yanked = "deny"', 'yanked = "warn"'),
    })
  })
})
