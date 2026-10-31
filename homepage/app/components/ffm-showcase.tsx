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
      title="Say more. It is still Markdown."
      intro="No HTML islands, no shortcodes, no plugin hunt. Ferromark Flavored Markdown lets you write the context your readers need right where it belongs, in syntax that still reads like plain text. Every feature is opt-in."
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
