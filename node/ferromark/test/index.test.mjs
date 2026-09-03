import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { copyFile, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises'
import { createRequire } from 'node:module'
import { tmpdir } from 'node:os'
import path from 'node:path'
import process from 'node:process'
import test from 'node:test'
import { fileURLToPath, pathToFileURL } from 'node:url'

import {
  Renderer,
  toHtml,
  toHtmlBuffer,
  toHtmlWithHighlighter,
  transform,
  transformWithHighlighter,
} from '../index.mjs'
import { nativeTarget } from '../native-target.mjs'

test('renders Markdown through the native binding', () => {
  assert.equal(toHtml('# Hello'), '<h1 id="hello">Hello</h1>\n')
})

test('renders UTF-8 HTML directly into Node.js Buffers', () => {
  const expected = '<h1 id="grüße">Grüße</h1>\n'
  const output = toHtmlBuffer('# Grüße')

  assert.ok(Buffer.isBuffer(output))
  assert.equal(output.toString('utf8'), expected)

  const renderer = new Renderer({ headingIds: false })
  const reusedOutput = renderer.toHtmlBuffer('# Grüße')
  assert.ok(Buffer.isBuffer(reusedOutput))
  assert.equal(reusedOutput.toString('utf8'), '<h1>Grüße</h1>\n')
})

test('rejects non-string Markdown across every public render entry point', () => {
  const highlighter = { codeToHtml: () => '<pre><code></code></pre>\n' }
  const calls = [
    () => toHtml(123),
    () => toHtmlBuffer(123),
    () => transform(null),
    () => new Renderer().toHtml({}),
    () => new Renderer().toHtmlBuffer({}),
    () => toHtmlWithHighlighter(123, highlighter, { theme: 'dark' }),
    () => transformWithHighlighter(123, highlighter, { theme: 'dark' }),
  ]

  for (const call of calls) {
    assert.throws(call, error => error instanceof Error && /string/i.test(error.message))
  }
})

test('reuses a renderer without leaking document state', () => {
  const renderer = new Renderer({ footnotes: true })

  assert.match(renderer.toHtml('# Same\n\n# Same\n\nA[^a]\n\n[^a]: First'), /id="same-1"/)
  assert.equal(
    renderer.toHtml('# Same\n\n[local][ref]\n\n[^b]: Unused'),
    '<h1 id="same">Same</h1>\n<p>[local][ref]</p>\n',
  )
})

test('validates reusable renderer options at construction', () => {
  assert.throws(
    () => new Renderer({ taskList: true }),
    error => error instanceof TypeError && /unknown option.*taskList/i.test(error.message),
  )
})

test('maps typed options to the Rust surface', () => {
  assert.equal(toHtml('==mark==', { highlight: true }), '<p><mark>mark</mark></p>\n')
  assert.match(
    toHtml('| Short | Long |\n| -- | ------ |', { tableColumnWidths: true }),
    /<col style="width: 25%">/,
  )
  assert.match(
    toHtml('| A | B |\n| --- | --- |\n| merged ||', { mergedTableCells: true }),
    /colspan="2"/,
  )
  assert.match(
    toHtml('Text.^[Node note.]', { inlineFootnotes: true }),
    /user-content-inline-fn-1/,
  )
  assert.equal(
    toHtml('Term\n: Definition', { definitionLists: true }),
    '<dl>\n<dt>Term</dt>\n<dd>Definition</dd>\n</dl>\n',
  )
  assert.equal(toHtml('// private note', { lineComments: true }), '')
  assert.equal(
    toHtml('    code', { indentedCodeBlocks: false }),
    '<p>code</p>\n',
  )
  assert.throws(
    () => toHtml('text', { renderPolicy: 'invalid' }),
    /renderPolicy must be either 'untrusted' or 'trusted'/,
  )
})

test('requires trusted rendering before allowHtml passes raw HTML through', () => {
  assert.equal(
    toHtml('<i>content</i>', { allowHtml: true }),
    '<p>&lt;i&gt;content&lt;/i&gt;</p>\n',
  )
  assert.equal(
    toHtml('<i>content</i>', { renderPolicy: 'trusted', allowHtml: true }),
    '<p><i>content</i></p>\n',
  )
})

test('rejects unknown option keys across every public render entry point', () => {
  const highlighter = {
    codeToHtml() {
      return '<pre><code></code></pre>\n'
    },
  }
  const calls = [
    () => toHtml('text', { taskList: true }),
    () => toHtmlBuffer('text', { taskList: true }),
    () => transform('text', { footnote: true }),
    () => toHtmlWithHighlighter('text', highlighter, { theme: 'dark' }, { taskList: true }),
    () => transformWithHighlighter('text', highlighter, { theme: 'dark' }, { footnote: true }),
  ]

  for (const call of calls) {
    assert.throws(
      call,
      error => error instanceof TypeError && /unknown option.*(?:taskList|footnote)/i.test(error.message),
    )
  }
})

test('selects the musl optional package on Alpine-style Linux', () => {
  const entry = new URL('../index.mjs', import.meta.url).href
  const script = `
    Object.defineProperty(process, 'platform', { value: 'linux' })
    Object.defineProperty(process, 'arch', { value: 'arm64' })
    Object.defineProperty(process, 'report', {
      value: { getReport: () => ({ header: {} }) },
    })
    const { toHtml } = await import(${JSON.stringify(entry)})
    try {
      toHtml('text')
    }
    catch (error) {
      if (error instanceof Error && error.message.includes('ferromark-linux-arm64-musl')) {
        process.exit(0)
      }
    }
    process.exit(1)
  `
  const result = spawnSync(process.execPath, ['--input-type=module', '--eval', script], {
    encoding: 'utf8',
  })

  assert.equal(result.status, 0, result.stderr)
})

test('maps every supported native platform and rejects unsupported targets', () => {
  const targets = [
    ['darwin', 'arm64', undefined, 'darwin-arm64'],
    ['darwin', 'x64', undefined, 'darwin-x64'],
    ['linux', 'arm64', '2.17', 'linux-arm64-gnu'],
    ['linux', 'arm64', undefined, 'linux-arm64-musl'],
    ['linux', 'x64', '2.39', 'linux-x64-gnu'],
    ['linux', 'x64', undefined, 'linux-x64-musl'],
    ['win32', 'arm64', undefined, 'win32-arm64-msvc'],
    ['win32', 'x64', undefined, 'win32-x64-msvc'],
  ]

  for (const [platform, arch, glibcVersionRuntime, expected] of targets) {
    assert.equal(nativeTarget(platform, arch, glibcVersionRuntime), expected)
  }
  assert.throws(
    () => nativeTarget('linux', 'riscv64', '2.39'),
    /ferromark does not support linux\/riscv64/,
  )
  assert.throws(
    () => nativeTarget('freebsd', 'x64'),
    /ferromark does not support freebsd\/x64/,
  )
})

test('does not collect a diagnostic report on non-Linux platforms', () => {
  const entry = new URL('../index.mjs', import.meta.url).href
  const script = `
    Object.defineProperty(process, 'platform', { value: 'darwin' })
    Object.defineProperty(process, 'arch', { value: 'arm64' })
    Object.defineProperty(process, 'report', {
      value: { getReport: () => { throw new Error('diagnostic report collected') } },
    })
    const { toHtml } = await import(${JSON.stringify(entry)})
    try {
      toHtml('text')
    }
    catch (error) {
      if (error instanceof Error && error.message === 'diagnostic report collected') {
        process.exit(1)
      }
    }
  `
  const result = spawnSync(process.execPath, ['--input-type=module', '--eval', script], {
    encoding: 'utf8',
  })

  assert.equal(result.status, 0, result.stderr)
})

test('wraps native dynamic-loader failures with platform guidance', async (t) => {
  const target = currentNativeTarget()
  const packageName = `ferromark-${target}`
  const binaryName = `ferromark.${target}.node`
  const fixture = await mkdtemp(path.join(tmpdir(), 'ferromark-loader-'))
  const packageDir = path.join(fixture, 'node_modules', packageName)
  const entry = path.join(fixture, 'index.mjs')
  t.after(() => rm(fixture, { force: true, recursive: true }))

  await mkdir(packageDir, { recursive: true })
  await Promise.all([
    copyFile(new URL('../index.mjs', import.meta.url), entry),
    copyFile(
      new URL('../native-target.mjs', import.meta.url),
      path.join(fixture, 'native-target.mjs'),
    ),
    writeFile(
      path.join(packageDir, 'package.json'),
      JSON.stringify({ name: packageName, main: binaryName }),
    ),
    writeFile(path.join(packageDir, binaryName), 'not a native addon'),
  ])

  const fixtureModule = await import(pathToFileURL(entry).href)
  assert.throws(
    () => fixtureModule.toHtml('text'),
    (error) => {
      assert.ok(error instanceof Error)
      assert.match(error.message, new RegExp(binaryName.replaceAll('.', '\\.')))
      assert.match(error.message, new RegExp(`${process.platform}/${process.arch}`))
      assert.match(error.message, /ERR_DLOPEN_FAILED/)
      assert.equal(error.cause?.code, 'ERR_DLOPEN_FAILED')
      if (process.platform === 'linux') {
        assert.match(error.message, /glibc 2\.17|musl runtime/)
      }
      else if (process.platform === 'win32') {
        assert.match(error.message, /Visual C\+\+ Redistributable/)
      }
      else {
        assert.match(error.message, /quarantine or code-signing policy/)
      }
      return true
    },
  )
})

test('loads the package by name from CommonJS', () => {
  const require = createRequire(import.meta.url)
  const { toHtml: cjsToHtml } = require('ferromark')

  assert.equal(cjsToHtml('# CommonJS'), '<h1 id="commonjs">CommonJS</h1>\n')
})

test('composes with a synchronous Ferriki-compatible highlighter', () => {
  const calls = []
  const highlighter = {
    codeToHtml(code, options) {
      calls.push({ code, options })
      return '<pre class="ferriki"><code>safe</code></pre>\n'
    },
  }

  const html = toHtmlWithHighlighter(
    '```rust\nconst x = 1\n```',
    highlighter,
    { theme: 'github-dark' },
  )

  assert.equal(html, '<pre class="ferriki"><code>safe</code></pre>\n')
  assert.deepEqual(calls, [{
    code: 'const x = 1\n',
    options: { lang: 'rust', theme: 'github-dark' },
  }])
})

test('falls back to escaped code when highlighting fails', () => {
  const failures = []
  const failure = new Error('unsupported language')
  const highlighter = {
    codeToHtml() {
      throw failure
    },
  }

  const html = toHtmlWithHighlighter(
    '```unknown\n<tag>\n```',
    highlighter,
    {
      theme: 'github-dark',
      onHighlightError(error, context) {
        failures.push({ error, context })
      },
    },
  )

  assert.equal(
    html,
    '<pre><code class="language-unknown">&lt;tag&gt;\n</code></pre>\n',
  )
  assert.deepEqual(failures, [{ error: failure, context: { lang: 'unknown' } }])
  assert.equal(
    toHtmlWithHighlighter(
      '```unknown\n<tag>\n```',
      highlighter,
      { theme: 'github-dark' },
    ),
    html,
  )
})

test('uses fallbackLanguage and preserves transform metadata when highlighting fails', () => {
  const calls = []
  const failure = new Error('unsupported language')
  const highlighter = {
    codeToHtml(code, options) {
      calls.push({ code, options })
      throw failure
    },
  }
  const failures = []
  const result = transformWithHighlighter(
    '```\n<tag>\n```\n\n# After',
    highlighter,
    {
      theme: 'github-dark',
      fallbackLanguage: 'plaintext',
      onHighlightError(error, context) {
        failures.push({ error, context })
      },
    },
  )

  assert.match(result.html, /<pre><code>&lt;tag&gt;\n<\/code><\/pre>/)
  assert.match(result.html, /<h1 id="after">After<\/h1>/)
  assert.deepEqual(result.headings, [{ level: 1, id: 'after', text: 'After' }])
  assert.deepEqual(calls, [{
    code: '<tag>\n',
    options: { lang: 'plaintext', theme: 'github-dark' },
  }])
  assert.deepEqual(failures, [{ error: failure, context: { lang: 'plaintext' } }])
})

test('validates and surfaces highlighter error observers', () => {
  const highlighter = {
    codeToHtml() {
      throw new Error('highlight failed')
    },
  }

  assert.throws(
    () => toHtmlWithHighlighter('```js\ncode\n```', highlighter, {
      theme: 'dark',
      onHighlightError: 'invalid',
    }),
    /onHighlightError must be a function/,
  )
  assert.throws(
    () => transformWithHighlighter('```js\ncode\n```', highlighter, {
      theme: 'dark',
      onHighlightError() {
        throw new Error('observer failed')
      },
    }),
    /observer failed/,
  )
})

test('surfaces invalid highlighter return values from the native callback', () => {
  const highlighter = {
    codeToHtml() {
      return { html: '<pre>wrong shape</pre>' }
    },
  }

  assert.throws(
    () => toHtmlWithHighlighter('```js\ncode\n```', highlighter, { theme: 'dark' }),
    error => error instanceof Error,
  )
})

test('transform returns html, headings, and front matter', () => {
  const result = transform('---\ntitle: X\n---\n# Top\n\n## Sub `code`\n', { frontMatter: true })

  assert.equal(result.frontMatter, 'title: X\n')
  assert.match(result.html, /<h1 id="top">Top<\/h1>/)
  assert.deepEqual(result.headings, [
    { level: 1, id: 'top', text: 'Top' },
    { level: 2, id: 'sub-code', text: 'Sub code' },
  ])
})

test('transform extracts TOML-style front matter', () => {
  const result = transform('+++\ntitle = "TOML"\n+++\n# Top', { frontMatter: true })

  assert.equal(result.frontMatter, 'title = "TOML"\n')
  assert.match(result.html, /<h1 id="top">Top<\/h1>/)
})

test('transform omits ids when headingIds is disabled', () => {
  const result = transform('# Top', { headingIds: false })

  assert.equal(result.headings.length, 1)
  assert.equal(result.headings[0].id, undefined)
  assert.equal(result.headings[0].text, 'Top')
})

test('linkBasePath prefixes internal links only', () => {
  const html = toHtml('[in](/guide) [out](https://e.com/) ![img](/i.png)', {
    linkBasePath: '/docs',
  })

  assert.match(html, /<a href="\/docs\/guide">/)
  assert.match(html, /<a href="https:\/\/e.com\/">/)
  assert.match(html, /<img src="\/i.png"/)
})

test('highlighter receives fence meta as Shiki-style __raw', () => {
  const calls = []
  const highlighter = {
    codeToHtml(code, options) {
      calls.push(options)
      return '<pre class="hl">x</pre>\n'
    },
  }

  const result = transformWithHighlighter(
    '```ts {1-3} title="Example"\ncode\n```\n\n```ts\ncode\n```',
    highlighter,
    { theme: 'github-dark' },
  )

  assert.match(result.html, /class="hl"/)
  assert.deepEqual(calls, [
    { lang: 'ts', theme: 'github-dark', meta: { __raw: '{1-3} title="Example"' } },
    { lang: 'ts', theme: 'github-dark' },
  ])
})

function currentNativeTarget() {
  const report = process.report?.getReport?.()
  const libc = report?.header?.glibcVersionRuntime ? 'gnu' : 'musl'
  const key = process.platform === 'linux'
    ? `${process.platform}-${process.arch}-${libc}`
    : `${process.platform}-${process.arch}`
  const targets = {
    'darwin-arm64': 'darwin-arm64',
    'darwin-x64': 'darwin-x64',
    'linux-arm64-gnu': 'linux-arm64-gnu',
    'linux-arm64-musl': 'linux-arm64-musl',
    'linux-x64-gnu': 'linux-x64-gnu',
    'linux-x64-musl': 'linux-x64-musl',
    'win32-arm64': 'win32-arm64-msvc',
    'win32-x64': 'win32-x64-msvc',
  }
  const target = targets[key]
  assert.ok(target, `test requires a supported native target, received ${key}`)
  return target
}
