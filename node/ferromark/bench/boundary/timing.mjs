// Paired, interleaved batch timing with process.hrtime.bigint.

import { setImmediate as nextTurn } from "node:timers/promises";

// Loop lanes run many core iterations in one call. The named per-call lane of
// the same round (the call plus its input conversion) is subtracted before the
// batch is divided by the iteration count.
export const loopCorrection = { coreFresh: "len", coreReuse: "len", coreSetup: "noop" };
const minimumLoopIterations = 8;

function time(run, k) {
  const start = process.hrtime.bigint();
  run(k);
  return Number(process.hrtime.bigint() - start);
}

// Grows the batch until it lasts a quarter of the target, keeps it running
// for the warmup budget, then sizes it from the warm time.
async function calibrate(name, run, { batchMs, warmupMs }) {
  const target = batchMs * 1e6;
  const floor = name in loopCorrection ? minimumLoopIterations : 1;
  let k = floor;
  let elapsed = time(run, k);
  while (elapsed < target / 4) {
    const estimate = Math.ceil((k * target) / 4 / Math.max(elapsed, 1));
    k = Math.min(k * 16, Math.max(k * 2, estimate));
    elapsed = time(run, k);
  }
  const warmupEnd = process.hrtime.bigint() + BigInt(Math.round(warmupMs * 1e6));
  while (process.hrtime.bigint() < warmupEnd) {
    elapsed = time(run, k);
    await nextTurn();
  }
  return Math.max(floor, Math.round((k * target) / Math.max(elapsed, 1)));
}

// One round: every lane once, in the given order. N-API finalizers of external
// buffers run from the event loop, so the loop turns after every batch,
// outside the timed region, which also keeps memory bounded.
async function runRound(lanes, order, iterations) {
  const elapsed = {};
  for (const name of order) {
    elapsed[name] = time(lanes[name], iterations[name]);
    await nextTurn();
  }
  return elapsed;
}

function perUnit(elapsed, iterations, name) {
  const correction = loopCorrection[name];
  const batch = correction
    ? elapsed[name] - elapsed[correction] / iterations[correction]
    : elapsed[name];
  return batch / iterations[name];
}

/**
 * Times every lane in `settings.rounds` paired rounds. The lane order rotates
 * each round and alternate rounds run it backwards.
 * @returns Per lane, the batch size and nanoseconds per unit for each round.
 */
export async function measureLanes(lanes, settings) {
  const names = Object.keys(lanes);
  const iterations = {};
  for (const name of names) iterations[name] = await calibrate(name, lanes[name], settings);
  const rounds = Object.fromEntries(names.map((name) => [name, []]));
  for (let round = 0; round < settings.rounds; round++) {
    const rotated = names.map((_, index) => names[(index + round) % names.length]);
    const order = round % 2 === 0 ? rotated : rotated.toReversed();
    const elapsed = await runRound(lanes, order, iterations);
    for (const name of names) rounds[name].push(perUnit(elapsed, iterations, name));
  }
  return { iterations, rounds };
}
