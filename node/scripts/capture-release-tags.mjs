import { readFileSync, writeFileSync } from 'node:fs'
const pkg = JSON.parse(readFileSync(new URL('../ferromark/package.json', import.meta.url)))
const tags = {}
for (const name of [...Object.keys(pkg.optionalDependencies), pkg.name]) {
  const response = await fetch(`https://registry.npmjs.org/${encodeURIComponent(name)}`, { signal: AbortSignal.timeout(30_000) })
  if (!response.ok && response.status !== 404) throw new Error(`${name}: tag preflight returned ${response.status}`)
  if (response.ok) tags[name] = (await response.json())['dist-tags']?.latest
}
writeFileSync(process.argv[2], JSON.stringify(tags, null, 2) + '\n')
