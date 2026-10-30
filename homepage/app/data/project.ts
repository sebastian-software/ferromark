import { family } from "ferramenta-family/registry";

export const tool = family.find((member) => member.name === "ferromark");
if (!tool) throw new Error("Missing Ferromark registry entry");

export const contents = [
  { href: "#overview", label: "The engine", description: "Rust and Node.js, one native core" },
  { href: "#features", label: "Four stages", description: "CommonMark, GFM, FFM and Afterburner" },
  { href: "#ffm", label: "Publishing features", description: "More context in your documents" },
  {
    href: "#document-analysis",
    label: "Document analysis",
    description: "References, text and static Markdown",
  },
  { href: "#evidence", label: "Benchmarks", description: "Platforms, comparisons and sources" },
  { href: "#conformance", label: "Correctness", description: "The contract and its evidence" },
  { href: "#start", label: "Start building", description: "Choose your runtime" },
];
