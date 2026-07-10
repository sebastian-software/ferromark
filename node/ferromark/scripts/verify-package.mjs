import { access, readFile } from 'node:fs/promises'

const required = ['index.mjs', 'index.d.mts', 'native.d.ts', 'package.json']
await Promise.all(required.map(file => access(new URL(`../${file}`, import.meta.url))))

const declarations = await readFile(new URL('../native.d.ts', import.meta.url), 'utf8')
for (const name of ['Options', 'toHtml', 'toHtmlWithRenderer']) {
  if (!declarations.includes(name)) {
    throw new Error(`Generated native declarations are missing ${name}`)
  }
}
