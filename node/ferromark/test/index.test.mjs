import assert from 'node:assert/strict'
import test from 'node:test'

import { toHtml, toHtmlWithHighlighter, transform, transformWithHighlighter } from '../index.mjs'

test('renders Markdown through the native binding', () => {
  assert.equal(toHtml('# Hello'), '<h1 id="hello">Hello</h1>\n')
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

test('rejects unknown option keys across every public render entry point', () => {
  const highlighter = {
    codeToHtml() {
      return '<pre><code></code></pre>\n'
    },
  }
  const calls = [
    () => toHtml('text', { taskList: true }),
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
  const highlighter = {
    codeToHtml() {
      throw new Error('unsupported language')
    },
  }

  const html = toHtmlWithHighlighter(
    '```unknown\n<tag>\n```',
    highlighter,
    { theme: 'github-dark' },
  )

  assert.equal(
    html,
    '<pre><code class="language-unknown">&lt;tag&gt;\n</code></pre>\n',
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
