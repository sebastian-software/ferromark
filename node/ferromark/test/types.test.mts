import type { CodeHighlighter, Options } from '../index.mjs'
import { toHtml, toHtmlWithHighlighter } from '../index.mjs'

const options: Options = {
  renderPolicy: 'untrusted',
  tables: true,
  lineComments: true,
}
const highlighter: CodeHighlighter = {
  codeToHtml: (code, { lang, theme }) => `${lang}:${theme}:${code}`,
}

toHtml('# Typed', options)
toHtmlWithHighlighter('```ts\nconst typed = true\n```', highlighter, {
  theme: 'github-dark',
})
