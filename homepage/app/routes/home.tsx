import type { ArdoMetaFunction } from "ardo/runtime";

import { ArdoLink as Link } from "ardo/runtime";
import { RegistryFacts } from "ferramenta-family";
import {
  FerraAction,
  FerraEditorialRow,
  FerraNotes,
  FerraProductPage,
} from "ferramenta-family/ferra";
import { WORKSHOP } from "ferramenta-family/registry";

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
import { ProjectContext, ProjectOpening, ProjectOverview } from "../components/project-intro";
import { TransformShowcase } from "../components/transform-showcase";
import { tool } from "../data/project";
import { registrySnapshot, registrySnapshotGeneratedAt } from "../data/registry-snapshot";

export const handle = { layout: "bare" };
export const meta: ArdoMetaFunction = () => [
  { title: "Ferromark — Markdown that arrives finished" },
  {
    name: "description",
    content:
      "Write plain Markdown, publish polished HTML. A native engine for Rust and Node.js with CommonMark, GFM, publishing features and real typography.",
  },
];

function ContractBand() {
  return (
    <Section
      id="contract"
      title="Your application stays in charge."
      intro="Ferromark parses and renders, and does that one job really well. Trust, translation, routing and presentation remain your decisions."
    >
      <Principles
        items={[
          {
            heading: "Standards, not surprises",
            text: "CommonMark and GFM define the syntax. No private dialect to learn, and no behavior you cannot look up.",
          },
          {
            heading: "Safe until you say otherwise",
            text: "Node.js escapes authored HTML and filters unsafe URL schemes until you trust the source. Rust keeps raw HTML unless you enable sanitizing. Either way, you decide.",
          },
          {
            heading: "Small core, open edges",
            text: "Typography, GitHub references and emoji are optional passes. Highlighting and diagrams plug in through rendering hooks, so you pick the tools.",
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
    detail: "Every single example in the specification renders exactly like the reference.",
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
    detail:
      "Every example again with CR and with CRLF line endings, and the output must not change.",
  },
  {
    name: "Original cmark corpus",
    status: "Covered",
    settled: true,
    detail: "The upstream cmark comparison corpus stays in the regression suite.",
  },
];

function DocumentAnalysisSection() {
  return (
    <Section
      id="document-analysis"
      title="Parse once. Use it everywhere."
      intro="The document Ferromark already parsed for rendering can do more. Check links, feed your search index, or export clean Markdown, without parsing a second time."
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
            text: "Every link, image and static MDX URL, with its destination and source range. Broken-link checks become a loop over a list.",
          },
          {
            heading: "Text and search sections",
            text: "Plain text and ordered heading sections, with the final heading IDs and hierarchy your search index needs.",
          },
          {
            heading: "Static Markdown export",
            text: "Strip dynamic MDX, keep the original Markdown and code. Diagnostics tell you exactly what was left out.",
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
      title="Good company."
      intro="Ferromark is happy on its own. Pair it with Ferriki for syntax highlighting, or add Ferrigura diagrams through a custom build. Each engine does one job and has its own API."
      note={<Link to="/guide/code-rendering">Choose a highlighting or diagram integration.</Link>}
    >
      <p>
        <a href="https://ferramenta.dev/#ferromark">Meet the rest of the Ferramenta family</a>.
      </p>
    </Section>
  );
}

function EvidenceSection() {
  return (
    <Section
      id="evidence"
      title={benchmarkLead()}
      intro="All of that polish would be worth little if it made you wait. Ferromark beats every library in our comparison, on Linux and on macOS, from Rust and from Node.js. Fair warning: each extra syntax feature you enable adds a little parsing work."
      note={
        <>
          57 documents on macOS and Linux, fresh calls. 2× means twice the throughput. Node.js
          includes binding overhead. <Link to="/guide/benchmarks">Methods and raw data</Link> ·{" "}
          <Link to="/guide/feature-comparison">Features and costs</Link>
        </>
      }
    >
      <BenchmarkComparison />
      <p className="ferra-caption">{benchmarkHosts}</p>
    </Section>
  );
}

function ConformanceSection() {
  return (
    <Section
      id="conformance"
      title="Fast is nothing without right."
      intro="Speed is easy if you cut corners. We did not. Every result below is backed by the specification and fixtures you can run yourself."
      note={
        <Link to="/guide/correctness">Read the conformance methodology and source fixtures.</Link>
      }
    >
      <dl className="ferra-definitions">
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
    <section className="ferra-editorial-closing" id="start" aria-labelledby="work">
      <h2 id="work">Your next document deserves this.</h2>
      <p>
        One command and you are rendering. Rust gives you the parser and document tree directly,
        Node.js a typed JavaScript API on top of the same native engine.
      </p>
      <div className="ferra-actions">
        <FerraAction href="/rust/getting-started">Start with Rust</FerraAction>
        <FerraAction variant="secondary" href="/node/getting-started">
          Start with Node.js
        </FerraAction>
      </div>
      <div className="ferra-install">
        <code>cargo add ferromark</code>
        <code>npm install ferromark</code>
      </div>
      <p>
        <a href="https://github.com/sebastian-software/ferromark">GitHub repository</a> ·{" "}
        <Link to="/guide/getting-started">Documentation</Link> ·{" "}
        <a href={WORKSHOP.consulting}>Integration and support</a>
      </p>
    </section>
  );
}

export default function HomePage() {
  return (
    <RegistryFacts snapshot={registrySnapshot} snapshotGeneratedAt={registrySnapshotGeneratedAt}>
      <FerraProductPage>
        <ProjectOpening />
        <ProjectOverview />
        <TransformShowcase />
        <ProjectContext />
        <FerraEditorialRow
          notes={
            tool?.editorial !== undefined && (
              <FerraNotes label="Language and publishing" notes={tool.editorial.notes.slice(1)} />
            )
          }
        >
          <FlavoredMarkdownSection />
        </FerraEditorialRow>
        <EvidenceSection />
        <FerraEditorialRow
          notes={
            tool?.editorial !== undefined && (
              <FerraNotes
                label="How correctness is checked"
                notes={tool.editorial.notes.slice(0, 1)}
              />
            )
          }
        >
          <ConformanceSection />
        </FerraEditorialRow>
        <ContractBand />
        <DocumentAnalysisSection />
        <FamilySection />
        <StartSection />
      </FerraProductPage>
    </RegistryFacts>
  );
}
