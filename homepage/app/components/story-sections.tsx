import { MonoPassage, MonoProofs, MonoSection, MonoTerms } from "ferramenta-family/monografie";

import { EngineDrawing } from "./engine-drawing";
import { measured, ownAgreement, samples, suiteLabels } from "./homepage-data";
import { AgreementTable, SpeedChart } from "./homepage-evidence";
import { RenderedSheets } from "./rendered-sheets";

/* The homepage's first chapters: what a renderer is, speed, the engine and correctness. */

/* From the correctness guide's own runs: every example again with CR and CRLF
   line endings, and the original cmark / cmark-gfm comparison corpus. */
const lineEndingVariants = (ownAgreement.commonmark.total * 2).toLocaleString("en-US");
const cmarkCorpus = 106;

const proofs = [
  {
    label: `${suiteLabels.commonmark} examples`,
    value: `${ownAgreement.commonmark.passed} / ${ownAgreement.commonmark.total}`,
  },
  {
    label: "GFM extension examples",
    value: `${ownAgreement.gfm.passed} / ${ownAgreement.gfm.total}`,
  },
  {
    label: "Every example again with CR and CRLF line endings",
    value: `${lineEndingVariants} / ${lineEndingVariants}`,
  },
  { label: "Original cmark and cmark-gfm corpus", value: `${cmarkCorpus} / ${cmarkCorpus}` },
];

export function Explainer() {
  return (
    <MonoSection title="Plain text in. A finished page out.">
      <MonoPassage
        note={{
          term: "Markdown",
          children: "Plain text with a few marks for structure, readable as it is typed.",
        }}
      >
        <p>
          Markdown is how most technical writing is typed today. READMEs on GitHub, documentation
          sites, release notes, chat messages, notes apps and the answers of AI assistants are
          written as plain text with a few marks: a pound sign for a heading, asterisks for
          emphasis, hyphens for a list. The marks stay readable for the person who writes them. A
          browser cannot display them, though. Something has to translate the text into HTML, the
          language of web pages.
        </p>
      </MonoPassage>
      <MonoPassage
        note={{
          side: "right",
          term: "Renderer",
          children: "The program that reads the marks and writes the HTML. Ferromark is one.",
        }}
      >
        <p>
          That translator is a Markdown renderer. It reads every character and decides which marks
          belong together. Is this asterisk emphasis, or a multiplication sign? Does this indented
          line continue the list, or start a block of code? Then it writes the matching HTML. Most
          people never see it. They notice it only when it gets something wrong: a list that falls
          apart, a link that swallows a sentence, a page that takes too long to build.
        </p>
      </MonoPassage>
      <p>
        Ferromark is that translator, built to be unnoticeable: exact on the awkward document, fast
        on the ten-thousandth one, and careful with text it did not write.
      </p>
      <RenderedSheets
        markdown={samples.basic.markdown}
        html={samples.basic.html}
        file="release-notes.md"
        call="toHtml()"
        caption="Point at a line of the source to see the element it became."
      />
    </MonoSection>
  );
}

export function Speed() {
  return (
    <MonoSection
      id="speed"
      title="Ahead of every library we measured."
      standfirst={`Twenty Markdown libraries, ${measured.documents} real documents, two machines. Ferromark delivered the most throughput in every comparison, natively in Rust and through Node.js.`}
    >
      <MonoPassage
        note={{ figure: measured.documents, children: "real documents, every one of them counted" }}
      >
        <p>
          Why speed matters for text: a documentation site with a few thousand pages renders all of
          them on every build. A chat app renders each message as it arrives. A search index rereads
          everything when the content changes. Each millisecond is paid thousands of times over: in
          waiting developers, in CI minutes, in server costs, in battery.
        </p>
      </MonoPassage>
      <SpeedChart />
    </MonoSection>
  );
}

export function Inside() {
  return (
    <MonoSection
      id="inside"
      title="Inside the engine."
      standfirst="One crate, four modules. The speed comes from how the work is organized, not from work that was skipped."
    >
      <EngineDrawing />
      <MonoPassage
        note={{
          term: "Arena",
          children: "One block of memory per document, released in one step.",
        }}
      >
        <p>
          One block of memory. Ferromark keeps a whole document in one arena, a single block of
          memory that is filled, used and released at once instead of in thousands of small pieces.
          It scans bytes with specialized routines and SIMD instructions where the processor offers
          them, and it reuses buffers from one document to the next. None of this changes the
          result.
        </p>
      </MonoPassage>
      <p>
        Between parsing and rendering, the tree is yours: inspect it, or, in Rust, run ordered
        transform passes over it. The renderer then hands every code block to your hooks.
      </p>
      <MonoTerms
        className="mono-wide"
        items={[
          { term: "allocator", description: "Arena allocation and buffers" },
          { term: "ast", description: "Document nodes, source spans, visitors" },
          { term: "parser", description: "Markdown to tree" },
          { term: "renderer", description: "HTML output and hooks" },
        ]}
      />
    </MonoSection>
  );
}

export function Correctness() {
  const { commonmark, gfm } = ownAgreement;
  return (
    <MonoSection
      id="correctness"
      title={`${commonmark.passed} of ${commonmark.total}. Not one shortcut.`}
      standfirst="Speed is easy if you skip the hard cases. Every result here is backed by specification fixtures you can run yourself."
    >
      <MonoPassage
        note={{
          term: "Specification",
          children: "A document that says, example by example, what an input must produce.",
        }}
      >
        <p>
          In 2014, a group of developers set out to settle what Markdown actually means. The result,
          CommonMark, is a specification with hundreds of worked examples: this input, exactly this
          output. GitHub documented its own extensions, tables, strikethrough, task lists and
          autolinks, the same way.
        </p>
      </MonoPassage>
      <p>
        Ferromark&rsquo;s specification profile agrees with all {commonmark.total} CommonMark
        examples and all {gfm.total} GFM extension examples. It runs every example again with
        Windows and classic Mac line endings, {lineEndingVariants} variants, and the output must not
        change. It compares itself with cmark and cmark-gfm, the reference implementations, on their
        own test corpus.
      </p>
      <MonoPassage
        note={{
          side: "right",
          term: "Before the stopwatch",
          children: "Structure, source positions and HTML are compared first.",
        }}
      >
        <p>
          When a faster version of a routine is proposed, the document structure, source positions
          and HTML are compared before anything is timed. A change in behavior needs its own
          explanation and its own tests. It cannot arrive quietly as the price of going faster.
        </p>
      </MonoPassage>
      <MonoProofs items={proofs} />
      <AgreementTable />
    </MonoSection>
  );
}
