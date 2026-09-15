import { Link } from "react-router";
import "../styles/home.css";

export default function HomePage() {
  return (
    <div className="landing">
      <section className="hero panel-mesh">
        <p className="eyebrow">Ferromark v2 · Development preview</p>
        <h1>Markdown, with room to build.</h1>
        <p className="lead">An arena-allocated Markdown parser and HTML renderer for Rust and Node.js. Inspect the syntax tree, retain source spans, and shape the output with render hooks.</p>
        <div className="hero-actions">
          <Link className="button button-primary" to="/guide/quick-start">Try the v2 workspace</Link>
          <Link className="button button-secondary" to="/guide/features">Explore the API</Link>
        </div>
        <p>V2 is under development. Packages are unpublished and the Rust API differs from v1.</p>
      </section>
      <section className="panel">
        <div className="section-head"><p className="eyebrow">A new foundation</p><h2>Parse, inspect, render.</h2></div>
        <div className="feature-grid">
          <article><h3>Markdown to AST</h3><p>CommonMark and GFM profiles, source spans, front matter, and configurable syntax extensions.</p></article>
          <article><h3>AST to HTML</h3><p>Reusable render buffers, per-node hooks, heading IDs, table layout, and explicit HTML output options.</p></article>
          <article><h3>Rust and Node.js</h3><p>Five focused Rust crates plus a native Node package with HTML, Buffer, metadata, and highlighter APIs.</p></article>
        </div>
      </section>
      <section className="panel panel-mesh">
        <div className="section-head"><p className="eyebrow">Start in Rust</p><h2>A document owns its shape.</h2></div>
        <pre className="code"><code>{`use ferromark::{Allocator, HtmlRenderer, Parser};

let source = "# Hello, v2";
let arena = Allocator::for_source_len(source.len());
let document = Parser::new(&arena, source).parse()?;
let html = HtmlRenderer::new().render(&document);`}</code></pre>
        <p>The arena and source outlive the parsed document. Visit the AST or render it directly.</p>
      </section>
      <section className="panel">
        <div className="section-head"><p className="eyebrow">Evidence and scope</p><h2>Measured work, explicit boundaries.</h2></div>
        <p>V2 builds on the MIT-licensed OX-Content core and selected Ferromark optimizations. Benchmarks retain their source revisions, build settings, and output checks.</p>
        <p>MDX support recognizes syntax and renders static island payloads. It does not compile or execute JavaScript.</p>
        <div className="hero-actions"><Link to="/guide/benchmarks">Read the benchmark evidence</Link><Link to="/guide/mdx-examples">Understand MDX support</Link></div>
      </section>
    </div>
  );
}
