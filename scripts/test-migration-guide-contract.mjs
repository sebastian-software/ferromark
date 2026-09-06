import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { describe, it } from 'node:test'

import { ContractError, readRepositoryFile, repositoryRoot } from './lib/contracts.mjs'

const GUIDE_PATH = 'docs/migration-0.4.md'
const EXAMPLES = [
  'profile-extended',
  'inline-parser-argument',
  'options-clone',
  'fenced-code-pattern',
  'parse-result-headings',
]

function failContract(message) {
  throw new ContractError(`migration guide contract: ${message}`)
}

function extractExample(document, name) {
  const marker = `<!-- migration-example: ${name} -->`
  const start = document.indexOf(marker)
  if (start === -1) {
    failContract(`must contain ${marker}`)
  }
  const block = document.slice(start + marker.length)
  const match = block.match(/^[ \t\r\n]*```rust\r?\n([\s\S]*?)^```[ \t]*\r?$/m)
  if (!match) {
    failContract(`${name} must be followed by a Rust code block`)
  }
  return match[1]
}

function assertContains(document, needle, message) {
  if (!document.includes(needle)) {
    failContract(message)
  }
}

function validateDocument({ guide, readme, cargoToml, nodePackage, nodeWorkspace, changelog }) {
  assertContains(guide, '# Migrating from ferromark 0.3 to 0.7', 'must name its supported upgrade range')
  for (const heading of [
    '## Before you start',
    '## 0.4: replace `Profile`',
    '## 0.5: update configurable inline parsing',
    '## 0.6: clone options and make integration matches forward-compatible',
    '## 0.7: raise runtime prerequisites',
    '## Validate the completed upgrade',
  ]) {
    assertContains(guide, heading, `must contain ${heading}`)
  }

  assertContains(
    readme,
    `[0.4–0.7 migration guide](${GUIDE_PATH})`,
    'README must link the 0.4–0.7 guide',
  )
  assertContains(guide, 'Options::from(Profile)', 'must explain the removed Profile conversion')
  assertContains(guide, 'inline_footnotes: bool', 'must name the inserted inline-parser argument')
  assertContains(guide, 'InlineFootnote(Range)', 'must name the added inline event')
  assertContains(guide, 'Options` no longer implements `Copy`', 'must explain the Options Copy removal')
  assertContains(guide, '`#[non_exhaustive]`', 'must explain the fenced-code forward-compatibility contract')
  assertContains(guide, '`headings`', 'must explain the ParseResult metadata field')
  for (const change of [
    'remove Profile and Options::from(Profile)',
    'a positional parse_with_options argument',
    'Options no longer implements Copy',
    'FencedCodeBlock gained the meta field and is now non_exhaustive',
    'ferromark now requires Rust 1.88 or newer',
    'ferromark npm package now requires Node.js 22 or newer',
  ]) {
    assertContains(changelog, change, `CHANGELOG must record ${JSON.stringify(change)}`)
  }

  const rustVersion = cargoToml.match(/^rust-version\s*=\s*"([^"]+)"\s*$/m)?.[1]
  if (!rustVersion) {
    failContract('Cargo.toml must declare rust-version')
  }
  assertContains(
    guide,
    `requires Rust ${rustVersion} or newer`,
    'must state Cargo.toml Rust requirement',
  )

  const nodeVersion = nodePackage.match(/"node"\s*:\s*">=([0-9]+(?:\.[0-9]+){0,2})"/)?.[1]
  if (!nodeVersion) {
    failContract('node package must declare a minimum Node version')
  }
  assertContains(guide, `Node.js ${nodeVersion} or newer`, 'must state npm package Node requirement')

  const workspaceNode = nodeWorkspace.match(/"node"\s*:\s*">=([0-9]+(?:\.[0-9]+){0,2})"/)?.[1]
  const workspacePnpm = nodeWorkspace.match(/"packageManager"\s*:\s*"pnpm@([^"]+)"/)?.[1]
  if (!workspaceNode) {
    failContract('node workspace must declare a minimum Node version')
  }
  if (!workspacePnpm) {
    failContract('node workspace must pin pnpm')
  }
  assertContains(guide, `pnpm ${workspacePnpm}`, 'must state the node workspace pnpm version')
  assertContains(
    guide,
    `Node.js ${workspaceNode} or newer`,
    'must state the node workspace Node requirement',
  )

  for (const name of EXAMPLES) {
    extractExample(guide, name)
  }
  assertContains(
    extractExample(guide, 'inline-parser-argument'),
    'false, // inline_footnotes',
    'inline parser example must preserve the new argument position',
  )
  assertContains(
    extractExample(guide, 'fenced-code-pattern'),
    '..',
    'fenced-code example must ignore future fields',
  )
}

function compileExamples(guide) {
  const directory = mkdtempSync(path.join(tmpdir(), 'ferromark-migration-guide.'))
  try {
    writeFileSync(
      path.join(directory, 'Cargo.toml'),
      `[package]\nname = "ferromark-migration-guide-contract"\nversion = "0.0.0"\nedition = "2024"\n\n[dependencies]\nferromark = { path = ${JSON.stringify(repositoryRoot)}, features = ["mdx"] }\n`,
    )
    mkdirSync(path.join(directory, 'src'), { recursive: true })
    const functions = EXAMPLES.map(name => {
      const code = extractExample(guide, name)
        .split('\n')
        .map(line => `  ${line}`)
        .join('\n')
      return `fn ${name.replaceAll('-', '_')}() {\n${code}\n}\n`
    }).join('\n')
    writeFileSync(
      path.join(directory, 'src/main.rs'),
      `#![allow(dead_code)]\n\n${functions}\nfn main() {}\n`,
    )

    const result = spawnSync(
      'cargo',
      ['check', '--quiet', '--manifest-path', path.join(directory, 'Cargo.toml')],
      { encoding: 'utf8' },
    )
    if (result.error) {
      failContract(`cargo check could not be started: ${result.error.message}`)
    }
    if (result.status !== 0) {
      failContract(`documented Rust examples must compile:\n${result.stdout}${result.stderr}`)
    }
  } finally {
    rmSync(directory, { force: true, recursive: true })
  }
}

const inputs = {
  guide: readRepositoryFile(GUIDE_PATH),
  readme: readRepositoryFile('README.md'),
  cargoToml: readRepositoryFile('Cargo.toml'),
  nodePackage: readRepositoryFile('node/ferromark/package.json'),
  nodeWorkspace: readRepositoryFile('node/package.json'),
  changelog: readRepositoryFile('CHANGELOG.md'),
}

function assertRejected(overrides) {
  assert.throws(() => validateDocument({ ...inputs, ...overrides }), ContractError)
}

describe('migration guide contract', () => {
  it('accepts the current guide', () => {
    validateDocument(inputs)
  })

  it('compiles every documented example', () => {
    compileExamples(inputs.guide)
  })

  it('rejects a missing README guide link', () => {
    assertRejected({
      readme: inputs.readme.replace(`[0.4–0.7 migration guide](${GUIDE_PATH})`, 'migration guide'),
    })
  })

  it('rejects a stale inline parser argument', () => {
    assertRejected({
      guide: inputs.guide.replace('false, // inline_footnotes', 'None, // footnote_store'),
    })
  })

  it('rejects a fenced-code example without the fallback pattern', () => {
    assertRejected({ guide: inputs.guide.replace('            ..\n', '') })
  })

  it('rejects a stale Rust version', () => {
    assertRejected({
      guide: inputs.guide.replace('requires Rust 1.94 or newer', 'requires Rust 1.85 or newer'),
    })
  })

  it('rejects a stale Node version', () => {
    assertRejected({
      guide: inputs.guide.replace('Node.js 22.12.0 or newer', 'Node.js 20.0.0 or newer'),
    })
  })
})
