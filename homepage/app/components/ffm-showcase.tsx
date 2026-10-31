import { ArdoLink as Link } from "ardo/runtime";
import { Mark } from "ferramenta-family";
import { FerraExample } from "ferramenta-family/ferra";

import sample from "../data/ffm-sample.json";
import { FeatureGrid as Principles, LandingSection as Section } from "./landing-section";

const syntax = [
  {
    heading: "Credit under a quote.",
    text: (
      <p>
        A line starting with <code>:</code> right after a quotation becomes its source. Readers see
        who said it, and the credit stays outside the quoted words.
      </p>
    ),
  },
  {
    heading: "Tables that explain themselves.",
    text: (
      <p>
        The same <code>:</code> line under a table becomes its caption. A doubled pipe{" "}
        <code>||</code> merges a cell across columns, so “On request” spans both prices.
      </p>
    ),
  },
  {
    heading: "Small marks with meaning.",
    text: (
      <p>
        <code>==…==</code> highlights, <code>++…++</code> marks an insertion, and <code>^[…]</code>{" "}
        drops a footnote right where the thought happens.
      </p>
    ),
  },
];

function FfmExtras() {
  return (
    <>
      Also on board: <Link to="/guide/configuration">front matter, callouts and footnotes</Link>,
      plus <Link to="/guide/typography">Afterburner</Link> for real typography, GitHub links and
      emoji.{" "}
      <Link to="/guide/ffm">
        Every FFM feature <Mark name="arrow" className="icon" size={18} />
      </Link>
    </>
  );
}

export function FlavoredMarkdownSection() {
  return (
    <Section
      id="ffm"
      className="ferromark-ffm"
      title="Then come the extras."
      intro="Plain Markdown runs out of words exactly where publishing gets interesting: who said this, what does this table show, which cells belong together. Ferromark Flavored Markdown fills those gaps with a few small additions that still read like plain text. All of them are opt-in, so plain documents stay plain and fast."
      note={<FfmExtras />}
    >
      <FerraExample
        title="Write this. Get this."
        introduction="One Markdown source with the FFM preset, rendered by Ferromark itself."
        source={sample.markdown}
        controls={null}
      >
        {/* Frozen output of the fixed source above; no visitor input reaches the preview. */}
        <div dangerouslySetInnerHTML={{ __html: sample.html }} />
      </FerraExample>
      <Principles items={syntax} />
    </Section>
  );
}
