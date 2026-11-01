/* eslint-disable security/detect-non-literal-fs-filename -- Paths come from the fixed route table and locally generated site output. */
import { access, readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

// Product sites consume shared presentation rather than maintaining a parallel CSS system.
const appFiles = await readdir(new URL("../app/", import.meta.url), { recursive: true });
if (appFiles.some((file) => file.endsWith(".css"))) {
  throw new Error("Keep reusable site styles in ferramenta-family, not in homepage/app");
}

const outputDirectory = new URL("../build/client/", import.meta.url);
const expectedPages = [
  "index.html",
  "guide/architecture/index.html",
  "guide/benchmark-explorer/index.html",
  "guide/benchmarks/index.html",
  "guide/cli/index.html",
  "guide/code-rendering/index.html",
  "guide/configuration/index.html",
  "guide/correctness/index.html",
  "guide/document-analysis/index.html",
  "guide/feature-comparison/index.html",
  "guide/features/index.html",
  "guide/ffm/index.html",
  "guide/getting-started/index.html",
  "guide/mdx-examples/index.html",
  "guide/mdx/index.html",
  "guide/pipelines/index.html",
  "guide/quick-start/index.html",
  "guide/rendering/index.html",
  "guide/typography/index.html",
  "guide/workflow-benchmarks/index.html",
  "node/configuration/index.html",
  "node/deployment/index.html",
  "node/getting-started/index.html",
  "node/highlighting/index.html",
  "node/pipelines/index.html",
  "rust/configuration/index.html",
  "rust/getting-started/index.html",
  "rust/highlighting/index.html",
  "rust/mdx-examples/index.html",
  "rust/mdx/index.html",
  "rust/pipelines/index.html",
  "rust/transforms/index.html",
];

await Promise.all(expectedPages.map((page) => access(new URL(page, outputDirectory))));

const homepage = await readFile(new URL("index.html", outputDirectory), "utf8");
const benchmarkPage = await readFile(
  new URL("guide/benchmarks/index.html", outputDirectory),
  "utf8",
);
const correctnessPage = await readFile(
  new URL("guide/correctness/index.html", outputDirectory),
  "utf8",
);
const guidePage = await readFile(new URL("guide/quick-start/index.html", outputDirectory), "utf8");
const analysisPage = await readFile(
  new URL("guide/document-analysis/index.html", outputDirectory),
  "utf8",
);

// The documented version has one source: the npm facade manifest the release
// pull request updates. The build snapshots it for `app/version.ts`, so a release must reach the
// rendered pages without any homepage edit. See docs/releasing.md.
const { version } = JSON.parse(
  await readFile(new URL("../../node/ferromark/package.json", import.meta.url), "utf8"),
);

// The benchmark guide states the measured figures that
// scripts/publish-native-readme.py derives from one archived native comparison
// per platform; a hard-coded number would survive the next measurement, so the
// prerendered page must carry every platform's library figures, machine and
// measured revision.
const benchmarks = JSON.parse(
  await readFile(new URL("../app/data/native-benchmarks.json", import.meta.url), "utf8"),
);
if (benchmarks.platforms.length < 2) {
  throw new Error("native-benchmarks.json must publish every measured platform");
}
const projects = JSON.parse(
  await readFile(new URL("../app/data/benchmark-projects.json", import.meta.url), "utf8"),
);
const libraryLabels = new Map(projects.map((project) => [project.id, project.label]));
const ecosystemBenchmarks = JSON.parse(
  await readFile(
    new URL("../app/data/markdown-ecosystem-benchmarks.json", import.meta.url),
    "utf8",
  ),
);
const completedBenchmarks = JSON.parse(
  await readFile(new URL("../app/data/benchmark-platform-values.json", import.meta.url), "utf8"),
);
const completedKeys = new Set(
  completedBenchmarks.figures.map((figure) => `${figure.platform}/${figure.id}`),
);
// Historical values remain in the guide; require the currently selected values
// in the benchmark guide when a new manual report replaces a platform's cells.
const completedPlatforms = new Map(
  completedBenchmarks.figures.map((figure) => [figure.platform, figure.platformLabel]),
);
const platformFragments = benchmarks.platforms.flatMap((platform) => [
  completedPlatforms.get(platform.id) ?? platform.label,
  ...platform.figures.flatMap((figure) => {
    if (figure.id === "v1" || completedKeys.has(`${platform.id}/${figure.id}`)) return [];
    return [
      libraryLabels.get(figure.id) ?? figure.label,
      `${figure.fresh.toFixed(1)}×`,
      ...platform.machine.split(", "),
      platform.revision,
    ];
  }),
]);
const ecosystemFragments = ecosystemBenchmarks.figures.flatMap((figure) =>
  completedKeys.has(`macos-arm64/${figure.id}`)
    ? []
    : [
        libraryLabels.get(figure.id),
        figure.runtime,
        `${figure.fresh.toFixed(1)}×`,
        `${figure.documents}/${figure.corpusDocuments} documents`,
        figure.revision,
      ],
);
const completedFragments = completedBenchmarks.figures.flatMap((figure) => [
  figure.platformLabel,
  `${figure.fresh.toFixed(1)}×`,
  `${figure.documents}/${figure.corpusDocuments} ${figure.scoringScope === "all-documents" ? "timed documents" : "documents"}`,
  figure.machine,
  figure.revision,
  `href="https://github.com/sebastian-software/ferromark/tree/main/${figure.overviewReport ?? figure.report}"`,
]);

const requiredComparisonFragments = [
  ...completedFragments,
  ...ecosystemFragments,
  ...projects.flatMap((project) => [project.label, `href="${project.github}"`]),
  'id="markdown-ecosystem"',
  "Ferromark speedup over each library",
  "2× means twice the throughput",
  'class="ferra-table ferra-benchmark-table"',
  ...[...completedPlatforms.keys()].flatMap((platform) => [
    `id="comparison-${platform}-native"`,
    `id="comparison-${platform}-node"`,
  ]),
  ...platformFragments,
];

const requiredFragments = [
  'id="markdown-ecosystem"',
  ...["pulldown-cmark", "marked"].map((id) => {
    const values = completedBenchmarks.figures
      .filter((figure) => figure.id === id)
      .map((figure) => figure.fresh);
    const low = Math.min(...values).toFixed(1);
    const high = Math.max(...values).toFixed(1);
    return low === high ? `${low}×` : `${low}–${high}×`;
  }),
  'href="/guide/benchmarks#markdown-ecosystem"',
  '"/assets/',
  'rel="icon"',
  'class="ferra-header ferra-wrap"',
  'class="ferra-footer ferra-wrap"',
  "https://ferramenta.dev",
  version,
  "Start with Rust",
  "Start with Node.js",
  'id="document-analysis"',
  'id="work"',
  'id="ffm-title"',
  'id="evidence-title"',
  'href="/guide/document-analysis"',
  "Reference inventories",
  "Text and search sections",
  "Static Markdown export",
];

// app/root.tsx passes the family header to Ardo and disables its default footer.
// If either default comes back, the page carries two of each.
const forbiddenFragments = ['class="ardo-header', 'class="ardo-footer'];

// Every page carries the family chrome, and the guide keeps its navigation:
// the sidebar rail, plus the header menu that stands in for it once the rail is
// hidden — without that menu a narrow viewport has no way into the guide.
const requiredGuideFragments = [
  'class="ferra-header ferra-wrap"',
  'class="ferra-footer ferra-wrap"',
  'class="ferra-guide-navigation"',
  'aria-label="Toggle menu"',
];

function check(page, label, { required, forbidden = [] }) {
  // Ardo 6 emits canonical trailing slashes; both spellings address the same static page.
  page = page.replaceAll(/href="(\/[^"?#]*?)\/(?=[?#"])/g, 'href="$1');
  for (const fragment of required) {
    if (!page.includes(fragment)) {
      throw new Error(`Prerendered ${label} is missing ${JSON.stringify(fragment)}`);
    }
  }
  for (const fragment of forbidden) {
    if (page.includes(fragment)) {
      throw new Error(`Prerendered ${label} still contains ${JSON.stringify(fragment)}`);
    }
  }
}

const footer = homepage.match(/<footer class="ferra-footer[\s\S]*?<\/footer>/)?.[0] ?? "";
check(footer, "family footer", {
  required: ["ferramenta.dev", "Sebastian Software"],
  forbidden: ["https://sebastian-software.github.io/ferromark/"],
});

check(homepage, "homepage", {
  required: requiredFragments,
  forbidden: [
    ...forbiddenFragments,
    "Ferromark v1",
    "OX-Content original",
    "ferra-benchmark-table",
    'class="ferra-install"',
  ],
});
check(benchmarkPage, "v2 benchmark evidence", {
  required: [
    ...requiredComparisonFragments,
    "v2",
    "source revisions",
    ...benchmarks.platforms.flatMap((platform) => [
      platform.label,
      platform.report,
      platform.revision,
    ]),
  ],
});
check(correctnessPage, "conformance comparison", {
  required: [
    "Specification agreement · public spec settings",
    'id="conformance-native"',
    'id="conformance-node"',
    "28 extension examples, not full GFM",
  ],
  forbidden: ["times the throughput"],
});
check(guidePage, "guide page", {
  required: requiredGuideFragments,
  forbidden: forbiddenFragments,
});
check(analysisPage, "document analysis guide", {
  required: [
    ...requiredGuideFragments,
    "collectReferences",
    "collect_references",
    "extractText",
    "extract_text",
    "projectMarkdown",
    "project_markdown",
    "trackReferences",
    "All analysis ranges count UTF-8 bytes.",
    'href="https://github.com/sebastian-software/ferromark/blob/main/docs/references.md"',
    'href="https://github.com/sebastian-software/ferromark/blob/main/docs/text-extraction.md"',
    'href="https://github.com/sebastian-software/ferromark/blob/main/docs/markdown-projection.md"',
  ],
  forbidden: forbiddenFragments,
});

for (const path of [
  "guide/getting-started/index.html",
  "node/pipelines/index.html",
  "rust/pipelines/index.html",
]) {
  const html = await readFile(new URL(path, outputDirectory), "utf8");
  check(html, path, { required: ['href="/guide/document-analysis"'] });
}

// Preserve published Ardo 4 section URLs where Ardo 6 changed the heading slug.
for (const [path, anchor] of [
  ["guide/feature-comparison", "nodejs-projects"],
  ["guide/document-analysis", "prepare-once-in-nodejs"],
  ["guide/mdx-examples", "tocomponent"],
  ["guide/benchmarks", "nodejs-ecosystem-coverage"],
  ["guide/typography", "nodejs"],
  ["rust/mdx-examples", "tocomponent"],
]) {
  const html = await readFile(new URL(`${path}/index.html`, outputDirectory), "utf8");
  check(html, path, { required: [`id="${anchor}"`] });
}

// Ardo 6 emits the complete static index as an ESM asset.
const assets = await readdir(new URL("assets/", outputDirectory));
const searchAsset = assets.find((file) => /^search-index-.*\.js$/.test(file));
if (!searchAsset) throw new Error("Prerendered site is missing its search index");
const { default: searchRecords } = await import(new URL(`assets/${searchAsset}`, outputDirectory));
const analysisRecords = searchRecords.filter(
  (record) => record.path.split("#")[0] === "/guide/document-analysis",
);
for (const method of ["collectReferences", "extractText", "projectMarkdown"]) {
  if (!analysisRecords.some((record) => record.content.includes(method))) {
    throw new Error(`Document analysis search records are missing ${method}`);
  }
}

// eslint-disable-next-line security/detect-unsafe-regex -- This scans local build output, not externally supplied HTML.
if (/<p(?:\s[^>]*)?>\s*<nav\b/i.test(homepage)) {
  throw new Error("Prerendered homepage contains a nav nested directly inside a paragraph");
}

// Removing the published version must leave no candidate version behind: any
// `-rc.` that survives was written into a page by hand instead of read from
// `node/ferromark/package.json`.
if (homepage.replaceAll(version, "").includes("-rc.")) {
  throw new Error("Prerendered homepage contains a hard-coded release-candidate version");
}

for (const path of expectedPages) {
  const html = await readFile(new URL(path, outputDirectory), "utf8");
  check(html, path, {
    required: ['href="/rust/getting-started"', 'href="/node/getting-started"'],
    forbidden: ['href="/ferromark', "sebastian-software.github.io/ferromark"],
  });
  if ((html.match(/<h1(?:\s|>)/g) ?? []).length !== 1) {
    throw new Error(`${path} must have one h1`);
  }
  // eslint-disable-next-line security/detect-unsafe-regex -- This scans local build output, not externally supplied HTML.
  for (const [, href] of html.matchAll(/href="([^"#]*)(?:#[^"]*)?"/g)) {
    if (!href.startsWith("/") || href.startsWith("//") || href.startsWith("/assets/")) continue;
    const pathname = href.slice(1).split("?")[0].replace(/\/$/, "");
    if (/\.[a-z0-9]+$/i.test(pathname)) continue;
    const target = pathname ? `${pathname}/index.html` : "index.html";
    await access(new URL(target, outputDirectory));
  }
}

console.log(`Verified ${expectedPages.length} prerendered pages in ${join("build", "client")}`);
