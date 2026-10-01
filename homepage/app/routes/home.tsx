import {
  ClosingAction,
  IronBand,
  Ledger,
  Mark,
  Principles,
  ProjectHero,
  RegistryFacts,
  Relations,
  Section,
  WorkWithUs,
} from "ferramenta-family";
import { Link } from "react-router";

import {
  BenchmarkComparison,
  benchmarkHosts,
  benchmarkLead,
} from "../components/benchmark-comparison";
import { FlavoredMarkdownSection } from "../components/ffm-showcase";
import { TransformShowcase } from "../components/transform-showcase";
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
      icon="ferromark"
      title="Ferromark"
      what="A native Markdown engine for Rust and Node.js."
      lede="Publish Markdown with CommonMark and GFM, add richer publishing features with FFM, and give your content its final polish with Afterburner."
      facts={[
        { label: "Built to", value: "CommonMark / GFM" },
        { label: "Checked against", value: "Specification and reference corpora" },
        { label: "Release", value: `v${version}` },
      ]}
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
    >
      <Principles
        items={[
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
    </IronBand>
  );
}

function FamilySection() {
  return (
    <Section
      id="family"
      title="A place in your toolchain."
      intro="Use Ferromark on its own, pair it with Ferriki for syntax highlighting, or find it inside Palamedes. Each engine has its own job and its own API."
    >
      <Relations current="ferromark" />
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
        <TransformShowcase />
        <FlavoredMarkdownSection />
        <ContractBand />
        <FamilySection />
        <EvidenceSection />
        <ConformanceSection />
        <StartSection />
        <WorkWithUs title="Need Ferromark in your stack?" />
      </div>
    </RegistryFacts>
  );
}
