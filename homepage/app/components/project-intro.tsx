import { Mark } from "ferramenta-family";
import {
  FerraAction,
  FerraArticle,
  FerraEditorialRow,
  FerraInstrument,
  FerraNote,
  FerraReleaseFacts,
} from "ferramenta-family/ferra";
import { WORKSHOP } from "ferramenta-family/registry";

import { tool } from "../data/project";
import { version } from "../version";

function ProjectIntro() {
  if (tool === undefined) throw new Error("Missing Ferromark registry entry");
  return (
    <FerraArticle
      id="overview"
      title="One engine. Two ways to build."
      introduction="A native Markdown engine for Rust and Node.js."
      art={<FerraInstrument product="ferromark" />}
      meta={
        <>
          <FerraReleaseFacts tool={tool} />
          <p>Documented release v{version}</p>
        </>
      }
    >
      <p>
        Keep CommonMark and GFM, add publishing features when you need them, and give your prose its
        final polish with Afterburner.
      </p>
      <div className="ferra-actions">
        <FerraAction href="/rust/getting-started">
          Start with Rust <Mark name="arrow" size={18} />
        </FerraAction>
        <FerraAction variant="secondary" href="/node/getting-started">
          Start with Node.js
        </FerraAction>
      </div>
      <div className="ferra-install">
        <code>cargo add ferromark</code>
        <code>npm install ferromark</code>
      </div>
    </FerraArticle>
  );
}

export function ProjectOpening() {
  return (
    <FerraEditorialRow
      notes={
        <FerraNote
          title="A focused engine"
          source={{ href: "/guide/architecture", label: "Inside Ferromark" }}
        >
          <p>
            Your application keeps control of routing, translation and presentation. Ferromark
            concentrates on parsing, rendering and the document tree.
          </p>
        </FerraNote>
      }
    >
      <header className="ferra-editorial-intro">
        <h1>Markdown, ready for your application.</h1>
        <p>
          Start with the Markdown you know. Add the publishing details your documents need, and
          build the rest around a native Rust engine.
        </p>
        <p className="ferra-editorial-byline">
          An open-source project by <a href={WORKSHOP.openSource}>{WORKSHOP.company}</a>.
        </p>
      </header>
    </FerraEditorialRow>
  );
}

export function ProjectOverview() {
  return (
    <FerraEditorialRow
      notes={
        <>
          <FerraNote
            title="Choose the API that fits"
            source={{ href: "/guide/quick-start", label: "Compare the entry points" }}
          >
            <p>
              Rust gives you direct access to the parser and document tree. Node.js wraps the same
              native engine in a typed JavaScript API, with HTML strings, buffers and metadata.
            </p>
          </FerraNote>
          <FerraNote
            title="Reuse the work, keep ownership clear"
            source={{ href: "/rust/pipelines", label: "Build a content pipeline" }}
          >
            <p>
              Arena allocation keeps document nodes together. Reuse the arena and renderer buffers
              across a collection, then reset the arena when its documents are no longer needed.
            </p>
          </FerraNote>
        </>
      }
    >
      <ProjectIntro />
    </FerraEditorialRow>
  );
}
