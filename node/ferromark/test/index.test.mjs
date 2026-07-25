import assert from 'node:assert/strict'
import test from 'node:test'

import { toHtml, toHtmlWithHighlighter } from '../index.mjs'

test('renders Markdown through the native binding', () => {
  assert.equal(toHtml('# Hello'), '<h1 id="hello">Hello</h1>\n')
})

test('maps typed options to the Rust surface', () => {
  assert.equal(toHtml('==mark==', { highlight: true }), '<p><mark>mark</mark></p>\n')
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
