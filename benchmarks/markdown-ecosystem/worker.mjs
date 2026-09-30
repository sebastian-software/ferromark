#!/usr/bin/env node
// The public Node string APIs run in one runtime; imports and I/O are untimed.
import { existsSync, readFileSync } from "node:fs";
import { createInterface } from "node:readline";
import { micromark } from "micromark";
import { gfmStrikethrough, gfmStrikethroughHtml } from "micromark-extension-gfm-strikethrough";
import { gfmTable, gfmTableHtml } from "micromark-extension-gfm-table";
import { gfmTaskListItem, gfmTaskListItemHtml } from "micromark-extension-gfm-task-list-item";
import { Renderer, toHtml } from "../../node/ferromark/index.mjs";

import { linuxLibc, nativeTarget } from "../../node/ferromark/native-target.mjs";

const [engine, profile, mode, ...paths] = process.argv.slice(2);
if (
  !["v2", "micromark"].includes(engine) ||
  !["commonmark", "gfm", "gfm-shared"].includes(profile) ||
  !["fresh", "reuse"].includes(mode) ||
  !paths.length
) {
  throw new Error("worker ENGINE PROFILE MODE INPUT...");
}
const target = nativeTarget(
  process.platform,
  process.arch,
  process.platform === "linux"
    ? linuxLibc(process.report?.getReport?.(), () => {
        try {
          return readFileSync("/usr/bin/ldd", "utf8");
        } catch {
          return "";
        }
      })
    : undefined,
);
if (
  engine === "v2" &&
  !existsSync(new URL(`../../node/ferromark/ferromark.${target}.node`, import.meta.url))
) {
  throw new Error(
    "Build the local Ferromark addon before measuring; a registry fallback is not a local-core comparison.",
  );
}
const gfm = profile !== "commonmark";
const inputs = paths.map((path) => readFileSync(path, "utf8"));
const options = {
  renderPolicy: "trusted",
  allowHtml: true,
  tables: gfm,
  strikethrough: gfm,
  taskLists: gfm,
  mergedTableCells: false,
  tableColgroup: false,
  tableColumnNames: false,
  tableAttributes: false,
  headingAttributes: false,
  headingIds: false,
  superscript: false,
  subscript: false,
  autolinkLiterals: false,
  disallowedRawHtml: false,
  footnotes: false,
  highlight: false,
  inlineFootnotes: false,
  allowLinkRefs: true,
  frontMatter: false,
  math: false,
  callouts: false,
  definitionLists: false,
  lineComments: false,
  cjkEmphasis: false,
  mdx: false,
};
const microOptions = {
  allowDangerousHtml: true,
  allowDangerousProtocol: true,
  extensions: gfm ? [gfmStrikethrough(), gfmTable(), gfmTaskListItem()] : [],
  htmlExtensions: gfm ? [gfmStrikethroughHtml(), gfmTableHtml(), gfmTaskListItemHtml()] : [],
};
const renderer = engine === "v2" && mode === "reuse" ? new Renderer(options) : null;
const render =
  engine === "micromark"
    ? (source) => micromark(source, microOptions)
    : renderer
      ? (source) => renderer.toHtml(source)
      : (source) => toHtml(source, options);

for await (const line of createInterface({ input: process.stdin })) {
  if (line === "quit") break;
  if (line === "verify") {
    inputs.forEach((source, index) =>
      process.stdout.write(`html ${index} ${Buffer.from(render(source)).toString("hex")}\n`),
    );
    process.stdout.write("done\n");
  } else if (line.startsWith("bench ")) {
    const budget = BigInt(line.slice(6));
    let iterations = 0;
    let checksum = 0;
    const start = process.hrtime.bigint();
    let elapsed;
    do {
      for (const source of inputs) checksum += render(source).length;
      iterations += 1;
      elapsed = process.hrtime.bigint() - start;
    } while (elapsed < budget);
    // Consume UTF-16 string lengths; conversion to UTF-8 is only for verification.
    process.stdout.write(`timing ${iterations} ${elapsed} ${checksum}\n`);
  } else throw new Error("unknown command");
}
