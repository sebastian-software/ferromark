// Plain-text tables for the terminal. The JSON output carries the raw rounds.

import { apiNames, candidateNames, median, parts } from "./model.mjs";

function nanoseconds(value) {
  return Math.abs(value) >= 100 ? value.toFixed(0) : value.toFixed(1);
}

function percent(value) {
  return `${(value * 100).toFixed(0)}%`;
}

function table(headers, rows) {
  const cells = [headers, ...rows].map((row) => row.map(String));
  const widths = headers.map((_, column) => Math.max(...cells.map((row) => row[column].length)));
  const line = (row) =>
    row
      .map((cell, column) =>
        column === 0 ? cell.padEnd(widths[column]) : cell.padStart(widths[column]),
      )
      .join("  ");
  console.log(line(cells[0]));
  console.log(widths.map((width) => "-".repeat(width)).join("  "));
  for (const row of cells.slice(1)) console.log(line(row));
}

export function printEnvironment(environment, settings, documentCount) {
  console.log("# N-API boundary measurement\n");
  console.log(
    `Node ${environment.node} (V8 ${environment.v8}) on ${environment.platform}/${environment.arch}, ${environment.cpu}`,
  );
  console.log(`Addon: ${environment.addon}`);
  const widened = settings.twoByte ? " (inputs widened to two-byte)" : "";
  console.log(
    `Documents: ${documentCount}${widened}; rounds ${settings.rounds}, ` +
      `batch ${settings.batchMs} ms, warmup ${settings.warmupMs} ms per lane`,
  );
  if (!environment.exposeGc) console.log("Hint: node --expose-gc collects between documents.");
}

export function printFixed({ medians }) {
  console.log("\n## Fixed per-call costs (ns)\n");
  const rows = [
    ["free-function call floor (boundaryNoop)", medians.noop],
    ["method call floor (BoundaryProbe.noop)", medians.methodNoop],
    ["options: undefined, over the call floor", medians.optionsNone - medians.noop],
    ["options: {}, over the call floor", medians.optionsEmpty - medians.noop],
    ["options: { renderPolicy }, over the call floor", medians.optionsTrusted - medians.noop],
    ["toHtml('')", medians.toHtmlEmpty],
    ["toHtml('', {})", medians.toHtmlEmptyOptions],
    ["new Renderer()", medians.rendererConstruct],
    ["Renderer::new plus drop inside Rust", medians.coreSetup],
  ];
  table(
    ["cost", "ns"],
    rows.map(([label, value]) => [label, nanoseconds(value)]),
  );
}

function timeRow(result) {
  const { medians } = result;
  return [
    result.name,
    result.inputBytes,
    `${result.content}/${result.representation}`,
    `${result.outputBytes}${result.outputAscii ? "" : "*"}`,
    ...[
      medians.toHtml,
      medians.rendererToHtml,
      medians.toHtmlBuffer,
      medians.len - medians.noop,
      medians.html - medians.methodNoop,
      medians.coreFresh - medians.coreSetup,
      medians.coreReuse,
      medians.coreSetup,
    ].map((value) => nanoseconds(value)),
  ];
}

export function printDocuments(results) {
  console.log("\n## Per document: time per call (ns)\n");
  table(
    [
      "document",
      "in B",
      "input",
      "out B",
      "toHtml",
      "reuse",
      "buffer",
      "in conv",
      "out conv",
      "core fresh",
      "core reuse",
      "setup",
    ],
    results.map((result) => timeRow(result)),
  );
  console.log("\ninput: content/V8 storage. *: non-ASCII HTML.");
  console.log("\n## Per document: share of each call (input/core/output/fixed/residual, %)\n");
  table(
    ["document", "in B", ...apiNames],
    results.map((result) => [
      result.name,
      result.inputBytes,
      ...apiNames.map((api) => shareCell(result.attribution[api].share)),
    ]),
  );
}

function shareCell(share) {
  return parts.map((part) => Math.round(share[part] * 100)).join("/");
}

function groupRow(group, members, api) {
  const columns = [...parts, "boundary"];
  const shares = Object.fromEntries(
    columns.map((part) => [
      part,
      median(members.map((result) => result.attribution[api].share[part])),
    ]),
  );
  const totalNs = median(members.map((result) => result.attribution[api].ns.total));
  const row = [
    group,
    members.length,
    nanoseconds(totalNs),
    ...columns.map((c) => percent(shares[c])),
  ];
  return { data: { documents: members.length, shares, totalNs }, row };
}

function groupTable(groups, api) {
  const data = {};
  const rows = [];
  for (const [group, members] of groups) {
    const result = groupRow(group, members, api);
    data[group] = result.data;
    rows.push(result.row);
  }
  table(["group", "docs", "ns", ...parts, "N-API"], rows);
  return data;
}

function candidateCell(members, name) {
  const eligible = members.filter((result) => name in result.candidates);
  if (eligible.length === 0) return { cell: "n/a" };
  const savedNs = median(eligible.map((result) => result.candidates[name].savedNs));
  const relative = median(eligible.map((result) => result.candidates[name].relative));
  return {
    cell: `${nanoseconds(savedNs)} (${percent(relative)})`,
    data: { documents: eligible.length, relative, savedNs },
  };
}

function candidateTable(groups) {
  const data = {};
  const rows = [];
  for (const [group, members] of groups) {
    const cells = candidateNames.map((name) => candidateCell(members, name));
    data[group] = Object.fromEntries(
      candidateNames.map((name, index) => [name, cells[index].data]),
    );
    rows.push([group, ...cells.map(({ cell }) => cell)]);
  }
  table(["group", ...candidateNames], rows);
  console.log(
    "\nPaired per round: current path minus candidate, as a share of the baseline call.\n" +
      "singlePassInput has shipped: napi-rs's String conversion minus the exports' single pass.\n" +
      "latin1Output covers ASCII HTML only; externalOutput uses the current path for non-ASCII HTML.",
  );
  return data;
}

// Groups in a stable order: `order` compares [key, members] entries.
function groupBy(results, key, order) {
  const groups = new Map();
  for (const result of results) {
    const value = key(result);
    if (!groups.has(value)) groups.set(value, []);
    groups.get(value).push(result);
  }
  return new Map([...groups].toSorted(order));
}

const smallestInput = (members) => Math.min(...members.map((result) => result.inputBytes));

/** Prints the grouped summaries and returns them for the JSON output. */
export function printGroups(results) {
  const bySize = groupBy(
    results,
    (result) => result.sizeBin,
    ([, left], [, right]) => smallestInput(left) - smallestInput(right),
  );
  const byRepresentation = groupBy(
    results,
    (result) => `${result.content}/${result.representation}`,
    ([left], [right]) => left.localeCompare(right),
  );
  const summary = { byRepresentation: {}, bySize: {} };
  for (const api of apiNames) {
    console.log(`\n## ${api}: median share per size bin (equal document weight)\n`);
    summary.bySize[api] = groupTable(bySize, api);
  }
  for (const api of ["toHtml", "Renderer.toHtml"]) {
    console.log(`\n## ${api}: median share per input representation\n`);
    summary.byRepresentation[api] = groupTable(byRepresentation, api);
  }
  console.log("\n## Candidates: median saving per call and size bin (ns, % of the call)\n");
  summary.candidates = candidateTable(bySize);
  return summary;
}
