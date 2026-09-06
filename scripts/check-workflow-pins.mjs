#!/usr/bin/env node
import { readdirSync, readFileSync, statSync } from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { parseAllDocuments } from 'yaml'

class WorkflowScanError extends Error {}

const PINNED = /^[^\s@]+@[0-9a-fA-F]{40}$/
const SELF_REPOSITORY = /^\$\/[^\s@]+$/

function ensureAcyclic(node, ancestors = new Set()) {
  if (node === null || typeof node !== 'object') {
    return
  }
  if (ancestors.has(node)) {
    throw new WorkflowScanError('cyclic YAML aliases are not supported')
  }
  ancestors.add(node)
  if (Array.isArray(node)) {
    for (const value of node) {
      ensureAcyclic(value, ancestors)
    }
  } else {
    for (const value of Object.values(node)) {
      ensureAcyclic(value, ancestors)
    }
  }
  ancestors.delete(node)
}

function isMapping(value) {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function addUsesEntry(value, file, entries) {
  entries.push([file, typeof value === 'string' ? value : null])
}

function scanStep(step, file, entries) {
  if (isMapping(step) && 'uses' in step) {
    addUsesEntry(step.uses, file, entries)
  }
}

function scanJob(job, file, entries) {
  if (!isMapping(job)) {
    return
  }
  if ('uses' in job) {
    addUsesEntry(job.uses, file, entries)
  }
  if (Array.isArray(job.steps)) {
    for (const step of job.steps) {
      scanStep(step, file, entries)
    }
  }
}

function scanDocument(document, file, entries) {
  if (!isMapping(document) || !isMapping(document.jobs)) {
    return
  }
  for (const job of Object.values(document.jobs)) {
    scanJob(job, file, entries)
  }
}

function fail(stage, file, error) {
  process.stderr.write(`Failed to ${stage} workflow file ${file}: ${error.message}\n`)
  process.exit(2)
}

function scanFile(file, entries) {
  let source
  try {
    source = readFileSync(file, 'utf8')
  } catch (error) {
    fail('read', file, error)
  }

  // Alias resolution belongs to parsing: an unresolved alias is a broken
  // document, while a cycle only shows up once the document is materialized.
  let documents
  let document
  try {
    documents = parseAllDocuments(source, { merge: true })
    for (const parsed of documents) {
      if (parsed.errors.length > 0) {
        throw parsed.errors[0]
      }
    }
    document = documents.length === 1 ? documents[0].toJS({ maxAliasCount: -1 }) : undefined
  } catch (error) {
    fail('parse', file, error)
  }

  try {
    if (documents.length !== 1) {
      throw new WorkflowScanError('multiple YAML documents are not supported')
    }
    ensureAcyclic(document)
    scanDocument(document, file, entries)
  } catch (error) {
    fail('scan', file, error)
  }
}

function workflowFiles(directory) {
  const files = []
  const pending = [directory]
  while (pending.length > 0) {
    const current = pending.pop()
    const info = statSync(current)
    if (info.isDirectory()) {
      for (const entry of readdirSync(current).sort()) {
        pending.push(path.join(current, entry))
      }
    } else if (info.isFile() && /\.ya?ml$/.test(current)) {
      files.push(current)
    }
  }
  return files.sort()
}

const directory = process.argv[2]
if (directory === undefined) {
  process.stderr.write('usage: check-workflow-pins.mjs <workflow-directory>\n')
  process.exit(2)
}

const entries = []
let files
try {
  files = workflowFiles(directory)
} catch (error) {
  process.stderr.write(`Failed to scan workflow files: ${error.message}\n`)
  process.exit(2)
}
for (const file of files) {
  scanFile(file, entries)
}

const invalid = entries.filter(([, value]) => {
  if (typeof value !== 'string') {
    return true
  }
  return !(
    value.startsWith('./') ||
    value.startsWith('docker://') ||
    SELF_REPOSITORY.test(value) ||
    PINNED.test(value)
  )
})

if (invalid.length > 0) {
  process.stderr.write('Workflow actions must use full 40-character commit SHAs:\n')
  for (const [file, value] of invalid) {
    process.stderr.write(`${file}: uses: ${value ?? 'uses value is not a scalar'}\n`)
  }
  process.exit(1)
}
