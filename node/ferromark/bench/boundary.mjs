// Splits a Node.js `toHtml` call into N-API boundary and Rust core time.
//
// Needs an addon built with the `boundary-bench` Cargo feature; README.md next
// to this file has the build command, the measurement model and how to read
// the output. Every lane runs in that one addon, so the public exports and the
// diagnostic exports are timed from the same binary.

import { readdirSync, statSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { cpus } from "node:os";
import path from "node:path";
import { parseArgs } from "node:util";

import { verify, verifyFixed } from "./boundary/checks.mjs";
import { loadDocuments } from "./boundary/corpus.mjs";
import { loadFacade } from "./boundary/facade.mjs";
import { documentLanes, documentState, fixedLanes, lastResult } from "./boundary/lanes.mjs";
import {
  attribution,
  byRound,
  bytesInput,
  candidates,
  mapValues,
  median,
  optionComparison,
} from "./boundary/model.mjs";
import { printDocuments, printEnvironment, printFixed, printGroups } from "./boundary/report.mjs";
import { measureLanes } from "./boundary/timing.mjs";

const repoRoot = path.resolve(import.meta.dirname, "..", "..", "..");

const settings = readSettings();
const addon = loadAddon(settings.addon);
const facade = await loadFacade(addon, settings.facade);
const documents = loadDocuments(settings);
const environment = {
  addon: addon.file,
  arch: process.arch,
  cpu: cpus()[0]?.model ?? "unknown",
  exposeGc: typeof globalThis.gc === "function",
  facade: settings.facade,
  node: process.version,
  platform: process.platform,
  v8: process.versions.v8,
};

printEnvironment(environment, settings, documents.length);
const fixed = await measureFixed();
printFixed(fixed);
const results = [];
for (const document of documents) {
  results.push(await measureDocument(document));
  process.stderr.write(`measured ${results.length}/${documents.length} ${document.name}\n`);
}
printDocuments(results);
const groups = printGroups(results);
if (settings.json) {
  const report = { environment, fixed, groups, results, schema: 1, settings };
  writeFileSync(settings.json, `${JSON.stringify(report, null, 2)}\n`);
  console.log(`\nWrote ${settings.json}`);
}
// Reading the sink once keeps every lane's result observable.
if (lastResult() === Symbol.for("unreachable")) process.exitCode = 1;

function readSettings() {
  const { values } = parseArgs({
    options: {
      addon: { default: path.join(repoRoot, "target", "boundary-bench"), type: "string" },
      "batch-ms": { type: "string" },
      corpus: {
        default: path.join(
          repoRoot,
          "docs/reports/2026-09-14-optimization-rounds/broad-corpus.json.gz",
        ),
        type: "string",
      },
      facade: { default: path.resolve(import.meta.dirname, ".."), type: "string" },
      filter: { type: "string" },
      json: { type: "string" },
      rounds: { type: "string" },
      smoke: { default: false, type: "boolean" },
      "two-byte": { default: false, type: "boolean" },
      "warmup-ms": { type: "string" },
    },
  });
  const { smoke } = values;
  return {
    addon: path.resolve(values.addon),
    batchMs: positive(values["batch-ms"], smoke ? 0.25 : 2),
    corpus: values.corpus,
    facade: path.resolve(values.facade),
    filter: values.filter,
    json: values.json,
    rounds: Math.round(positive(values.rounds, smoke ? 3 : 15)),
    smoke,
    twoByte: values["two-byte"],
    warmupMs: positive(values["warmup-ms"], smoke ? 2 : 20),
  };
}

function positive(text, fallback) {
  if (text === undefined) return fallback;
  const value = Number(text);
  if (!Number.isFinite(value) || value <= 0) throw new Error(`Expected a positive number: ${text}`);
  return value;
}

function loadAddon(location) {
  let file = location;
  if (statSync(location).isDirectory()) {
    const addons = readdirSync(location).filter((name) => name.endsWith(".node"));
    if (addons.length !== 1) {
      throw new Error(`Expected exactly one .node addon in ${location}, found ${addons.length}`);
    }
    file = path.join(location, addons[0]);
  }
  const native = createRequire(import.meta.url)(file);
  if (typeof native.boundaryNoop !== "function") {
    throw new TypeError(`${file} lacks the boundary-bench exports; see bench/README.md`);
  }
  return { file, native };
}

async function measureFixed() {
  verifyFixed(addon.native, facade);
  const { iterations, rounds } = await measureLanes(fixedLanes(addon.native, facade), settings);
  return {
    comparison: optionComparison(rounds),
    iterations,
    medians: mapValues(rounds, median),
    rounds,
  };
}

async function measureDocument(document) {
  globalThis.gc?.();
  const state = documentState(addon.native, document);
  verify(document, state);
  const { iterations, rounds } = await measureLanes(documentLanes(state), settings);
  const perRound = byRound(rounds);
  return {
    attribution: attribution(perRound),
    bytes: bytesInput(perRound),
    candidates: candidates(perRound),
    category: document.category,
    content: document.content,
    inputBytes: document.inputBytes,
    iterations,
    medians: mapValues(rounds, median),
    name: document.name,
    outputAscii: state.outputAscii,
    outputBytes: state.outputBytes,
    outputRepresentation: state.outputRepresentation,
    representation: document.representation,
    rounds,
    sizeBin: document.sizeBin,
  };
}
