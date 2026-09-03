import type { CodeHighlighter, Options } from '../index.mjs'
import { Renderer, toHtml, toHtmlWithHighlighter } from '../index.mjs'

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
new Renderer(options).toHtml('# Reused')
toHtmlWithHighlighter('```ts\nconst typed = true\n```', highlighter, {
  theme: 'github-dark',
})
