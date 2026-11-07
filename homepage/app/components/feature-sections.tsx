import { MonoCredit, MonoLeadIn, MonoPassage, MonoSection } from "ferramenta-family/monografie";

import { nodeVersion, rustVersion } from "../version";
import {
  AnalysisCards,
  Applications,
  CodeCards,
  FfmFeatures,
  RuntimeCards,
  TrustCards,
} from "./homepage-cards";
import { samples } from "./homepage-data";
import { RenderedSheets } from "./rendered-sheets";
import { TypographyLoupe } from "./typography-loupe";

/* The homepage's later chapters: richer documents, typography, trust, code, MDX,
   runtimes and the applications that run on Ferromark. */

/** Where each API starts, and what keeps untrusted Markdown safe. */
function DefaultsTable() {
  return (
    <table className="mono-table fm-defaults">
      <thead>
        <tr>
          <th scope="col">API</th>
          <th scope="col">Default</th>
          <th scope="col">For untrusted Markdown</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>Rust</td>
          <td>Raw HTML passthrough</td>
          <td>
            <code>HtmlRendererOptions::sanitize</code>
          </td>
        </tr>
        <tr>
          <td>Node.js</td>
          <td>Escaped raw HTML, filtered URL schemes</td>
          <td>
            Keep <code>renderPolicy: &apos;untrusted&apos;</code>
          </td>
        </tr>
      </tbody>
    </table>
  );
}

export function Ffm() {
  return (
    <MonoSection
      id="ffm"
      title="Richer documents, still plain text."
      standfirst="Captions, credits, merged table cells, highlights and notes, written in Markdown and rendered as proper HTML."
    >
      <p>
        <MonoLeadIn>Plain Markdown stops</MonoLeadIn> where real publishing begins. A photo needs a
        caption, a quotation needs its source, a price table needs a cell that spans two columns.
        Usually that means falling back to raw HTML, or to a shortcode only one tool understands.
      </p>
      <MonoPassage
        note={{
          side: "right",
          term: "Off until you turn it on",
          children: (
            <>
              Turn each feature on alone, or all of them with the <code>ffm</code> preset.
            </>
          ),
        }}
      >
        <p>
          Ferromark Flavored Markdown adds these features as a small set of syntax that still reads
          naturally as text. Each one is off until you turn it on, so documents that don&rsquo;t use
          them pay nothing for them, and standard CommonMark keeps meaning exactly what it means.
        </p>
      </MonoPassage>
      <RenderedSheets
        markdown={samples.ffm.markdown}
        html={samples.ffm.html}
        file="pricing.md"
        call={`toHtml(source, ${samples.ffm.options})`}
        captions
        caption={`Real output of ferromark ${samples.renderedWith.ferromark}. Point at a line of the source to see what it became. The technical abbreviation in the note is marked up as well.`}
      />
      <FfmFeatures />
    </MonoSection>
  );
}

export function Typography() {
  return (
    <MonoSection
      id="typography"
      title="Punctuation that speaks the language."
      standfirst="Straight quotes become proper ones, in fifteen languages, each by its own rules."
    >
      <p>
        <MonoLeadIn>The typewriter</MonoLeadIn> gave us one straight quotation mark for everything.
        Typesetters never accepted it, and neither do careful readers. English opens a quotation
        with “, German with „, French sets « guillemets » with space inside them. An optional
        typography pass turns straight quotes, apostrophes and ellipses into the right characters
        after parsing, for the one language your application chooses. Ferromark never guesses the
        language. Code, links and raw HTML stay exactly as written.
      </p>
      <TypographyLoupe />
    </MonoSection>
  );
}

export function Trust() {
  return (
    <MonoSection
      id="trust"
      title="Recognition is not permission."
      standfirst="Ferromark can read raw HTML without letting it through."
    >
      <p>
        <MonoLeadIn>Markdown allows</MonoLeadIn> raw HTML. That is handy for trusted authors and
        dangerous for everyone else: a comment field that passes an <code>onerror</code> handler
        through is an attack waiting to happen. Ferromark keeps recognizing HTML separate from
        emitting it. In Node.js the untrusted mode is the default: raw HTML is escaped, and unsafe
        link schemes such as <code>javascript:</code> are filtered. In Rust, one option turns
        sanitizing on.
      </p>
      <MonoPassage
        note={{ figure: 100, children: "levels of block nesting by default, then an error" }}
      >
        <p>
          The engine is written in Rust, so whole classes of memory errors cannot compile. Unsafe
          code is confined to a few small, documented SIMD and allocator spots. Input is bounded
          too: block nesting stops at 100 levels by default and returns an error instead of
          exhausting the stack.
        </p>
      </MonoPassage>
      <TrustCards />
      <DefaultsTable />
    </MonoSection>
  );
}

export function Code() {
  return (
    <MonoSection id="code" title="Code highlighted. Diagrams drawn.">
      <p>
        <MonoLeadIn>Technical writing</MonoLeadIn> is full of code samples and diagrams. Ferromark
        hands each code block to a hook. Connect Ferriki and the code comes back highlighted with
        Shiki-compatible grammars and themes. Connect Ferrigura and a Mermaid block becomes a static
        SVG diagram, rendered on the server without a browser. Both are separate engines from the
        same workshop. Ferromark contains neither until you add it, and any other highlighter fits
        the same hook.
      </p>
      <CodeCards />
    </MonoSection>
  );
}

export function Mdx() {
  return (
    <MonoSection id="mdx" title="From Markdown to components. Parse once, use it everywhere.">
      <p>
        <MonoLeadIn>Documentation sites</MonoLeadIn> built with React and similar frameworks mix
        Markdown with components: an interactive chart inside an article, a tabbed code example.
        That format is called MDX. <code>compileJsx</code> turns an MDX document into JSX, and your
        build supplies the components. Ferromark never executes the authored JavaScript while
        compiling and adds no framework imports of its own.
      </p>
      <p>
        A parsed document holds more than HTML. Ask it for every link and image with its exact
        position, and a broken-link check becomes a loop over a list. Ask for ordered heading
        sections with their final IDs, ready for a search index. Ask for a static Markdown export
        that says exactly what it left out. None of it parses the document a second time.
      </p>
      <AnalysisCards />
      <MonoCredit>
        Both cards show real output of ferromark {samples.renderedWith.ferromark}, the current
        release.
      </MonoCredit>
    </MonoSection>
  );
}

export function Runtimes() {
  return (
    <MonoSection id="runtimes" title="One engine, in Rust or Node.js.">
      <p>
        Use the crate directly in Rust: parse into an arena-allocated tree, inspect or transform it,
        then render. Or install the npm package: the same native engine behind a typed JavaScript
        API. The two share the core and differ in defaults on purpose. Node.js starts in untrusted
        mode because it more often renders text that users typed.
      </p>
      <RuntimeCards rust={rustVersion} node={nodeVersion} />
    </MonoSection>
  );
}

export function AtWork() {
  return (
    <MonoSection id="work" title="At work in Ardo and Palamedes.">
      <p>
        Two applications from the same workshop run on Ferromark. The guides on this site are
        rendered by Ferromark through Ardo.
      </p>
      <Applications />
    </MonoSection>
  );
}
