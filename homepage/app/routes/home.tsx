import { Link } from "ardo/router";
import {
  FerraAction,
  FerraCode,
  FerraColumns,
  FerraHero,
  FerraSection,
  FerraSpecimen,
  FerraTable,
} from "ferramenta-family/ferra";

import { HomeMeasurements } from "../components/home-measurements";
import { ApiNotebook } from "../components/home-reference";
import { version } from "../version";
import "../styles/home.css";

export const handle = { layout: "bare" };

function Introduction() {
  return (
    <div className="ferra-hero-ground">
      <FerraHero
        product="ferromark"
        title="Ferromark."
        what="Markdown, ready for your pipeline."
        actions={
          <>
            <FerraAction href="/rust/getting-started">Start with Rust</FerraAction>
            <FerraAction href="/node/getting-started" variant="secondary">
              Start with Node.js
            </FerraAction>
          </>
        }
      >
        <p>
          Turn Markdown into HTML and the metadata around it. A focused Rust engine for the
          documents that get complicated.
        </p>
      </FerraHero>
    </div>
  );
}

function WorkingExample() {
  return (
    <FerraSection
      id="example"
      label="Rendering"
      intro="From Markdown to structured HTML and metadata."
      title="See what a document becomes."
      className="ferromark-working-example"
    >
      <p>The same content, from Markdown input to rendered HTML output.</p>
      <FerraColumns className="ferromark-transformation">
        <FerraCode caption="Markdown input" file="release-notes.md">
          {"# Release notes\n\n- Clear structure\n- Useful metadata"}
        </FerraCode>
        <span className="ferromark-transform-arrow" aria-hidden="true">
          →
        </span>
        <FerraSpecimen caption="Rendered HTML output">
          <h3>Release notes</h3>
          <ul>
            <li>Clear structure</li>
            <li>Useful metadata</li>
          </ul>
        </FerraSpecimen>
      </FerraColumns>
      <p>
        A document is more than its markup. Use the AST to inspect structure, extract front matter
        as raw text and let your application decide how to publish it.
      </p>
      <p>
        <Link to="/guide/rendering">Explore rendering and metadata</Link>
      </p>
    </FerraSection>
  );
}

function Correctness() {
  return (
    <FerraSection
      id="correctness"
      label="Output policy"
      intro="Correct handling for real documents."
      title="Correctness is a contract."
      className="ferromark-correctness"
    >
      <FerraColumns>
        <div>
          <p>
            A Markdown engine has to earn trust on the awkward document: a nested list, a reference
            that moves, a table with real content or a footnote added at the last minute.
          </p>
          <p>
            Ferromark starts with CommonMark and GFM, then makes the publishing features explicit.
            Specification profiles turn off publishing extras so conformance can be checked on its
            own terms.
          </p>
          <p>
            <Link to="/guide/correctness">Inspect correctness</Link> ·{" "}
            <Link to="/guide/features">Explore syntax</Link>
          </p>
        </div>
        <FerraSpecimen caption="A small publishing ledger">
          <dl className="ferromark-notebook">
            <div>
              <dt>Front matter</dt>
              <dd>Extracted as raw text</dd>
            </div>
            <div>
              <dt>Heading IDs</dt>
              <dd>Configurable output</dd>
            </div>
            <div>
              <dt>Footnotes</dt>
              <dd>Document references and notes</dd>
            </div>
            <div>
              <dt>Rendering hooks</dt>
              <dd>Your application decides</dd>
            </div>
          </dl>
        </FerraSpecimen>
      </FerraColumns>
    </FerraSection>
  );
}

function RenderingPolicy() {
  return (
    <FerraSection
      id="policy"
      label="Compatibility"
      intro="Recognize content without granting permission."
      title="Recognition is not permission."
    >
      <p>
        The engine can recognize raw HTML without granting permission to emit it. Choose the output
        policy for the source your application accepts.
      </p>
      <FerraTable caption="Defaults depend on the API">
        <thead>
          <tr>
            <th scope="col">API</th>
            <th scope="col">Default</th>
            <th scope="col">For untrusted Markdown</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <th scope="row">Rust</th>
            <td>Raw HTML passthrough</td>
            <td>
              Enable <code>sanitize</code>
            </td>
          </tr>
          <tr>
            <th scope="row">Node.js</th>
            <td>Escaped HTML and filtered URL schemes</td>
            <td>
              Keep <code>renderPolicy: &apos;untrusted&apos;</code>
            </td>
          </tr>
        </tbody>
      </FerraTable>
      <p>
        Custom hooks and highlighter HTML are written verbatim. They must escape untrusted values
        themselves. The GFM tag filter is not a general HTML sanitizer.
      </p>
      <p>
        <Link to="/guide/rendering">Choose your rendering policy</Link>
      </p>
    </FerraSection>
  );
}

function DocumentStructure() {
  return (
    <FerraSection
      id="structure"
      label="The document"
      intro="More than paragraphs."
      title="The document, beyond paragraphs."
    >
      <FerraColumns>
        <div>
          <p>
            Publishing needs structure as well as markup. Extract front matter as raw text; let the
            application interpret YAML or TOML. Configure heading IDs and footnotes for the site you
            are building.
          </p>
          <p>
            Ferromark leaves templates, translation, routing and syntax highlighting outside the
            core. That boundary keeps the renderer focused and gives the surrounding pipeline room
            to make its own decisions.
          </p>
          <p>
            <Link to="/guide/pipelines">Plan your content pipeline</Link> ·{" "}
            <Link to="/guide/ffm">Explore Ferromark Flavored Markdown</Link> ·{" "}
            <Link to="/rust/transforms">Apply Afterburner transforms</Link>
          </p>
        </div>
        <FerraSpecimen caption="API notebook">
          <ApiNotebook />
        </FerraSpecimen>
      </FerraColumns>
    </FerraSection>
  );
}

function DocumentAnalysisSection() {
  return (
    <FerraSection
      id="document-analysis"
      title="Use the document you already parsed."
      intro="Collect references, build search records, and export static Markdown from the same prepared document. Request each analysis when your application needs it."
      label="Document analysis"
    >
      <dl className="ferromark-notebook">
        <div>
          <dt>Reference inventories</dt>
          <dd>
            Collect Markdown links, images, and static MDX URL attributes with destinations and
            available source ranges for your link and asset checks.
          </dd>
        </div>
        <div>
          <dt>Text and search sections</dt>
          <dd>
            Extract static plain text and ordered heading sections with final JSX heading IDs and
            hierarchy for your search index.
          </dd>
        </div>
        <div>
          <dt>Static Markdown export</dt>
          <dd>
            Keep original Markdown and code while removing dynamic MDX constructs. Omission
            diagnostics show where content was discarded.
          </dd>
        </div>
      </dl>
      <p>
        <Link to="/guide/document-analysis">Explore document analysis in Rust and Node.js</Link> ·
        Available on main; check your installed release.
      </p>
    </FerraSection>
  );
}

function GetStarted() {
  return (
    <FerraSection
      id="get-started"
      label="Get started"
      intro="Use it your way."
      title="Build the pipeline that fits."
    >
      <p>
        Start with the Rust crate, or use the native engine through Node.js. Pair Ferromark with
        Ferriki when your documents need syntax highlighting; neither engine requires the other.
      </p>
      <FerraCode caption={`Install Ferromark · version ${version}`} file="Rust">
        cargo add ferromark
      </FerraCode>
      <div className="ferra-actions">
        <FerraAction href="/rust/getting-started">Start with Rust</FerraAction>
        <FerraAction href="/node/getting-started" variant="secondary">
          Start with Node.js
        </FerraAction>
        <Link to="/guide/cli">Use the command line</Link>
      </div>
    </FerraSection>
  );
}

export default function HomePage() {
  return (
    <>
      <Introduction />
      <WorkingExample />
      <Correctness />
      <HomeMeasurements />
      <RenderingPolicy />
      <DocumentStructure />
      <DocumentAnalysisSection />
      <GetStarted />
    </>
  );
}
