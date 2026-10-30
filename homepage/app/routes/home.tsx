import type { ArdoMetaFunction } from "ardo/runtime";

import { ArdoLink as Link } from "ardo/runtime";
import { Mark, RegistryFacts } from "ferramenta-family";
import { FerraAction, FerraHero, FerraReleaseFacts, FerraSoftCard } from "ferramenta-family/ferra";
import { family } from "ferramenta-family/registry";

import {
  BenchmarkComparison,
  benchmarkHosts,
  benchmarkLead,
} from "../components/benchmark-comparison";
import { FlavoredMarkdownSection } from "../components/ffm-showcase";
import {
  FeatureGrid as Principles,
  LandingSection as Section,
} from "../components/landing-section";
import { TransformShowcase } from "../components/transform-showcase";
import { registrySnapshot, registrySnapshotGeneratedAt } from "../data/registry-snapshot";
import { version } from "../version";

export const handle = { layout: "bare" };
export const meta: ArdoMetaFunction = () => [
  { title: "Ferromark — Markdown at native speed" },
  {
    name: "description",
    content:
      "A native Markdown engine for Rust and Node.js. CommonMark and GFM, publishing features, and optional typography transforms.",
  },
];
const tool = family.find((member) => member.name === "ferromark");
if (!tool) throw new Error("Missing Ferromark registry entry");

function ProjectIntro() {
  if (tool === undefined) throw new Error("Missing Ferromark registry entry");
  return (
    <section className="ferromark-intro" aria-label="Ferromark overview">
      <FerraHero
        product="ferromark"
        title="Ferromark."
        what="Markdown, ready for your application."
        actions={
          <>
            <FerraAction href="/rust/getting-started">
              Start with Rust <Mark name="arrow" size={18} />
            </FerraAction>
            <FerraAction variant="secondary" href="/node/getting-started">
              Start with Node.js
            </FerraAction>
          </>
        }
      >
        <p>
          A native Markdown engine for Rust and Node.js. Keep CommonMark and GFM, add publishing
          features when you need them, and give your prose its final polish with Afterburner.
        </p>
        <p className="ferromark-install">
          <code>cargo add ferromark</code>
          <code>npm install ferromark</code>
        </p>
      </FerraHero>
      <div className="ferromark-intro-facts">
        <FerraReleaseFacts tool={tool} />
        <span>Documented release v{version}</span>
      </div>
    </section>
  );
}

function ContractBand() {
  return (
    <Section
      id="contract"
      title="Correctness is part of the contract."
      intro="The engine keeps parsing and rendering in scope. Your application chooses trust, translation, routing and presentation."
    >
      <Principles
        items={[
          {
            heading: "Standards first",
            text: "CommonMark and GFM define the syntax. Conformance results and reference corpora keep the behavior reviewable.",
          },
          {
            heading: "Trust stays explicit",
            text: "Rust preserves raw HTML unless you enable sanitizing. Node.js escapes authored HTML and filters unsafe URL schemes until you trust the source.",
          },
          {
            heading: "A focused core",
            text: "Optional transforms add typography, GitHub references and emoji. Applications connect highlighting and diagrams through rendering hooks.",
          },
        ]}
      />
    </Section>
  );
}
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

function DocumentAnalysisSection() {
  return (
    <Section
      id="document-analysis"
      title="Use the document you already parsed."
      intro="Collect references, build search records, and export static Markdown from the same prepared document. Request each analysis when your application needs it."
      note={
        <>
          <Link to="/guide/document-analysis">Explore document analysis in Rust and Node.js</Link>
          {" · "}Available on main; check your installed release.
        </>
      }
    >
      <Principles
        items={[
          {
            heading: "Reference inventories",
            text: "Collect Markdown links, images, and static MDX URL attributes with destinations and available source ranges for your link and asset checks.",
          },
          {
            heading: "Text and search sections",
            text: "Extract static plain text and ordered heading sections with final JSX heading IDs and hierarchy for your search index.",
          },
          {
            heading: "Static Markdown export",
            text: "Keep original Markdown and code while removing dynamic MDX constructs. Omission diagnostics show where content was discarded.",
          },
        ]}
      />
    </Section>
  );
}

function FamilySection() {
  return (
    <Section
      id="family"
      title="A place in your toolchain."
      intro="Use Ferromark on its own, pair it with Ferriki for syntax highlighting, or add Ferrigura diagrams through a custom build. Each engine has its own job and its own API."
      note={<Link to="/guide/code-rendering">Choose a highlighting or diagram integration.</Link>}
    >
      <p className="ferromark-family-note">
        <a href="https://ferramenta.dev/#ferromark">
          Explore the independent engines in Ferramenta
        </a>
        .
      </p>
    </Section>
  );
}

function EvidenceSection() {
  return (
    <Section
      id="evidence"
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
      title="Conformance with the evidence attached."
      intro="Published output remains tied to the specification and compatibility fixtures."
      note={
        <Link to="/guide/correctness">Read the conformance methodology and source fixtures.</Link>
      }
    >
      <dl className="ferromark-conformance-grid">
        {conformanceEntries.map((entry) => (
          <div key={entry.name}>
            <dt>{entry.name}</dt>
            <dd>
              <strong>{entry.status}</strong>
              <p>{entry.detail}</p>
            </dd>
          </div>
        ))}
      </dl>
    </Section>
  );
}

function StartSection() {
  return (
    <FerraSoftCard
      as="section"
      className="ferromark-section ferromark-start"
      id="start"
      aria-labelledby="work"
    >
      <h2 id="work">Bring Markdown into your application.</h2>
      <p>
        Choose Rust for direct access to the parser and document tree, or Node.js for a typed
        JavaScript API backed by the same native engine.
      </p>
      <div className="ferra-actions">
        <FerraAction href="/rust/getting-started">Start with Rust</FerraAction>
        <FerraAction variant="secondary" href="/node/getting-started">
          Start with Node.js
        </FerraAction>
      </div>
      <p>
        <a href="https://github.com/sebastian-software/ferromark">GitHub repository</a> ·{" "}
        <a href="/guide/getting-started">Documentation</a> ·{" "}
        <a href="https://sebastian-consulting.com">Integration and support</a>
      </p>
    </FerraSoftCard>
  );
}

export default function HomePage() {
  return (
    <RegistryFacts snapshot={registrySnapshot} snapshotGeneratedAt={registrySnapshotGeneratedAt}>
      <div className="ferromark-home ferra-wrap">
        <ProjectIntro />
        <TransformShowcase />
        <FlavoredMarkdownSection />
        <DocumentAnalysisSection />
        <ContractBand />
        <FamilySection />
        <EvidenceSection />
        <ConformanceSection />
        <StartSection />
      </div>
    </RegistryFacts>
  );
}
