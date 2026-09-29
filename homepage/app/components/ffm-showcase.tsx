import { Mark, Section } from "ferramenta-family";
import { Link } from "react-router";

const ffmExamples = [
  {
    title: "Captioned images",
    description: "Keep alternative text for accessibility and write a separate visible caption.",
    options: "imageCaptions",
    markdown: "![Pipeline](pipeline.svg)\n: The pipeline",
    html: `<figure>
<img src="pipeline.svg" alt="Pipeline">
<figcaption>The pipeline</figcaption>
</figure>
`,
  },
  {
    title: "Attributed quotes",
    description: "Put a source below a quotation while keeping it outside the quoted text.",
    options: "blockquoteAttributions",
    markdown: "> A memorable passage.\n: Jane Doe",
    html: `<figure>
<blockquote>
<p>A memorable passage.</p>
</blockquote>

<figcaption>Jane Doe</figcaption>
</figure>
`,
  },
  {
    title: "Structured tables",
    description: "Add captions, cell spans, and CSS hooks to familiar GFM tables.",
    options: "mergedTableCells, tableAttributes",
    markdown: "| A | B |\n| --- | --- |\n| One ||\n\n: Totals {#totals}",
    html: `<table id="totals">
<caption>Totals</caption>
<thead>
<tr>
<th>A</th>
<th>B</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2">One</td>
</tr>
</tbody>
</table>
`,
  },
] as const;

export function FlavoredMarkdownSection() {
  return (
    <Section
      id="ffm"
      className="ferromark-ffm"
      title="Markdown with room for publishing."
      intro="Ferromark Flavored Markdown (FFM) adds syntax for published documents. Use the FFM preset or enable extensions one at a time. Each HTML example below uses only the listed options."
      note={
        <Link to="/guide/ffm">
          Explore FFM syntax and options <Mark name="arrow" className="icon" size={18} />
        </Link>
      }
    >
      <div className="ferromark-ffm-examples">
        {ffmExamples.map((example) => (
          <article className="ferromark-ffm-example" key={example.title}>
            <div className="ferromark-ffm-example-intro">
              <h3>{example.title}</h3>
              <p>{example.description}</p>
              <p className="ferromark-ffm-options">
                Node.js options: <code>{example.options}</code>
              </p>
            </div>
            <div className="ferromark-ffm-comparison">
              <figure className="ferromark-ffm-panel ferromark-ffm-input">
                <figcaption>Markdown input</figcaption>
                <pre>
                  <code>{example.markdown}</code>
                </pre>
              </figure>
              <figure className="ferromark-ffm-panel ferromark-ffm-output">
                <figcaption>HTML output</figcaption>
                <pre>
                  <code>{example.html}</code>
                </pre>
              </figure>
            </div>
          </article>
        ))}
      </div>
    </Section>
  );
}
