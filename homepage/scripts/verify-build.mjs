import { access, readFile } from "node:fs/promises"
import { join } from "node:path"

const outputDirectory = new URL("../build/client/", import.meta.url)
const expectedPages = [
  "index.html",
  "guide/benchmarks/index.html",
  "guide/features/index.html",
  "guide/getting-started/index.html",
  "guide/mdx-examples/index.html",
  "guide/quick-start/index.html",
]

await Promise.all(expectedPages.map((page) => access(new URL(page, outputDirectory))))

const homepage = await readFile(new URL("index.html", outputDirectory), "utf8")
const requiredFragments = [
  "/ferromark/assets/",
  "/ferromark/favicon.ico",
  'class="ferramenta-family"',
  'aria-current="true"',
]

for (const fragment of requiredFragments) {
  if (!homepage.includes(fragment)) {
    throw new Error(`Prerendered homepage is missing ${JSON.stringify(fragment)}`)
  }
}

if (/<p(?:\s[^>]*)?>\s*<nav\b/i.test(homepage)) {
  throw new Error("Prerendered homepage contains a nav nested directly inside a paragraph")
}

console.log(`Verified ${expectedPages.length} prerendered pages in ${join("build", "client")}`)
