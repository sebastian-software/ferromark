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
      title="Markdown that arrives finished."
      introduction="You write plain Markdown. Your readers get real typography, captioned images and footnotes that land where they belong. Ferromark does the polishing, natively in Rust and Node.js, and faster than every library we measured."
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
        title="Everything a real document needs."
        icon={<Mark name="document" size={28} />}
        meta={<FerraReleaseFacts tool={tool} />}
      >
        <p>
          Captions, quote attributions, footnotes, tables with merged cells. The things you used to
          bolt on later are simply there. Turn on what your documents use, skip the rest.
        </p>
        <a href="/guide/ffm">See what you can write</a>
      </FerraArticle>
      <FerraArticle
        title="Fast, and right down to the edge cases."
        icon={<Mark name="pen" size={28} />}
        meta={<a href="#conformance">How correctness is checked</a>}
      >
        <p>
          Ahead of every library in our 57-document benchmark, and in agreement with all 652
          CommonMark examples. Speed you can measure, correctness you can check.
        </p>
        <a href="#evidence">See the numbers</a>
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
