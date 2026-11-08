import type { ArdoMetaFunction } from "ardo/runtime";

import { Fragment } from "ardo/react";
import { ArdoLink as Link } from "ardo/runtime";
import {
  MonoAction,
  MonoClosing,
  MonoColophon,
  MonoEngraving,
  MonoOpening,
} from "ferramenta-family/monografie";
import { WORKSHOP } from "ferramenta-family/registry";

import {
  AtWork,
  Code,
  Ffm,
  Mdx,
  Runtimes,
  Trust,
  Typography,
} from "../components/feature-sections";
import { libraryCount, ownAgreement, suiteLabels } from "../components/homepage-data";
import { Correctness, Explainer, Inside, Speed } from "../components/story-sections";
import { nodeVersion, releaseLine, rustVersion } from "../version";

export const handle = { layout: "bare" };
export const meta: ArdoMetaFunction = () => [
  { title: "Ferromark — Markdown, rendered exactly" },
  {
    name: "description",
    content:
      "Ferromark turns Markdown into clean, safe HTML: a native Rust engine with Node.js bindings that agrees with every CommonMark example and led every library in our comparison.",
  },
];

function Actions() {
  return (
    <>
      <MonoAction href="/rust/getting-started" primary detail="cargo add ferromark">
        Start with Rust
      </MonoAction>
      <MonoAction href="/node/getting-started" detail="npm install ferromark">
        Start with Node.js
      </MonoAction>
    </>
  );
}

function Opening() {
  const { commonmark, gfm } = ownAgreement;
  return (
    <MonoOpening
      product="ferromark"
      instrumentLabel="Ferromark's instrument: a steel registration bridge holding three thin leaves in alignment, an engraved source sheet, a ruled output sheet with a small table and a rust-brown metadata index."
      kicker={`Ferromark ${releaseLine}, a Markdown renderer for Rust and Node.js`}
      title="Markdown, rendered exactly."
      subtitle="And faster than every engine we measured."
      standfirst="Ferromark turns plain-text Markdown into clean, safe HTML for documentation sites, publishing tools and applications. A native Rust engine with bindings for Node.js, it agrees with every example in the CommonMark specification and led all twenty libraries in our comparison."
      actions={<Actions />}
      facts={[
        {
          value: (
            <>
              <em>{commonmark.passed}</em> of {commonmark.total}
            </>
          ),
          label: `${suiteLabels.commonmark} examples`,
        },
        {
          value: (
            <>
              <em>{gfm.passed}</em> of {gfm.total}
            </>
          ),
          label: "GFM extension examples",
        },
        {
          value: (
            <>
              <em>{libraryCount}</em> of {libraryCount}
            </>
          ),
          label: "measured libraries it led",
        },
        { value: `Rust ${rustVersion}+`, label: `Node.js ${nodeVersion}+, MIT license` },
      ]}
    />
  );
}

// Keys are written out: minified builds do not keep function names.
const chapters = [
  { key: "explainer", Chapter: Explainer },
  { key: "speed", Chapter: Speed },
  { key: "inside", Chapter: Inside },
  { key: "correctness", Chapter: Correctness },
  { key: "ffm", Chapter: Ffm },
  { key: "typography", Chapter: Typography },
  { key: "trust", Chapter: Trust },
  { key: "code", Chapter: Code },
  { key: "mdx", Chapter: Mdx },
  { key: "runtimes", Chapter: Runtimes },
  { key: "work", Chapter: AtWork },
];

function Origin() {
  return (
    <section aria-label="Origin">
      <div className="mono-axis">
        <MonoColophon>
          Ferromark carries forward the features and experience of Ferromark v1 and builds on the
          MIT-licensed OX-Content core. Its source provenance records the imported revision and
          keeps the attribution. It is one of the Ferramenta engines, made by {WORKSHOP.name} in{" "}
          {WORKSHOP.place}.
        </MonoColophon>
      </div>
    </section>
  );
}

export default function HomePage() {
  return (
    <>
      <Opening />
      {chapters.map(({ key, Chapter }, index) => (
        <Fragment key={key}>
          <MonoEngraving seed={index + 1} />
          <Chapter />
        </Fragment>
      ))}
      <MonoEngraving seed={chapters.length + 1} />
      <Origin />
      <MonoClosing
        title="Your next document deserves this."
        standfirst="Choose your runtime and follow the getting-started guide. Rust gives you the parser and document tree directly, Node.js a typed JavaScript API on the same native engine."
        actions={<Actions />}
        links={
          <>
            <a href="https://github.com/sebastian-software/ferromark">GitHub repository</a>
            <Link to="/guide/getting-started">Documentation</Link>
            <a href={WORKSHOP.consulting}>Integration and support</a>
          </>
        }
      />
    </>
  );
}
