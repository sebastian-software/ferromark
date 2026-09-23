// Paired Node-level benchmark: Renderer.toHtml (reused) and toHtml (one-shot)
// over the broad corpus, alternating two addon builds in separate child
// processes. usage: node node-bench.mjs <corpus.json> <addonA.node> <addonB.node> [rounds]
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";

const [, script, corpusPath, addonA, addonB, roundsArg, worker] = process.argv;

if (worker) {
  // Child: run one addon, print ms per sweep for each lane.
  const require = createRequire(import.meta.url);
  const addon = require(worker === "A" ? addonA : addonB);
  const corpus = JSON.parse(readFileSync(corpusPath, "utf8"));
  const docs = corpus.cases.filter((c) => c.suite === "broad").map((c) => c.input);
  // Equal byte volume per document, as the Rust driver does.
  const reps = docs.map((d) => Math.max(1, Math.floor(200_000 / Math.max(1, d.length))));
  const lanes = {
    reuse: () => {
      const r = new addon.Renderer();
      let n = 0;
      for (let i = 0; i < docs.length; i++) for (let k = 0; k < reps[i]; k++) n += r.toHtml(docs[i]).length;
      return n;
    },
    oneshot: () => {
      let n = 0;
      for (let i = 0; i < docs.length; i++) for (let k = 0; k < reps[i]; k++) n += addon.toHtml(docs[i]).length;
      return n;
    },
  };
  const out = {};
  for (const [name, lane] of Object.entries(lanes)) {
    lane(); // warm-up
    const times = [];
    for (let s = 0; s < 5; s++) {
      const t0 = process.hrtime.bigint();
      lane();
      times.push(Number(process.hrtime.bigint() - t0) / 1e6);
    }
    times.sort((a, b) => a - b);
    out[name] = times[2];
  }
  process.stdout.write(JSON.stringify(out));
  process.exit(0);
}

const rounds = Number(roundsArg ?? 8);
const ratios = { reuse: [], oneshot: [] };
for (let r = 0; r < rounds; r++) {
  const order = r % 2 === 0 ? ["A", "B"] : ["B", "A"];
  const res = {};
  for (const w of order) {
    const p = spawnSync(process.execPath, [script, corpusPath, addonA, addonB, "0", w], { encoding: "utf8" });
    if (p.status !== 0) throw new Error(p.stderr);
    res[w] = JSON.parse(p.stdout);
  }
  for (const lane of Object.keys(ratios)) ratios[lane].push(res.A[lane] / res.B[lane]);
  console.log(`round ${r}: A ${JSON.stringify(res.A)} B ${JSON.stringify(res.B)}`);
}
for (const [lane, xs] of Object.entries(ratios)) {
  const s = [...xs].sort((a, b) => a - b);
  console.log(`${lane}: median A/B ${s[Math.floor(s.length / 2)].toFixed(3)} (B faster if > 1) rounds ${xs.map((x) => x.toFixed(3)).join(" ")}`);
}
