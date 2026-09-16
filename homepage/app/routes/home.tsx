import { Link } from "react-router";

import { PlatformChoices } from "../components/platform-choices";
import "../styles/home.css";

function LandingIntro() {
  return (
    <section className="landing-intro" aria-labelledby="landing-title">
      <p className="landing-kicker">Ferromark · The Ferramenta Markdown engine</p>
      <h1 id="landing-title">
        Markdown.
        <br />
        <span>Ready for your pipeline.</span>
      </h1>
      <p className="landing-lead">
        Turn Markdown into HTML, with the metadata your application needs. A fast, focused Rust
        engine with CommonMark correctness, explicit rendering policies, and practical publishing
        features.
      </p>
      <p className="landing-aside">
        <strong>Release candidate · 2.0.0-rc.1.</strong> V2 is in release-candidate testing.{" "}
        <a href="https://github.com/sebastian-software/ferromark/blob/main/docs/migration-v2.md">
          Read the migration guide
        </a>
        .
      </p>
      <PlatformChoices />
      <p className="landing-aside">
        Working in a shell? <Link to="/guide/cli">Use the command line</Link>.
      </p>
    </section>
  );
}

function PrincipleList() {
  return (
    <div className="landing-principle-list">
      <article>
        <span className="landing-index" aria-hidden="true">
          01
        </span>
        <div>
          <h3>A core with a clear job</h3>
          <p>
            Parse Markdown. Render HTML. Keep translation and code presentation at explicit
            integration points. Arena allocation and reusable buffers keep the engine focused.
          </p>
          <Link to="/guide/architecture">Understand the design →</Link>
        </div>
      </article>
      <article>
        <span className="landing-index" aria-hidden="true">
          02
        </span>
        <div>
          <h3>Correctness you can inspect</h3>
          <p>
            The explicit CommonMark profile agrees with all 652 specification examples. Output
            comparisons with cmark and cmark-gfm expose edge cases and keep fixes reproducible.
          </p>
          <Link to="/guide/correctness">Inspect correctness checks →</Link>
        </div>
      </article>
      <article>
        <span className="landing-index" aria-hidden="true">
          03
        </span>
        <div>
          <h3>The pieces publishing needs</h3>
          <p>
            Front matter, headings for navigation, footnotes, callouts, table classes, and merged
            cells. Enable the syntax your authors need and get the output your application can use.
          </p>
          <Link to="/guide/features">Explore Markdown syntax →</Link>
        </div>
      </article>
    </div>
  );
}

function LandingPrinciples() {
  return (
    <section className="landing-principles" aria-labelledby="principles-title">
      <div className="landing-section-head">
        <p className="landing-kicker">Why Ferromark</p>
        <h2 id="principles-title">Built for real publishing work.</h2>
        <p>
          Render articles, extract front matter, and build navigation from headings. Keep control
          over syntax, trust, and how code blocks are presented.
        </p>
      </div>
      <PrincipleList />
    </section>
  );
}

function LandingReference() {
  return (
    <section className="landing-reference" aria-labelledby="reference-title">
      <p className="landing-kicker">One engine. A shared contract.</p>
      <h2 id="reference-title">Learn the behavior once.</h2>
      <p>
        Rust and Node.js have their own guides and APIs. Markdown syntax, trust policies, and output
        semantics live in one shared reference.
      </p>
      <div className="landing-reference-links">
        <Link to="/guide/configuration">
          <strong>Choose your syntax</strong>
          <span>Defaults, dialects, and optional extensions →</span>
        </Link>
        <Link to="/guide/pipelines">
          <strong>Plan your pipeline</strong>
          <span>Metadata, navigation, links, and presentation →</span>
        </Link>
        <Link to="/guide/benchmarks">
          <strong>Inspect the evidence</strong>
          <span>Measured workloads, trade-offs, and reproduction →</span>
        </Link>
      </div>
    </section>
  );
}

export default function HomePage() {
  return (
    <main className="landing">
      <LandingIntro />
      <LandingPrinciples />
      <LandingReference />
    </main>
  );
}
