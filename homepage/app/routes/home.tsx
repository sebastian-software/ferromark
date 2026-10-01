import {
  ClosingAction,
  IronBand,
  Ledger,
  Mark,
  PipelineAssembly,
  ProjectHero,
  RegistryFacts,
  RunSample,
  Section,
} from "ferramenta-family";
import { Link } from "react-router";

import {
  BenchmarkComparison,
  benchmarkHosts,
  benchmarkLead,
} from "../components/benchmark-comparison";
import { FlavoredMarkdownSection } from "../components/ffm-showcase";
import landingSample from "../data/landing-sample.json";
import { registrySnapshot, registrySnapshotGeneratedAt } from "../data/registry-snapshot";
import { version } from "../version";

const conformanceEntries = [
  {
    name: "CommonMark 0.31.2",
    status: "652 / 652",
    settled: true,
    detail: "Every specification example agrees with the CommonMark reference.",
  },
  {
    name: "GFM extensions",
    status: "Covered",
    settled: true,
    detail:
      "The official GitHub Flavored Markdown extension examples run in the conformance suite.",
  },
  {
    name: "CR and CRLF input",
    status: "1,304 variants",
    settled: true,
    detail: "Line-ending variants are checked against the same expected output.",
  },
  {
    name: "Original cmark corpus",
    status: "Covered",
    settled: true,
    detail: "The upstream comparison corpus remains in the regression suite.",
  },
];

function ProjectIntro() {
  return (
    <ProjectHero
      mark="ferromark"
      title={
        <>
          Markdown. <em>At native speed.</em>
        </>
      }
      lede="A focused native Markdown engine for Rust and Node.js. Parse Markdown, render HTML, and inspect the metadata publishing needs, with behavior grounded in CommonMark and GFM."
      actions={
        <>
          <Link className="fam-btn fam-btn-primary" to="/rust/getting-started">
            Start with Rust <Mark name="arrow" className="icon" size={18} />
          </Link>
          <Link className="fam-btn fam-btn-ghost" to="/node/getting-started">
            Start with Node.js
          </Link>
        </>
      }
      install={
        <>
          <code>cargo add ferromark</code>
          <span> · </span>
          <code>npm install ferromark</code>
          <span> · v{version}</span>
        </>
      }
    />
  );
}

function ContractBand() {
  return (
    <IronBand
      title="Correctness is part of the contract."
      intro="The engine keeps Markdown parsing and HTML rendering in scope, with specification tests for correctness and measured public API comparisons for speed. Your application keeps control of trust, translation, routing, and presentation."
      rows={[
        {
          heading: "Standards first",
          text: "CommonMark and GFM define the syntax. Conformance results and retained reference corpora make the behavior reviewable.",
        },
        {
          heading: "Trust stays explicit",
          text: "Rust preserves raw HTML unless you enable sanitizing. Node.js escapes authored HTML and filters unsafe URL schemes until you trust the source.",
        },
        {
          heading: "A focused core",
          text: "Ferromark parses Markdown and renders HTML, with opt-in typography and technical abbreviation markup. Translation, site assembly, templates, and syntax highlighting stay in your pipeline.",
        },
      ]}
    />
  );
}

function PipelineSection() {
  return (
    <Section
      id="pipeline"
      title="A Markdown stage that fits your pipeline."
      intro="Connect the renderer to the rest of your content tooling. Ferromark is the Markdown step; the surrounding tools remain independently useful."
    >
      <PipelineAssembly current="ferromark" />
      <RunSample
        input={landingSample.markdown}
        inputCaption="content.md"
        inputKind="Markdown source"
        output={landingSample.html}
        outputCaption={
          <>
            Rendered by Ferromark {landingSample.renderedWith} from commit{" "}
            <code>{landingSample.sourceCommit}</code>
          </>
        }
      />
    </Section>
  );
}

function EvidenceSection() {
  return (
    <Section
      id="evidence"
      layout="split"
      title={benchmarkLead()}
      intro="CommonMark and GFM, plus opt-in publishing features for richer documents. Enable what you need; extra syntax adds parsing work."
      note={
        <>
          57 documents on macOS and Linux, fresh calls. 2× means twice the throughput. Node.js
          includes binding overhead. <Link to="/guide/benchmarks">Methods and raw data</Link> ·{" "}
          <Link to="/guide/feature-comparison">Features and costs</Link>
        </>
      }
    >
      <BenchmarkComparison />
      <p className="native-benchmark-note">{benchmarkHosts}</p>
    </Section>
  );
}

function ConformanceSection() {
  return (
    <Section
      id="conformance"
      layout="split"
      title="Conformance with the evidence attached."
      intro="Published output remains tied to the specification and compatibility fixtures."
      note={
        <Link to="/guide/correctness">Read the conformance methodology and source fixtures.</Link>
      }
    >
      <Ledger entries={conformanceEntries} />
    </Section>
  );
}

function StartSection() {
  return (
    <ClosingAction
      id="start"
      title="Bring Markdown into your application."
      actions={
        <>
          <Link className="fam-btn fam-btn-primary" to="/rust/getting-started">
            Rust guide <Mark name="arrow" className="icon" size={18} />
          </Link>
          <Link className="fam-btn fam-btn-ghost" to="/node/getting-started">
            Node.js guide
          </Link>
        </>
      }
      links={
        <>
          <a href="https://github.com/sebastian-software/ferromark">GitHub repository</a>
          <span> · </span>
          <a href="https://github.com/sebastian-software/ferromark/blob/main/docs/migration-v2.md">
            Migration guide
          </a>
          <span> · </span>
          <Link to="/guide/ffm">FFM guide</Link>
        </>
      }
    >
      <p>
        Choose Rust for direct access to the parser and document tree, or Node.js for a typed
        JavaScript API backed by the same native engine.
      </p>
    </ClosingAction>
  );
}

export default function HomePage() {
  return (
    <RegistryFacts snapshot={registrySnapshot} snapshotGeneratedAt={registrySnapshotGeneratedAt}>
      {/* A div, not a second <main>: Ardo already renders the page's main landmark. */}
      <div className="fam-page">
        <ProjectIntro />
        <ContractBand />
        <PipelineSection />
        <FlavoredMarkdownSection />
        <EvidenceSection />
        <ConformanceSection />
        <StartSection />
      </div>
    </RegistryFacts>
  );
}
