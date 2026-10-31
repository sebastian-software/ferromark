import { ArdoLink as Link } from "ardo/runtime";
import { Mark } from "ferramenta-family";

import { FeatureGrid as Principles, LandingSection as Section } from "./landing-section";

const publishingBenefits = [
  {
    heading: "Images that explain themselves.",
    text: "A visible caption for every reader, alt text for every screen reader. Nobody has to guess what the picture means.",
  },
  {
    heading: "Quotes that keep their author.",
    text: "The attribution sits beside the quotation, never mixed into it. Credit travels wherever the passage goes.",
  },
  {
    heading: "Tables that hold real data.",
    text: "Captions and merged cells explain the structure. Named columns let your CSS style them without touching the document.",
  },
];

export function FlavoredMarkdownSection() {
  return (
    <Section
      id="ffm"
      className="ferromark-ffm"
      title="Then come the extras."
      intro="Standard Markdown is the foundation. On top, Ferromark brings a whole set of its own extensions: front matter, footnotes, callouts, tables of contents, and Ferromark Flavored Markdown for captions, attributions and richer tables. No plugin hunt. Everything is opt-in, so plain documents stay plain and fast."
      note={
        <Link to="/guide/ffm">
          See everything FFM can do <Mark name="arrow" className="icon" size={18} />
        </Link>
      }
    >
      <Principles items={publishingBenefits} />
    </Section>
  );
}
