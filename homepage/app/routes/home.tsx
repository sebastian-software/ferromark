import { Link } from "react-router";
import { BenchmarkTable, benchmarkEnvironment, headlineThroughput } from "../components/benchmarks";
import "../styles/home.css";

export default function HomePage() {
  const featureColumns = [
    {
      heading: "CommonMark + GFM",
      items: [
        "652/652 CommonMark spec tests pass in trusted mode",
        "Tables, strikethrough, task lists",
        "Autolink literals + disallowed raw HTML",
      ],
    },
    {
      heading: "Beyond GFM",
      items: [
        "Footnotes and front matter extraction",
        "GitHub-compatible heading IDs",
        "Math spans ($ / $$) and callouts",
        "Highlight, superscript, subscript",
      ],
    },
    {
      heading: "MDX Workflow",
      items: [
        "Render full MDX without JS toolchain",
        "Export ready-to-use JSX component modules",
        "Segment-level control for custom pipelines",
      ],
    },
  ];

  const mdxExamples = [
    {
      title: "render()",
      text: "One call for HTML body, extracted ESM imports, and front matter metadata.",
      link: "/guide/mdx-examples#render",
    },
    {
      title: "to_component()",
      text: "Generate production-ready JSX/TSX modules for React, Preact, Solid, and more.",
      link: "/guide/mdx-examples#component",
    },
    {
      title: "segment()",
      text: "Inspect and route each MDX block when you need low-level pipeline control.",
      link: "/guide/mdx-examples#segment",
    },
  ];

  return (
    <main className="landing">
      <section className="hero">
        <p className="eyebrow">Rust Markdown Engine</p>
        <h1>Markdown to HTML at {headlineThroughput}</h1>
        <p className="lead">
          Measured on a mixed Markdown document on Apple Silicon, with the benchmark conditions
          documented below. Ferromark passes the complete CommonMark suite in trusted rendering
          mode.
        </p>
        <div className="hero-actions">
          <Link className="button button-primary" to="/guide/quick-start">
            Start Quick
          </Link>
          <Link className="button button-secondary" to="/guide/benchmarks">
            See Benchmarks
          </Link>
          <a className="button button-ghost" href="https://github.com/sebastian-software/ferromark">
            Open GitHub
          </a>
        </div>
        <div className="hero-stats">
          <article>
            <strong>652/652</strong>
            <span>CommonMark tests passed in trusted mode</span>
          </article>
          <article>
            <strong>Fine-grained</strong>
            <span>Parsing and rendering controls for precise output</span>
          </article>
          <article>
            <strong>MDX</strong>
            <span>Segmented and rendered without a JavaScript toolchain</span>
          </article>
        </div>
      </section>

      <section className="panel">
        <div className="section-head">
          <p className="eyebrow">Proof</p>
          <h2>Benchmark numbers you can verify</h2>
          <p>
            {benchmarkEnvironment}. Non-PGO binaries with tables, strikethrough, and task lists
            enabled. Rendering policies and output-buffer reuse differ between parsers; these
            historical results are not an equal-work ranking and have not been re-measured on
            x86-64. See the <Link to="/guide/benchmarks">benchmark guide</Link> for the conditions,
            native Bun comparison, and individual feature costs.
          </p>
        </div>
        <BenchmarkTable id="commonmark-50k" variant="panel" />
      </section>

      <section className="panel">
        <div className="section-head">
          <p className="eyebrow">Feature Surface</p>
          <h2>Everything needed for modern docs pipelines</h2>
        </div>
        <div className="feature-grid">
          {featureColumns.map((column) => (
            <article key={column.heading}>
              <h3>{column.heading}</h3>
              <ul>
                {column.items.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            </article>
          ))}
        </div>
      </section>

      <section className="panel panel-mesh">
        <div className="section-head">
          <p className="eyebrow">MDX Included</p>
          <h2>Use MDX without a JavaScript parsing toolchain</h2>
          <p>
            Opt into the <code>mdx</code> feature and choose your control level from high-level
            render output down to per-segment handling.
          </p>
        </div>
        <div className="mdx-cards">
          {mdxExamples.map((example) => (
            <article key={example.title}>
              <h3>{example.title}</h3>
              <p>{example.text}</p>
              <Link to={example.link}>View code path</Link>
            </article>
          ))}
        </div>
        <pre className="code">
          <code>{`let mut buffer = Vec::new();
ferromark::to_html_into("# Reuse me", &mut buffer);
// reuse buffer across calls to avoid repeated allocation`}</code>
        </pre>
      </section>

      <section className="panel">
        <div className="section-head">
          <p className="eyebrow">Trade-offs</p>
          <h2>Deliberately optimized for throughput</h2>
        </div>
        <ul className="tradeoffs">
          <li>No AST traversal API</li>
          <li>No source map position tracking</li>
          <li>HTML output focus only</li>
        </ul>
      </section>

      <section className="final-cta">
        <h2>Ship faster Markdown pipelines with less overhead</h2>
        <p>
          Start with quick docs, inspect the benchmark setup, then plug ferromark into your
          production workload.
        </p>
        <div className="hero-actions">
          <Link className="button button-primary" to="/guide/getting-started">
            Read Getting Started
          </Link>
          <Link className="button button-secondary" to="/guide/features">
            Explore Features
          </Link>
        </div>
      </section>
    </main>
  );
}
