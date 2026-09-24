import { Link } from "react-router";

import {
  agreementDocuments,
  formatSpeed,
  nativeBenchmarkFigure,
  NativeBenchmarkFigures,
  nativeBenchmarks,
} from "../components/native-benchmarks";
import { PlatformChoices } from "../components/platform-choices";
import { version } from "../version";
import "../styles/home.css";

function LandingIntro() {
  return (
    <section className="landing-intro" aria-labelledby="landing-title">
      <p className="landing-kicker">Ferromark · The Ferramenta Markdown engine</p>
      <h1 id="landing-title">
        Markdown.
        <br />
        <span>Ready for your pipeline.</span>
      </h1>
      <p className="landing-lead">
        Ferromark turns Markdown into HTML and the metadata around it: front matter, heading IDs,
        footnotes, tables your CSS can address. A focused Rust engine that puts the CommonMark
        specification before speed, measures itself against the fastest native parsers before any
        number is published, and leaves the trust decisions to you, on purpose.
      </p>
      <p className="landing-aside">
        <strong>Version {version}.</strong> V2 is a breaking upgrade from v1.{" "}
        <a href="https://github.com/sebastian-software/ferromark/blob/main/docs/migration-v2.md">
          Read the migration guide
        </a>
        .
      </p>
      <PlatformChoices />
      <p className="landing-aside">
        Working in a shell? <Link to="/guide/cli">Use the command line</Link>.
      </p>
    </section>
  );
}

/**
 * The v1 and pulldown-cmark figures of every platform, for example "2.1× the speed of
 * Ferromark v1 and 2.6× the speed of pulldown-cmark on Apple Silicon, 2.0× and 2.4× on Linux x86-64".
 */
function storySpeeds(): string {
  return nativeBenchmarks.platforms
    .map((platform, index) => {
      const v1 = formatSpeed(nativeBenchmarkFigure(platform, "v1").fresh);
      const pulldown = formatSpeed(nativeBenchmarkFigure(platform, "pulldown-cmark").fresh);
      return index === 0
        ? `${v1} the speed of Ferromark v1 and ${pulldown} the speed of pulldown-cmark on ${platform.label}`
        : `${v1} and ${pulldown} on ${platform.label}`;
    })
    .join(", ");
}

const storyBeats = [
  {
    title: "Conformance first, speed second",
    text: (
      <>
        Ferromark v2 started from a fast core that strayed from the specification: corrupted line
        endings, five failing GFM examples, seven cmark discrepancies, reference definitions lost
        inside lists. Every one of them was closed before the first release candidate. Today all 652
        CommonMark examples agree, along with the GFM extension examples, 1,304 CR and CRLF
        variants, and the original cmark corpus. Where a reference and the specification disagree,
        the specification decides, and a decision record says why.
      </>
    ),
    to: "/guide/correctness",
    call: "Inspect the correctness checks",
  },
  {
    title: "Fast, and measured before it is said",
    text: (
      <>
        Same input, equivalent HTML, six native engines in one executable per platform:{" "}
        {storySpeeds()}, across {agreementDocuments()} real documents. Every report keeps its source
        revisions, flags, raw timing windows, and reproduction commands. A speed claim without a
        report does not ship.
      </>
    ),
    to: "/guide/benchmarks",
    call: "Read the measurements",
  },
  {
    title: "Trust is a decision, not a surprise",
    text: (
      <>
        Recognizing raw HTML and permission to emit it are separate choices. Rust passes raw HTML
        through until you turn on sanitizing; Node.js escapes it and filters unsafe URL schemes
        until you declare the source trusted. MDX syntax lands in the tree as data and is never
        executed.
      </>
    ),
    to: "/guide/rendering",
    call: "Choose your rendering policy",
  },
  {
    title: "The pieces publishing needs, nothing your app should own",
    text: (
      <>
        Front matter, heading IDs, footnotes, callouts, and inline tables of contents. Tables with
        classes, captions, merged cells, and named columns. Translation, templates, routing, and
        your highlighter stay yours, connected through rendering hooks.
      </>
    ),
    to: "/guide/features",
    call: "Explore the Markdown syntax",
  },
];

function StoryList() {
  return (
    <div className="landing-principle-list">
      {storyBeats.map((beat, index) => (
        <article key={beat.to}>
          <span className="landing-index" aria-hidden="true">
            {String(index + 1).padStart(2, "0")}
          </span>
          <div>
            <h3>{beat.title}</h3>
            <p>{beat.text}</p>
            <Link to={beat.to}>{beat.call} →</Link>
          </div>
        </article>
      ))}
    </div>
  );
}

function LandingStory() {
  return (
    <section className="landing-principles" aria-labelledby="story-title">
      <div className="landing-section-head">
        <p className="landing-kicker">What sets it apart</p>
        <h2 id="story-title">Built for the documents that get complicated.</h2>
        <p>
          Nested lists, reference links, tables with real content, the page that looked fine until
          an author added one more footnote. A Markdown engine earns its place there, and it has to
          prove it.
        </p>
      </div>
      <StoryList />
    </section>
  );
}

function LandingEvidence() {
  const sharedRunners = nativeBenchmarks.platforms.filter((platform) => platform.sharedRunner);
  return (
    <section className="landing-evidence" aria-labelledby="evidence-title">
      <div className="landing-section-head">
        <p className="landing-kicker">Measured throughput</p>
        <h2 id="evidence-title">Faster on real documents.</h2>
        <p>
          Ferromark v2 throughput relative to each engine on {agreementDocuments()} real documents,
          from short comments to long technical pages, that all five native engines render to
          equivalent HTML. Fresh parser state, measured separately on each platform; higher is
          faster.
        </p>
        <p className="landing-aside">
          Measured without profile-guided optimization.{" "}
          {sharedRunners.map((platform) => (
            <span key={platform.id}>
              The {platform.label} figures come from a shared CI runner: compare the ratios between
              engines, not absolute times.{" "}
            </span>
          ))}
          Reuse lifecycles, PGO builds, per-document timings, and reproduction:{" "}
          <Link to="/guide/benchmarks">Inspect the evidence →</Link>
        </p>
      </div>
      <NativeBenchmarkFigures />
    </section>
  );
}

const guardrails = [
  [
    "The core does one job.",
    "Parse Markdown, render HTML. Site assembly, translation, and syntax highlighting connect through explicit hooks and never move into the engine.",
  ],
  [
    "No semantic change without a record.",
    "Snapshot output and conformance baselines survive every optimization. An intended change to rendered output gets its own decision record and review, with the specification as the tie-breaker.",
  ],
  [
    "No unmeasured speed claims.",
    "One optimization per commit, output equality verified before timing, a control run before every round, and the patches that lost archived next to the ones that won.",
  ],
  [
    "Hostile input stays linear.",
    "Nesting is capped, quadratic shapes are bounded, and the pre-release review that found a regression repaired it before the release and documented what it cost.",
  ],
  [
    "Provenance travels with the code.",
    "Ferromark v2 starts from the MIT-licensed OX-Content core, imported by commit and checksum. Every comparison names the engines, revisions, and flags it measured.",
  ],
  [
    "Rust and Node.js share one contract.",
    "Syntax, trust policies, and output semantics are documented once and hold in both, so what you learn in one runtime carries over.",
  ],
] as const;

function GuardrailList() {
  return (
    <ul className="landing-guardrail-list">
      {guardrails.map(([rule, detail]) => (
        <li key={rule}>
          <strong>{rule}</strong> {detail}
        </li>
      ))}
    </ul>
  );
}

function LandingGuardrails() {
  return (
    <section className="landing-guardrails" aria-labelledby="guardrails-title">
      <div className="landing-section-head">
        <p className="landing-kicker">How we work</p>
        <h2 id="guardrails-title">Guardrails we keep.</h2>
        <p>
          Speed is easy to claim and easy to lose. These rules keep the engine correct while it gets
          faster, and they are written down in the repository, not only here.
        </p>
      </div>
      <GuardrailList />
      <p className="landing-aside">
        The rules live in{" "}
        <a href="https://github.com/sebastian-software/ferromark/blob/main/AGENTS.md">AGENTS.md</a>,
        the{" "}
        <a href="https://github.com/sebastian-software/ferromark/tree/main/docs/decisions">
          decision records
        </a>
        , and the{" "}
        <a href="https://github.com/sebastian-software/ferromark/tree/main/docs/reports">
          measurement reports
        </a>
        .
      </p>
    </section>
  );
}

function LandingReference() {
  return (
    <section className="landing-reference" aria-labelledby="reference-title">
      <p className="landing-kicker">One engine. A shared contract.</p>
      <h2 id="reference-title">Learn the behavior once.</h2>
      <p>
        Rust and Node.js have their own guides and APIs. Markdown syntax, trust policies, and output
        semantics live in one shared reference.
      </p>
      <div className="landing-reference-links">
        <Link to="/guide/configuration">
          <strong>Choose your syntax</strong>
          <span>Defaults, dialects, and optional extensions →</span>
        </Link>
        <Link to="/guide/pipelines">
          <strong>Plan your pipeline</strong>
          <span>Metadata, navigation, links, and presentation →</span>
        </Link>
        <Link to="/guide/benchmarks">
          <strong>Inspect the evidence</strong>
          <span>Measured workloads, trade-offs, and reproduction →</span>
        </Link>
      </div>
    </section>
  );
}

export default function HomePage() {
  return (
    <main className="landing">
      <LandingIntro />
      <LandingStory />
      <LandingEvidence />
      <LandingGuardrails />
      <LandingReference />
    </main>
  );
}
