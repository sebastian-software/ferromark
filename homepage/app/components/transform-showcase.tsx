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
    description: "Real quotes, real dashes, and the spacing your language expects.",
  },
  {
    id: "github",
    title: "GitHub references",
    description: "Issue numbers, mentions and commits become links to your repository.",
  },
  {
    id: "emoji",
    title: "Emoji shortcodes",
    description: "Type :rocket: and get the real thing. The dictionary ships with it.",
  },
] as const;

type ModuleId = (typeof modules)[number]["id"];
type EnabledModules = Record<ModuleId, boolean>;

const processingName = "Afterburner";

const capabilities = [
  {
    id: "commonmark",
    title: "CommonMark",
    description:
      "The Markdown you already know, checked against every example in the specification.",
    to: "/guide/features",
  },
  {
    id: "gfm",
    title: "GFM",
    description: "The extras you reach for daily: tables, task lists, strikethrough and autolinks.",
    to: "/guide/features",
  },
  {
    id: "ffm",
    title: "FFM",
    description:
      "Ferromark Flavored Markdown: captions, attributions and richer tables, written right into your text.",
    to: "/guide/ffm",
  },
  {
    id: "processing",
    title: processingName,
    description: "The finishing pass: typography, GitHub links, emoji, or a Rust pass of your own.",
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
        {processingName} runs on the document tree before rendering. Built-in passes work in Rust
        and Node.js, your own passes in Rust. If you know Remark, you will feel at home.
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
      title={`${processingName}, live.`}
      introduction="One Markdown source, eight possible results. Flip the switches and watch the same text grow up."
      source={samples.markdown}
      controls={<TransformControls enabled={enabled} previewId={previewId} onToggle={onToggle} />}
      footer={<TransformFootnote />}
    >
      <TransformPreview enabled={enabled} previewId={previewId} />
      <p>
        Missing something? <Link to="/rust/transforms">Write your own pass in Rust</Link>
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
      title="Watch your words get better."
      intro="This is real Ferromark output, not a mockup. Switch typography, GitHub references and emoji on and off, and see exactly what your readers will get."
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
