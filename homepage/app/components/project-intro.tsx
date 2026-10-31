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
      title="Absurdly fast. Fully compatible."
      introduction="Ferromark renders Markdown faster than every library we measured, and it does not bend a single rule of CommonMark or GFM to get there. Native for Rust and Node.js, with a stack of extras on top when you want them."
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
        title="Speed you can measure."
        icon={<Mark name="document" size={28} />}
        meta={<FerraReleaseFacts tool={tool} />}
      >
        <p>
          Ahead of every library in our 57-document benchmark, on Linux and macOS, from Rust and
          from Node.js. Fast enough that rendering stops being a step you think about.
        </p>
        <a href="#evidence">See the numbers</a>
      </FerraArticle>
      <FerraArticle
        title="Compatible down to the edge cases."
        icon={<Mark name="pen" size={28} />}
        meta={<a href="/guide/correctness">How correctness is checked</a>}
      >
        <p>
          All 652 CommonMark examples, the official GFM extensions, every line-ending variant. The
          Markdown you already have keeps rendering the way it should.
        </p>
        <a href="#conformance">See the conformance results</a>
      </FerraArticle>
    </FerraColumns>
  );
}

export function ProjectContext() {
  return (
    <FerraNote
      title="The details are the point."
      source={{ href: "/guide/architecture", label: "Inside Ferromark" }}
    >
      <p>
        Curly quotes. A caption under the photo. A footnote that links back. Readers rarely notice
        when these are right, and always notice when they are wrong. Ferromark takes care of them,
        so your attention can go to the words.
      </p>
    </FerraNote>
  );
}
