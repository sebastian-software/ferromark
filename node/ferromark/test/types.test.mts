import type { Buffer } from 'node:buffer'
import type { CodeHighlighter, Options } from '../index.mjs'
import { Renderer, toHtml, toHtmlBuffer, toHtmlWithHighlighter } from '../index.mjs'

const options: Options = {
  renderPolicy: 'untrusted',
  tables: true,
  mergedTableCells: true,
  tableColumnWidths: true,
  inlineFootnotes: true,
  definitionLists: true,
  lineComments: true,
  indentedCodeBlocks: false,
}
const highlighter: CodeHighlighter = {
  codeToHtml: (code, { lang, theme }) => `${lang}:${theme}:${code}`,
}

toHtml('# Typed', options)
const output: Buffer = toHtmlBuffer('# Buffered', options)
output.toString('utf8')
const renderer = new Renderer(options)
renderer.toHtml('# Reused')
const reusedOutput: Buffer = renderer.toHtmlBuffer('# Buffered and reused')
reusedOutput.toString('utf8')
toHtmlWithHighlighter('```ts\nconst typed = true\n```', highlighter, {
  theme: 'github-dark',
  onHighlightError(error, { lang }) {
    void error
    lang.toUpperCase()
  },
})
