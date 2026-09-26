import {
  ClosingAction,
  EvidenceFigures,
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

import { agreementDocuments, formatSpeed, nativeBenchmarks } from "../components/native-benchmarks";
import landingSample from "../data/landing-sample.json";
import { registrySnapshot, registrySnapshotGeneratedAt } from "../data/registry-snapshot";
import { version } from "../version";

const benchmarkFigures = nativeBenchmarks.platforms.flatMap((platform) =>
  platform.figures
    .filter((figure) => figure.id !== "ox-content")
    .map((figure) => ({
      label: `${platform.label} · ${figure.label}`,
      value: formatSpeed(figure.fresh),
      detail: `${figure.documents} documents · fresh parser state`,
      measure: (
        <a href={`https://github.com/sebastian-software/ferromark/tree/main/${platform.report}`}>
          {platform.machine} · {platform.measured} · {platform.revision}
        </a>
      ),
    })),
);

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
          Markdown. <em>Ready for your pipeline.</em>
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
      intro="Every speed claim starts with equivalent output. The engine keeps Markdown parsing and HTML rendering in scope; your application keeps control of trust, translation, routing, and presentation."
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
          text: "Ferromark parses Markdown and renders HTML. Translation, site assembly, templates, and syntax highlighting stay in your pipeline.",
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
      title="Measured on real documents."
      intro={`On ${agreementDocuments()} real documents, five native engines render equivalent HTML. Fresh parser state; higher is faster.`}
      note={
        <>
          Ratios are relative to each engine and platform. Each figure links to its archived report,
          including the machine, measurement date, source revisions, and reproduction commands. See
          the <Link to="/guide/benchmarks">full benchmark guide</Link> for reuse lifecycles and
          per-document results.
        </>
      }
    >
      <EvidenceFigures figures={benchmarkFigures} />
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
          <Link to="/guide/features">Markdown features</Link>
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
      <main className="fam-page">
        <ProjectIntro />
        <ContractBand />
        <PipelineSection />
        <EvidenceSection />
        <ConformanceSection />
        <StartSection />
      </main>
    </RegistryFacts>
  );
}
