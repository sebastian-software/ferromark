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

import { benchmarkLead, benchmarkRange } from "../components/benchmark-data";
import { FlavoredMarkdownSection } from "../components/ffm-showcase";
import {
  FeatureGrid as Principles,
  LandingSection as Section,
} from "../components/landing-section";
import { ProjectContext, ProjectOpening, ProjectOverview } from "../components/project-intro";
import { tool } from "../data/project";
import { registrySnapshot, registrySnapshotGeneratedAt } from "../data/registry-snapshot";

export const handle = { layout: "bare" };
export const meta: ArdoMetaFunction = () => [
  { title: "Ferromark — Absurdly fast, fully compatible Markdown" },
  {
    name: "description",
    content:
      "A native Markdown engine for Rust and Node.js. Faster than every library we measured, fully CommonMark and GFM compatible, with publishing extensions and typography on top.",
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
            heading: "Memory-safe by design",
            text: "Written in Rust, so whole classes of memory bugs never compile. Unsafe code fails the build, except in a few small, documented SIMD and allocator spots.",
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
      intro="Twenty Markdown libraries. 57 real documents. Both macOS and Linux. Ferromark delivered more throughput than every library in our comparison, from Rust and from Node.js."
      note={
        <>
          <Link to="/guide/benchmarks#markdown-ecosystem">Explore the benchmarks</Link> ·{" "}
          <Link to="/guide/correctness#library-comparison-scores">
            Compare specification agreement
          </Link>
        </>
      }
    >
      <p id="markdown-ecosystem">
        In native Rust, Ferromark delivered {benchmarkRange("pulldown-cmark")} the throughput of
        pulldown-cmark. Through Node.js, it delivered {benchmarkRange("marked")} that of marked,
        including binding overhead. These ranges cover both recorded hosts, using fresh calls across
        the full corpus.
      </p>
      <p>
        Your workload and enabled features affect the result. Some libraries also produce different
        output for these inputs. The detailed comparison keeps those differences, configurations and
        source measurements in view.
      </p>
    </Section>
  );
}

function ConformanceSection() {
  return (
    <Section
      id="conformance"
      title="And not one shortcut."
      intro="Speed is easy if you cut corners. Ferromark keeps the full CommonMark and GFM behavior, and every result below is backed by specification fixtures you can run yourself."
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
        Choose your runtime and follow the getting-started guide. Rust gives you the parser and
        document tree directly, Node.js a typed JavaScript API on top of the same native engine.
      </p>
      <div className="ferra-actions">
        <FerraAction href="/rust/getting-started">Start with Rust</FerraAction>
        <FerraAction variant="secondary" href="/node/getting-started">
          Start with Node.js
        </FerraAction>
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
        <FerraEditorialRow
          notes={
            tool?.editorial !== undefined && (
              <FerraNotes label="Language and publishing" notes={tool.editorial.notes.slice(1)} />
            )
          }
        >
          <FlavoredMarkdownSection />
        </FerraEditorialRow>
        <ProjectContext />
        <ContractBand />
        <DocumentAnalysisSection />
        <FamilySection />
        <StartSection />
      </FerraProductPage>
    </RegistryFacts>
  );
}
