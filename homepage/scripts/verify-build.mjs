/* eslint-disable security/detect-non-literal-fs-filename -- Paths come from the fixed route table and locally generated site output. */
import { access, readFile } from "node:fs/promises";
import { join } from "node:path";

const outputDirectory = new URL("../build/client/", import.meta.url);
const expectedPages = [
  "index.html",
  "guide/architecture/index.html",
  "guide/benchmark-explorer/index.html",
  "guide/benchmarks/index.html",
  "guide/cli/index.html",
  "guide/configuration/index.html",
  "guide/correctness/index.html",
  "guide/feature-comparison/index.html",
  "guide/features/index.html",
  "guide/getting-started/index.html",
  "guide/mdx-examples/index.html",
  "guide/mdx/index.html",
  "guide/pipelines/index.html",
  "guide/quick-start/index.html",
  "guide/rendering/index.html",
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
const guidePage = await readFile(new URL("guide/quick-start/index.html", outputDirectory), "utf8");

// The documented version has one source: the npm facade manifest the release
// pull request updates. `app/version.ts` reads it, so a release must reach the
// rendered pages without any homepage edit. See docs/releasing.md.
const { version } = JSON.parse(
  await readFile(new URL("../../node/ferromark/package.json", import.meta.url), "utf8"),
);

// The landing page states the measured figures that
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
const platformFragments = benchmarks.platforms.flatMap((platform) => {
  return [
    ...platform.figures.flatMap((figure) => {
      if (figure.id === "v1") return [];
      return [libraryLabels.get(figure.id) ?? figure.label, `${figure.fresh.toFixed(1)}×`];
    }),
    platform.label,
    ...platform.machine.split(", "),
    platform.revision,
  ];
});

const ecosystemBenchmarks = JSON.parse(
  await readFile(
    new URL("../app/data/markdown-ecosystem-benchmarks.json", import.meta.url),
    "utf8",
  ),
);
const ecosystemFragments = ecosystemBenchmarks.figures.flatMap((figure) => [
  libraryLabels.get(figure.id),
  figure.runtime,
  `${figure.fresh.toFixed(1)}×`,
  `${figure.documents}/${figure.corpusDocuments} documents`,
  figure.revision,
]);

const completedBenchmarks = JSON.parse(
  await readFile(new URL("../app/data/benchmark-platform-values.json", import.meta.url), "utf8"),
);
const completedFragments = completedBenchmarks.figures.flatMap((figure) => [
  `${figure.fresh.toFixed(1)}×`,
  `${figure.documents}/${figure.corpusDocuments} documents`,
  figure.revision,
  `href="https://github.com/sebastian-software/ferromark/tree/main/${figure.report}"`,
]);

const requiredFragments = [
  ...completedFragments,
  ...ecosystemFragments,
  ...projects.flatMap((project) => [project.label, `href="${project.github}"`]),
  'id="markdown-ecosystem"',
  "Ferromark speedup over each library",
  "2× means twice the throughput",
  'class="ferromark-comparison"',
  'id="comparison-native"',
  'id="comparison-node"',
  ...platformFragments,
  'href="/guide/benchmarks"',
  '"/assets/',
  '"/favicon.ico"',
  'class="site-header"',
  'class="site-footer"',
  "https://ferramenta.dev",
  version,
  "Start with Rust",
  "Start with Node.js",
];

// The family chrome replaces Ardo's own header and footer (`handle.chrome` in
// app/root.tsx). If either comes back, the page carries two of each.
const forbiddenFragments = ['class="ardo-header', 'class="ardo-footer'];

// Every page carries the family chrome, and the guide keeps its navigation:
// the sidebar rail, plus the header menu that stands in for it once the rail is
// hidden — without that menu a narrow viewport has no way into the guide.
const requiredGuideFragments = [
  'class="site-header"',
  'class="site-footer"',
  'class="ardo-sidebar',
  'class="ferromark-guide-menu"',
];

function check(page, label, { required, forbidden = [] }) {
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

const footer = homepage.match(/<footer\b[\s\S]*?<\/footer>/)?.[0] ?? "";
check(footer, "family footer", {
  required: ["ferramenta.dev", "ferriki"],
  forbidden: ["https://sebastian-software.github.io/ferromark/"],
});

check(homepage, "homepage", {
  required: requiredFragments,
  forbidden: [...forbiddenFragments, "Ferromark v1", "OX-Content original"],
});
check(benchmarkPage, "v2 benchmark evidence", {
  required: [
    "v2",
    "source revisions",
    ...benchmarks.platforms.flatMap((platform) => [
      platform.label,
      platform.report,
      platform.revision,
    ]),
  ],
});
check(guidePage, "guide page", {
  required: requiredGuideFragments,
  forbidden: forbiddenFragments,
});

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
