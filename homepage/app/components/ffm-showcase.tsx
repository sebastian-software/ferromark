import { ArdoLink as Link } from "ardo/runtime";
import { Mark } from "ferramenta-family";

import { FeatureGrid as Principles, LandingSection as Section } from "./landing-section";

const publishingBenefits = [
  {
    heading: "Give images their context.",
    text: "Add a visible caption while keeping alternative text for accessibility. Your reader gets the explanation; your image keeps its meaning.",
  },
  {
    heading: "Make the source clear.",
    text: "Put an attribution beside a quotation, separate from the quoted words. Credit travels with the passage.",
  },
  {
    heading: "Let tables carry real data.",
    text: "Use captions and merged cells to explain the structure. Style whole columns through CSS without rewriting the document.",
  },
];

export function FlavoredMarkdownSection() {
  return (
    <Section
      id="ffm"
      className="ferromark-ffm"
      title="More meaning. Still Markdown."
      intro="Write the context your readers need, right where it belongs. Ferromark Flavored Markdown adds publishing features to familiar Markdown. Enable the ones your documents use."
      note={
        <Link to="/guide/ffm">
          See what FFM makes possible <Mark name="arrow" className="icon" size={18} />
        </Link>
      }
    >
      <Principles items={publishingBenefits} />
    </Section>
  );
}
