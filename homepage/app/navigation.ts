export const documentationSections = [
  {
    id: "rust",
    label: "Rust",
    to: "/rust/getting-started",
    pages: [
      ["Start with Rust", "/rust/getting-started"],
      ["Configuration", "/rust/configuration"],
      ["Content pipelines", "/rust/pipelines"],
      ["AST transforms", "/rust/transforms"],
      ["Code rendering", "/rust/highlighting"],
      ["MDX integration", "/rust/mdx"],
      ["MDX examples", "/rust/mdx-examples"],
    ],
  },
  {
    id: "node",
    label: "Node.js",
    to: "/node/getting-started",
    pages: [
      ["Start with Node.js", "/node/getting-started"],
      ["Configuration", "/node/configuration"],
      ["Content pipelines", "/node/pipelines"],
      ["Syntax highlighting", "/node/highlighting"],
      ["Runtime and deployment", "/node/deployment"],
    ],
  },
  {
    id: "guide",
    label: "Shared reference",
    to: "/guide/getting-started",
    pages: [
      ["Documentation", "/guide/getting-started"],
      ["Choose a quick start", "/guide/quick-start"],
      ["Markdown configuration", "/guide/configuration"],
      ["Markdown syntax", "/guide/features"],
      ["Pipeline concepts", "/guide/pipelines"],
      ["Rendering and trust", "/guide/rendering"],
      ["MDX boundaries", "/guide/mdx"],
      ["Command line", "/guide/cli"],
      ["Architecture", "/guide/architecture"],
      ["Correctness", "/guide/correctness"],
      ["Feature comparison", "/guide/feature-comparison"],
      ["Parser benchmarks", "/guide/benchmarks"],
      ["Runtime profiles", "/guide/workflow-benchmarks"],
      ["Explore reports", "/guide/benchmark-explorer"],
    ],
  },
] as const;

export const sharedConcepts = [
  ["Markdown syntax", "/guide/features"],
  ["Rendering and trust", "/guide/rendering"],
  ["Pipeline concepts", "/guide/pipelines"],
  ["Benchmarks", "/guide/benchmarks"],
] as const;
