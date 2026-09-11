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
const guidePage = await readFile(new URL("guide/quick-start/index.html", outputDirectory), "utf8")

const requiredFragments = [
  "/ferromark/assets/",
  "/ferromark/favicon.ico",
  // The shared chrome from ferramenta-family: the header carries the tool
  // switcher and the footer carry sibling links, excluding this site.
  'class="site-header"',
  'class="site-footer"',
  "https://ferramenta.dev",
  "652/652 CommonMark spec tests pass in trusted mode",
  "CommonMark tests passed in trusted mode",
  "Fine-grained",
  "Parsing and rendering controls for precise output",
]

// The family chrome replaces Ardo's own header and footer (`handle.chrome` in
// app/root.tsx). If either comes back, the page carries two of each.
const forbiddenFragments = ['class="ardo-header', 'class="ardo-footer']

// Every page carries the family chrome, and the guide keeps its navigation:
// the sidebar rail, plus the header menu that stands in for it once the rail is
// hidden — without that menu a narrow viewport has no way into the guide.
const requiredGuideFragments = [
  'class="site-header"',
  'class="site-footer"',
  'class="ardo-sidebar',
  'class="ferromark-guide-menu"',
]

function check(page, label, required, forbidden = []) {
  for (const fragment of required) {
    if (!page.includes(fragment)) {
      throw new Error(`Prerendered ${label} is missing ${JSON.stringify(fragment)}`)
    }
  }
  for (const fragment of forbidden) {
    if (page.includes(fragment)) {
      throw new Error(`Prerendered ${label} still contains ${JSON.stringify(fragment)}`)
    }
  }
}

const footer = homepage.match(/<footer\b[\s\S]*?<\/footer>/)?.[0] ?? ""
check(footer, "family footer", ["ferramenta.dev", "ferriki"], ["https://sebastian-software.github.io/ferromark/"])

check(homepage, "homepage", requiredFragments, forbiddenFragments)
check(guidePage, "guide page", requiredGuideFragments, forbiddenFragments)

if (/<p(?:\s[^>]*)?>\s*<nav\b/i.test(homepage)) {
  throw new Error("Prerendered homepage contains a nav nested directly inside a paragraph")
}

console.log(`Verified ${expectedPages.length} prerendered pages in ${join("build", "client")}`)
