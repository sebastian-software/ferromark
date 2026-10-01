import { Mark, Section } from "ferramenta-family";
import { useId, useState } from "react";
import { Link } from "react-router";

import rocketStages from "../assets/rocket-stages.webp";
import samples from "../data/transform-samples.json";
import "../styles/transform-showcase.css";

const modules = [
  {
    id: "typography",
    title: "Typography",
    description: "Give prose the quotes, dashes, and spacing your language calls for.",
  },
  {
    id: "github",
    title: "GitHub references",
    description: "Turn issue numbers, mentions, and commits into links to your repository.",
  },
  {
    id: "emoji",
    title: "Emoji shortcodes",
    description: "Let familiar shortcodes become emoji, with the dictionary already included.",
  },
] as const;

type ModuleId = (typeof modules)[number]["id"];
type EnabledModules = Record<ModuleId, boolean>;

const processingName = "Afterburner";

const capabilities = [
  {
    id: "commonmark",
    title: "CommonMark",
    description: "The Markdown you know, with behavior checked against the specification.",
    to: "/guide/features",
  },
  {
    id: "gfm",
    title: "GFM",
    description: "The everyday extras: tables, task lists, strikethrough, and automatic links.",
    to: "/guide/features",
  },
  {
    id: "ffm",
    title: "FFM",
    description:
      "Publishing details that belong with your content: captions, attribution, and richer tables.",
    to: "/guide/ffm",
  },
  {
    id: "processing",
    title: processingName,
    description:
      "The finishing touches: polish typography, link GitHub references, expand emoji, or add your own Rust pass.",
    to: "/rust/transforms",
  },
] as const;

function StageRocket() {
  return (
    <img
      className="transform-rocket"
      src={rocketStages}
      width={792}
      height={864}
      loading="lazy"
      decoding="async"
      alt="A forged-steel rocket with four stacked stages: CommonMark, GitHub (GFM), Ferro (FFM), and Afterburner. Its fins are unmarked."
    />
  );
}

function CapabilityList() {
  return (
    <dl className="markdown-capabilities">
      {capabilities.map((capability) => (
        <div key={capability.id} data-stage={capability.id}>
          <dt>
            <Link to={capability.to}>
              {capability.title}
              <Mark name="arrow" size={16} />
            </Link>
          </dt>
          <dd>{capability.description}</dd>
        </div>
      ))}
    </dl>
  );
}

function TransformControls({
  enabled,
  previewId,
  onToggle,
}: {
  enabled: EnabledModules;
  previewId: string;
  onToggle: (id: ModuleId) => void;
}) {
  return (
    <div className="transform-module-list" role="group" aria-label="Choose your transforms">
      {modules.map((module) => (
        <button
          className="transform-module-control"
          key={module.id}
          type="button"
          aria-pressed={enabled[module.id]}
          aria-controls={previewId}
          onClick={() => {
            onToggle(module.id);
          }}
        >
          <span className="transform-module-copy">
            <strong>{module.title}</strong>
            <span>{module.description}</span>
          </span>
          <span className="transform-switch" aria-hidden="true">
            <Mark name="check" size={18} />
          </span>
        </button>
      ))}
    </div>
  );
}

function TransformPreview({ enabled, previewId }: { enabled: EnabledModules; previewId: string }) {
  const sampleKey =
    `${enabled.typography ? "1" : "0"}${enabled.github ? "1" : "0"}${enabled.emoji ? "1" : "0"}` as const;
  // All eight outputs come from the fixed source, rendered by Ferromark itself.
  // Visitor input and remote HTML never enter the preview.
  const html = samples.outputs[sampleKey];

  return (
    <div className="transform-workbench">
      <div className="transform-preview">
        <p className="transform-preview-label">Your author writes</p>
        <p className="transform-source">{samples.markdown}</p>
        <p className="transform-preview-label">Your reader sees</p>
        <div
          id={previewId}
          className="transform-output"
          aria-live="polite"
          aria-atomic="true"
          dangerouslySetInnerHTML={{ __html: html }}
        />
      </div>
      <p className="transform-custom-link">
        Make it your own.{" "}
        <Link to="/rust/transforms">
          Add your own Rust pass <Mark name="arrow" size={16} />
        </Link>
      </p>
    </div>
  );
}

function TransformFootnote() {
  return (
    <div className="transform-footnote">
      <p>
        {processingName} runs before rendering: built-in passes in Rust and Node.js, custom passes
        in Rust. A familiar Remark-style flow, with Ferromark’s own APIs.
      </p>
      <div className="transform-guide-links">
        <Link to="/rust/transforms">
          Rust transforms <Mark name="arrow" size={16} />
        </Link>
        <Link to="/node/pipelines#compose-native-transforms">
          Node.js transforms <Mark name="arrow" size={16} />
        </Link>
      </div>
    </div>
  );
}

function ProcessingDemo({
  enabled,
  onToggle,
}: {
  enabled: EnabledModules;
  onToggle: (id: ModuleId) => void;
}) {
  const previewId = useId();
  return (
    <div className="processing-demo">
      <div>
        <h3>{processingName} in action.</h3>
        <p className="processing-intro">
          The little publishing jobs, already built in. Pick a few finishing touches and see what
          reaches your reader.
        </p>
        <TransformControls enabled={enabled} previewId={previewId} onToggle={onToggle} />
      </div>
      <TransformPreview enabled={enabled} previewId={previewId} />
    </div>
  );
}

export function TransformShowcase() {
  const [enabled, setEnabled] = useState<EnabledModules>({
    typography: true,
    github: true,
    emoji: true,
  });
  const onToggle = (id: ModuleId) => {
    setEnabled((current) => ({ ...current, [id]: !current[id] }));
  };

  return (
    <Section
      id="features"
      className="ferromark-transforms"
      title="Four stages. One Markdown engine."
      intro={`Start with CommonMark. Add the everyday conveniences of GFM and the publishing features of FFM. Give the result its final polish with ${processingName}. All in one native engine.`}
    >
      <div className="transform-assembly">
        <div className="transform-visual">
          <StageRocket />
        </div>
        <CapabilityList />
      </div>
      <ProcessingDemo enabled={enabled} onToggle={onToggle} />
      <TransformFootnote />
    </Section>
  );
}
