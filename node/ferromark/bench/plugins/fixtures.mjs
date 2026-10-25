const fence = (code) => `\n\n\`\`\`mermaid\n${code}\n\`\`\`\n`;
const small = "flowchart LR\nA[Read] --> B[Parse]\nB --> C[Render]\nC --> D[Done]";
const medium = `flowchart TD\n${Array.from({ length: 30 }, (_, i) => `N${i}[Step ${i}] --> N${i + 1}[Step ${i + 1}]`).join("\n")}`;
const prose = Array.from(
  { length: 20 },
  (_, i) =>
    `## Section ${i}\n\nMarkdown with **strong text**, [a link](https://example.com), and ordinary code.\n\n\`\`\`rust\nlet value = ${i};\n\`\`\``,
).join("\n\n");
export const fixtures = {
  ordinary: { source: prose, diagrams: 0 },
  small: { source: fence(small), diagrams: 1 },
  medium: { source: fence(medium), diagrams: 1 },
  mixed: {
    source: prose + fence(small) + fence('pie\n"Ready" : 3\n"Waiting" : 1') + fence(small),
    diagrams: 3,
  },
};
