import { useId, useState } from "ardo/react";
import { ArdoLink as Link } from "ardo/runtime";
import { Mark } from "ferramenta-family";
import { FerraExample, FerraFeatureGrid, FerraToggle } from "ferramenta-family/ferra";

import samples from "../data/transform-samples.json";
import { LandingSection as Section } from "./landing-section";

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
    <div role="group" aria-label="Choose your transforms">
      {modules.map((module) => (
        <FerraToggle
          key={module.id}
          pressed={enabled[module.id]}
          label={module.title}
          description={module.description}
          aria-controls={previewId}
          onClick={() => {
            onToggle(module.id);
          }}
        />
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
    <div
      id={previewId}
      className="transform-output"
      aria-live="polite"
      aria-atomic="true"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

function TransformFootnote() {
  return (
    <>
      <p>
        {processingName} runs before rendering: built-in passes in Rust and Node.js, custom passes
        in Rust. A familiar Remark-style flow, with Ferromark’s own APIs.
      </p>
      <Link to="/rust/transforms">Rust transforms</Link>
      <Link to="/node/pipelines#compose-native-transforms">Node.js transforms</Link>
    </>
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
    <FerraExample
      title={`${processingName} in action.`}
      introduction="The little publishing jobs, already built in. Pick a few finishing touches and see what reaches your reader."
      source={samples.markdown}
      controls={<TransformControls enabled={enabled} previewId={previewId} onToggle={onToggle} />}
      footer={<TransformFootnote />}
    >
      <TransformPreview enabled={enabled} previewId={previewId} />
      <p>
        Make it your own. <Link to="/rust/transforms">Add your own Rust pass</Link>
      </p>
    </FerraExample>
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
      title="See what reaches your reader."
      intro="Try the finishing touches on a real Markdown sample. Switch typography, GitHub references and emoji on or off to see the result."
    >
      <ProcessingDemo enabled={enabled} onToggle={onToggle} />
      <FerraFeatureGrid
        items={capabilities.map((capability) => ({
          heading: capability.title,
          text: (
            <>
              <p>{capability.description}</p>
              <Link to={capability.to}>
                Read the guide <Mark name="arrow" size={16} />
              </Link>
            </>
          ),
        }))}
      />
    </Section>
  );
}
