import { Mark } from "ferramenta-family";
import {
  FerraAction,
  FerraArticle,
  FerraColumns,
  FerraInstrument,
  FerraNote,
  FerraProductLead,
  FerraReleaseFacts,
} from "ferramenta-family/ferra";
import { WORKSHOP } from "ferramenta-family/registry";

import { tool } from "../data/project";

export function ProjectOpening() {
  return (
    <FerraProductLead
      id="overview"
      title="Publish richer Markdown. At native speed."
      introduction="A Markdown engine for Rust and Node.js. CommonMark and GFM, with opt-in publishing features and native typography transforms."
      art={<FerraInstrument product="ferromark" />}
      byline={
        <>
          An open-source project by <a href={WORKSHOP.openSource}>{WORKSHOP.company}</a>.
        </>
      }
    >
      <div className="ferra-actions">
        <FerraAction href="/rust/getting-started">Start with Rust</FerraAction>
        <FerraAction variant="secondary" href="/node/getting-started">
          Start with Node.js
        </FerraAction>
      </div>
    </FerraProductLead>
  );
}

export function ProjectOverview() {
  if (tool === undefined) throw new Error("Missing Ferromark registry entry");
  return (
    <FerraColumns>
      <FerraArticle
        title="The publishing details, already built in."
        icon={<Mark name="document" size={28} />}
        meta={<FerraReleaseFacts tool={tool} />}
      >
        <p>
          Add image captions, quote attributions, footnotes and structured tables. Enable what your
          documents need.
        </p>
        <a href="/guide/ffm">Explore publishing features</a>
      </FerraArticle>
      <FerraArticle
        title="Give prose its final polish."
        icon={<Mark name="pen" size={28} />}
        meta={<a href="#evidence">Benchmarks, methods and conditions</a>}
      >
        <p>
          Afterburner adds language-aware typography, linked GitHub references and emoji before
          rendering.
        </p>
        <a href="#features">Try the finishing touches</a>
      </FerraArticle>
    </FerraColumns>
  );
}

export function ProjectContext() {
  return (
    <FerraNote
      title="Your document. Your presentation."
      source={{ href: "/guide/architecture", label: "Inside Ferromark" }}
    >
      <p>
        Your application owns routing, translation and presentation. Ferromark handles parsing,
        rendering and the document tree.
      </p>
    </FerraNote>
  );
}
