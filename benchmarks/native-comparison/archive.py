#!/usr/bin/env python3
"""Assemble one native comparison run into a docs/reports-style directory.

The native comparison workflow calls this after `run.py` and `report.py`; it
works the same way for a local run. It copies the retained evidence, rechecks
every timed window, compares the HTML with a reference report, and writes a
README.md and PROVENANCE.md whose figures are all computed from the archived
files next to them. It never edits a measurement.
"""

from __future__ import annotations

import argparse
from datetime import datetime
import gzip
import json
from pathlib import Path
import re
import shutil
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import prepare  # noqa: E402
import report as tables  # noqa: E402

COMPRESSED = ("corpus.json", "verification.json", "behavior.json", "samples.json")
PLAIN = ("run.json", "summary.json", "TABLES.md", "aggregates.json", "timings.csv")


def read(directory: Path, name: str):
    return tables.read(directory, name)


def write_gzip(path: Path, data: bytes) -> None:
    # A fixed header timestamp keeps the archive's checksums reproducible.
    path.write_bytes(gzip.compress(data, compresslevel=9, mtime=0))


def copy_results(results: Path, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    for name in PLAIN:
        shutil.copyfile(results / name, destination / name)
    for name in COMPRESSED:
        write_gzip(destination / (name + ".gz"), (results / name).read_bytes())


SYSTEMS = {"Darwin": ("macos", "macOS"), "Linux": ("linux", "Linux")}
MACHINES = {"x86_64": "x86-64", "amd64": "x86-64", "arm64": "arm64", "aarch64": "arm64"}


def platform_names(build: dict) -> tuple[str, str]:
    """Directory slug and prose label, for example ("linux-x86-64", "Linux x86-64")."""
    details = build.get("platform", {})
    slug, label = SYSTEMS.get(details.get("system"), ("unknown", "an unrecorded platform"))
    machine = MACHINES.get(details.get("machine", "").lower(), details.get("machine", "unknown"))
    return f"{slug}-{machine}", f"{label} {machine}"


def audit_windows(run: dict, samples: list, verification: dict) -> int:
    """Recheck every timed window: coverage, rotation, duration, and output checksum."""
    jobs = {job["name"]: job for job in run["jobs"]}
    width = 1 << 64
    seen = set()
    for sample in samples:
        identity = (sample["round"], sample["sample"], sample["case"], sample["mode"])
        assert identity not in seen, ("duplicate window", identity)
        seen.add(identity)
        assert sorted(sample["order"]) == sorted(run["engines"]), ("incomplete rotation", identity)
        for engine in run["engines"]:
            timing = sample[engine]
            expected = sum(len(verification[name]["outputs"][engine].encode())
                           for name in jobs[sample["case"]]["members"])
            assert timing["iterations"] > 0, ("empty window", identity, engine)
            assert timing["elapsed_ns"] >= run["window_ms"] * 1_000_000, ("short window", identity, engine)
            assert timing["checksum"] == timing["iterations"] * expected % width, ("checksum", identity, engine)
    complete = {(r, s, job, mode) for r in range(run["rounds"]) for s in range(run["samples"])
                for job in jobs for mode in run["modes"]}
    assert seen == complete, "missing timed windows"
    return len(samples) * len(run["engines"])


def registry_check(resolved_lock: Path, seed_lock: Path) -> dict:
    """Compare the registry packages Cargo resolved with the lock the build was seeded from."""
    resolved = prepare.registry_packages(resolved_lock)
    seed = prepare.registry_packages(seed_lock)
    added = sorted(f"{n}@{v}" for n, v in set(resolved) - set(seed))
    removed = sorted(f"{n}@{v}" for n, v in set(seed) - set(resolved))
    changed = sorted(f"{n}@{v}" for (n, v) in set(resolved) & set(seed) if resolved[n, v] != seed[n, v])
    return {
        "seed_lock": str(seed_lock),
        "seed_lock_sha256": prepare.sha(seed_lock),
        "resolved_lock_sha256": prepare.sha(resolved_lock),
        "registry_packages": len(resolved),
        "identical_to_seed": not (added or removed or changed),
        "added": added,
        "removed": removed,
        "checksum_changed": changed,
    }


def compare_outputs(verification: dict, reference: dict) -> dict:
    """Count archived HTML outputs that are byte-identical to a reference report."""
    shared = sorted(set(verification) & set(reference))
    engines = {}
    for engine in tables.ENGINES:
        different = [name for name in shared
                     if verification[name]["outputs"][engine] != reference[name]["outputs"][engine]]
        engines[engine] = {"identical": len(shared) - len(different), "different": different}
    classifications = [name for name in shared
                       if verification[name]["versus_v2"] != reference[name]["versus_v2"]]
    return {
        "documents_compared": len(shared),
        "only_in_this_run": sorted(set(verification) - set(reference)),
        "only_in_reference": sorted(set(reference) - set(verification)),
        "engines": engines,
        "classification_changes": classifications,
    }


def cpu_steal(before: dict, after: dict) -> float | None:
    """Share of all CPU time the hypervisor withheld between two Linux host records."""
    first, last = before.get("proc_stat_cpu"), after.get("proc_stat_cpu")
    if not isinstance(first, dict) or not isinstance(last, dict):
        return None
    # guest time is already counted in user time.
    fields = [f for f in ("user", "nice", "system", "idle", "iowait", "irq", "softirq", "steal") if f in first]
    total = sum(last[f] - first[f] for f in fields)
    return (last["steal"] - first["steal"]) / total if total > 0 else None


def main_scores(summary: list, verification: dict) -> tuple[set, set, dict]:
    six, five = tables.agreement_sets(verification)
    scores = {}
    for name, members, engines in (("six", six, tables.ENGINES), ("five", five, tables.CONFIGURABLE)):
        for mode in ("fresh", "reuse"):
            scores[name, mode] = tables.aggregate(summary, members, mode, engines)
    return six, five, scores


def main_table(six: set, five: set, scores: dict) -> str:
    lines = [
        f"| Engine | Fresh, {len(six)} agreeing across six | Reuse, same {len(six)} | "
        f"Fresh, {len(five)} agreeing across five | Reuse, same {len(five)} |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    for engine in tables.ENGINES:
        values = [scores[g, m].get(engine) for g, m in
                  (("six", "fresh"), ("six", "reuse"), ("five", "fresh"), ("five", "reuse"))]
        lines.append("| " + tables.LABELS[engine] + " | " + " | ".join(
            f"{v:.2f}×" if v is not None else "—" for v in values) + " |")
    return "\n".join(lines)


def round_comparisons(summary: list, run: dict, six: set, five: set) -> dict:
    """V2's throughput relative to OX (six-engine set) and v1 (five-engine set), per process round."""
    result = {}
    for engine, members in (("ox-content", six), ("v1", five)):
        by_mode = {}
        for mode in ("fresh", "reuse"):
            selected = [r for r in summary if r["case"] in members and r["mode"] == mode]
            by_mode[mode] = [tables.geomean(
                r["engines"][engine]["round_medians_ns"][i] / r["engines"]["v2"]["round_medians_ns"][i]
                for r in selected) for i in range(run["rounds"])] if selected else []
        result[engine] = {"documents": len(members), "v2_relative_throughput": by_mode}
    return result


def pgo_comparison(default: Path, optimized: Path, build_pgo: dict) -> dict:
    """Per-engine PGO speedup and v2's standing within each build, on the held-out half."""
    cases = {c["name"] for c in read(default, "corpus.json")["cases"]}
    default_summary = read(default, "summary.json")
    pgo_summary = read(optimized, "summary.json")
    verification = read(default, "verification.json")
    dmap = {(r["case"], r["mode"]): r for r in default_summary}
    pmap = {(r["case"], r["mode"]): r for r in pgo_summary}
    speedup = {}
    for engine in tables.ENGINES:
        speedup[engine] = {}
        for mode in ("fresh", "reuse"):
            ratios = [dmap[c, mode]["engines"][engine]["ns"] / pmap[c, mode]["engines"][engine]["ns"]
                      for c in sorted(cases)]
            speedup[engine][mode] = {"geomean": tables.geomean(ratios), "min": min(ratios), "max": max(ratios)}
    six, five = tables.agreement_sets(verification)
    standing = {}
    for engine in tables.ENGINES:
        if engine == "v2":
            continue
        members = (six if engine == "ox-content" else five) & cases
        engines = tables.ENGINES if engine == "ox-content" else tables.CONFIGURABLE
        row = {"documents": len(members)}
        for label, source in (("default", default_summary), ("pgo", pgo_summary)):
            for mode in ("fresh", "reuse"):
                row[f"{label}_{mode}"] = 1 / tables.aggregate(source, members, mode, engines)[engine] if members else None
        standing[engine] = row
    return {
        "held_out_documents": len(cases),
        "engine_profile_data": build_pgo["pgo"]["engine_profile_data"],
        "speedup_default_over_pgo_time": speedup,
        "v2_relative_throughput": standing,
    }


def harness_tests(log: Path | None) -> str:
    if log is None:
        return "not recorded"
    text = log.read_text()
    ran = re.search(r"Ran (\d+) tests?", text)
    if not ran or not re.search(r"^OK\b", text, re.MULTILINE):
        raise SystemExit(f"{log} does not record a passing unittest run")
    return f"{ran.group(1)} tests passed"


def duration(run: dict) -> str:
    start = datetime.strptime(run["host_before"]["time_utc"], "%Y-%m-%dT%H:%M:%SZ")
    end = datetime.strptime(run["host_after"]["time_utc"], "%Y-%m-%dT%H:%M:%SZ")
    seconds = int((end - start).total_seconds())
    return f"{seconds // 60}m{seconds % 60:02d}s"


def first_line(text: str) -> str:
    return text.strip().splitlines()[0] if text.strip() else "unknown"


def llvm_version(rustc: str) -> str:
    match = re.search(r"LLVM version: (\S+)", rustc)
    return match.group(1) if match else "unknown"


def percent(value: float | None) -> str:
    return "unavailable" if value is None else f"{value * 100:.2f}%"


def write_readme(out: Path, facts: dict) -> None:
    build, run = facts["build"], facts["run"]
    v2 = build["engines"]["ferromark_v2"]["revision"]
    v1 = build["engines"]["ferromark_v1"]["revision"]
    six, five, scores = facts["six"], facts["five"], facts["scores"]
    cases = facts["cases"]
    sizes = [c["byte_count"] for c in cases]
    rounds = facts["rounds"]
    round_lines = ["| V2 throughput relative to | Inputs | Fresh rounds | Reuse rounds |",
                   "| --- | ---: | --- | --- |"]
    for engine, row in rounds.items():
        cells = [", ".join(f"{v:.3f}×" for v in row["v2_relative_throughput"][m]) or "—"
                 for m in ("fresh", "reuse")]
        round_lines.append(f"| {tables.LABELS[engine]} | {row['documents']} | " + " | ".join(cells) + " |")
    spread = max((max(v) - min(v) for row in rounds.values()
                  for v in row["v2_relative_throughput"].values() if v), default=0.0)
    outputs = facts["outputs"]
    reference = facts["reference_name"]
    total_outputs = outputs["documents_compared"] * len(tables.ENGINES)
    identical = sum(e["identical"] for e in outputs["engines"].values())
    steal = facts["steal"]
    source = facts["source"]
    lines = [
        f"# Native comparison on {facts['platform_label']} — {facts['date']}",
        "",
        f"Ferromark **v2 `{v2[:10]}`** and the five pinned comparison engines (v1 `{v1[:7]}`, "
        "the original OX-Content core, md4c, pulldown-cmark, and Bun's native `bun_md`) were "
        f"measured on the frozen **{len(cases)} documents ({min(sizes):,}–{max(sizes):,} UTF-8 bytes)** "
        f"on one host: **{run['host_before']['cpu']}**, {source['host_summary']}. "
        "Each scored column uses the same input set and equivalent HTML for every included engine. "
        "**Speed relative to v2: higher is faster; v2 = 1.00×.**",
        "",
        main_table(six, five, scores),
        "",
        "The harness, syntax and renderer flags, sources, adapters, and allocator setup are the "
        "ones the Apple Silicon reports use; the few platform differences are listed in "
        "[PROVENANCE.md](PROVENANCE.md#platform-differences). OX has no score in the five-engine "
        "columns because its original renderer cannot disable heading IDs, callouts, or fence "
        "metadata cleanup.",
        "",
        "## What this run can and cannot claim",
        "",
        f"- **One host.** {source['origin']} All six engines ran in the same process rounds and "
        "rotating windows on that host, one worker at a time, so the ratios between engines "
        "describe that host's CPU.",
        "- **A shared machine.** A GitHub-hosted runner is a virtual machine on shared hardware. "
        "Neighbors, clock behavior, and the CPU model can change between runs, so absolute "
        "nanoseconds are not comparable with any other run, and another run may land on a "
        "different CPU. Compare engine ratios, not times.",
        (f"- **Hypervisor steal** was {percent(steal)} of all CPU time during the timed run "
         "(from `/proc/stat`, recorded before and after every process round in `run.json`)."
         if steal is not None else
         "- **Hypervisor steal** was not recorded on this host."),
        "- **Not a universal ranking** and not a statistical significance claim. Positions on "
        "x86-64 and Apple Silicon can differ because the engines' SIMD paths, the compilers' "
        "code generation, and the cache hierarchies differ; neither platform's figures describe "
        "the other.",
        "",
        "## Validation",
        "",
        "Direct comparison in each process round, using the same agreement sets:",
        "",
        *round_lines,
        "",
        f"These are round aggregates, not confidence intervals. The largest spread between the "
        f"{run['rounds']} rounds on any row above is {spread:.3f}.",
        "",
        f"All **{facts['windows']:,} timed windows** passed their output-length checksums and were "
        "rechecked from `samples.json.gz` when this directory was assembled. Fresh/reuse equality, "
        "repeated transitions, and exact pre/post-timing output checks passed for every engine. "
        f"Each of the {len(run['jobs'])} workloads ({len(cases)} documents and "
        f"{len(run['jobs']) - len(cases)} rotating batches) was measured in both lifecycles, with "
        f"{run['rounds']} process rounds, {run['samples']} windows of at least {run['window_ms']} ms "
        f"per round, and {run['warmup_ms']} ms per-engine warmup. The timed run took "
        f"{duration(run)}.",
        "",
        f"**{identical} of {total_outputs} HTML outputs** ({outputs['documents_compared']} documents × "
        f"{len(tables.ENGINES)} engines) are byte-identical to the Apple Silicon report "
        f"[`{reference}`](../{reference}/README.md), and "
        f"{outputs['documents_compared'] - len(outputs['classification_changes'])} of "
        f"{outputs['documents_compared']} agreement classifications are unchanged; details in "
        "[checks/commands.json](checks/commands.json).",
        "",
        f"Harness unit tests: {facts['tests']}. Registry resolution: "
        + ("all " if facts["registry"]["identical_to_seed"] else "**not** all ")
        + f"{facts['registry']['registry_packages']} registry packages are identical to the seed "
        "lock in name, version, and checksum.",
        "",
    ]
    if facts.get("pgo"):
        pgo = facts["pgo"]
        lines += [
            "## Profile-guided optimization, measured on the held-out half",
            "",
            f"`prepare.py --pgo` applied one recipe to every Rust engine and trained on the documents "
            f"the round-3 split reserves for training. Both executables were measured on the other "
            f"**{pgo['held_out_documents']} documents**, which no profile saw. Per-engine speedup is "
            "default-build time over PGO-build time, geometric mean over the held-out documents.",
            "",
            "| Engine | Profile data | Fresh | Reuse | Fresh range |",
            "| --- | --- | ---: | ---: | --- |",
        ]
        for engine in tables.ENGINES:
            row = pgo["speedup_default_over_pgo_time"][engine]
            kind = pgo["engine_profile_data"][engine].split(":")[0]
            lines.append(f"| {tables.LABELS[engine]} | {kind} | {row['fresh']['geomean']:.3f}× | "
                         f"{row['reuse']['geomean']:.3f}× | "
                         f"{row['fresh']['min']:.2f}–{row['fresh']['max']:.2f}× |")
        lines += [
            "",
            "A PGO row is compared only with PGO rows:",
            "",
            "| V2 throughput relative to | N | Default fresh | Default reuse | PGO fresh | PGO reuse |",
            "| --- | ---: | ---: | ---: | ---: | ---: |",
        ]
        for engine, row in pgo["v2_relative_throughput"].items():
            cells = [row[k] for k in ("default_fresh", "default_reuse", "pgo_fresh", "pgo_reuse")]
            lines.append(f"| {tables.LABELS[engine]} | {row['documents']} | "
                         + " | ".join("—" if v is None else f"{v:.2f}×" for v in cells) + " |")
        lines += [
            "",
            "md4c is C and is not PGO-built; Bun's engine gains through its Rust crates only. "
            + ("The held-out runs reproduce the same HTML from the default and the PGO executable."
               if facts["pgo_outputs_equal"] else
               "**The held-out runs did not reproduce the same HTML from both executables.**"),
            "",
        ]
    lines += [
        "## Files",
        "",
        "- [Full tables](TABLES.md), [per-document timings](timings.csv), [aggregates](aggregates.json), "
        "[summary](summary.json), [raw windows](samples.json.gz), [run metadata](run.json).",
        "- [Frozen inputs and attribution](corpus.json.gz), [all HTML](verification.json.gz), "
        "[executable option guards](behavior.json.gz).",
        "- [Build metadata](build.json), [dependency lock](Cargo.lock), [build log](build-log.txt), "
        "[host description](host.txt), [source audit](source-audit.json.gz), "
        "[checks](checks/commands.json), [restore script](restore.py) and [its record](restore.json).",
    ]
    if facts.get("pgo"):
        lines.append("- Held-out PGO evidence: [PGO build metadata](build-pgo.json), "
                     "[its build log](build-log-pgo.txt.gz), [comparison](pgo-comparison.json), "
                     "[default run](run-pgo-default.json) and [`pgo/default/`](pgo/default/), "
                     "[PGO run](run-pgo-pgo.json) and [`pgo/pgo/`](pgo/pgo/).")
    lines += [
        "- The harness that produced this run is preserved under [`harness/`](harness/); "
        "regenerate the tables with `python3 harness/report.py .`.",
        "",
        "Engine sources and corpus inputs retain their original licenses and attribution. This is "
        "Bun's native Markdown core, not its JavaScript API.",
        "",
    ]
    (out / "README.md").write_text("\n".join(lines))


def write_provenance(out: Path, facts: dict) -> None:
    build, run = facts["build"], facts["run"]
    platform = build["platform"]
    engines = build["engines"]
    source = facts["source"]
    registry = facts["registry"]
    audit = facts["source_audit"]
    darwin, linux = prepare.PLATFORMS["Darwin"], prepare.PLATFORMS["Linux"]
    rows = [
        ("C/C++ compiler", "clang, clang++ (Apple clang)", "clang, clang++ (runner default)"),
        ("C++ runtime", darwin["cxx_runtime_note"], linux["cxx_runtime_note"]),
        ("Rust link driver and linker", darwin["rust_linker_note"], linux["rust_linker_note"]),
        ("Stack bound for Bun's recursion check", darwin["stack_bounds"], linux["stack_bounds"]),
        ("Bun Highway platform branch", darwin["bun_os_branch"], linux["bun_os_branch"]),
        ("`-C target-cpu=generic`", "generic AArch64 (NEON)", "x86-64 baseline (SSE2); engines "
         "with runtime detection still select SSSE3/AVX2 paths"),
        ("Highway targets", "runtime dispatch; SVE list disabled as in Bun", "runtime dispatch over "
         "the x86 targets; the SVE list has no effect"),
        ("PGO training-binary stubs", "the shared list", "the shared list plus the Bun support "
         "symbols GNU ld also resolves; the measured executable links none of them"),
    ]
    table = ["| Aspect | macOS (Apple Silicon) | Linux (x86-64) |", "| --- | --- | --- |"]
    table += [f"| {a} | {b} | {c} |" for a, b, c in rows]
    audit_lines = [f"- {name}: {value.get('checked_files')} files, "
                   f"{len(value.get('mismatches', []))} mismatches"
                   for name, value in audit.items() if name != "native_dependencies"]
    audit_lines += [f"- {name} archive: {value['checked_files']} files, {len(value['mismatches'])} mismatches"
                    for name, value in audit.get("native_dependencies", {}).items()]
    observations = []
    for item in run["observations"]:
        before, after = item["before"], item.get("after", {})
        observations.append(
            f"| {item['round'] + 1} | {before['time_utc'][11:19]} | {after.get('time_utc', '')[11:19]} | "
            f"{before['load_average'][0]:.2f} | {after.get('load_average', [float('nan')])[0]:.2f} | "
            f"{percent(cpu_steal(before, after))} |")
    lines = [
        "# Source and build provenance",
        "",
        f"{source['origin']} The workflow checked out `{source['harness_revision']}` for the harness "
        "and exported the measured revisions with `git archive`; working-tree sources are never built.",
        "",
        "| Engine | Pin |",
        "| --- | --- |",
        f"| Ferromark v2 | `{engines['ferromark_v2']['revision']}` |",
        f"| Ferromark v1 | `{engines['ferromark_v1']['revision']}` |",
        f"| OX-Content original | `{engines['ox_content']['revision']}` (archive SHA-256 "
        f"`{engines['ox_content'].get('archive_sha256', 'n/a')}`) |",
        f"| md4c | `{engines['md4c']['revision']}` |",
        f"| Bun | `{engines['bun']['revision']}` |",
        "| pulldown-cmark | 0.13.4 from the seeded lock |",
        "",
        "The five comparison pins are the ones every Apple Silicon report since "
        "2026-09-14 uses; only v2 moves. [restore.py](restore.py) restored them from their upstream "
        "URLs and checked the OX, mimalloc, and Highway archive checksums "
        "([restore.json](restore.json)).",
        "",
        "## Build conditions",
        "",
        f"- Host: {source['host_summary']}; `{platform['host_triple']}`.",
        f"- Rust: `{first_line(build['rustc'])}`, LLVM {llvm_version(build['rustc'])}, the pinned "
        f"`{build['toolchain']}` required by the native Bun integration.",
        f"- C/C++: `{first_line(platform['clang'])}`. Linker: `{platform.get('linker', 'not recorded')}`.",
        f"- `RUSTFLAGS`: `{build['rustflags']}`; optimization level {build['optimization']['opt_level']}, "
        f"{build['optimization']['lto']} LTO, {build['optimization']['codegen_units']} codegen unit, "
        f"panic {build['optimization']['panic']}. No PGO in the default build.",
        "- C/C++ flags, mimalloc defines, and the md4c allocator redirection are the ones in "
        "`harness/prepare.py`, identical on both platforms.",
        f"- Executable SHA-256: `{build['binary_sha256']}`.",
        f"- Final Cargo lock SHA-256: `{build['lock_sha256']}`.",
        "",
        "### Platform differences",
        "",
        "Recorded per build in `build.json` under `platform`:",
        "",
        *table,
        "",
        "Bun's own Linux release build also turns on mimalloc's global `malloc` override and "
        "disables transparent huge pages for mimalloc arenas. The harness applies neither on either "
        "platform: every engine already allocates through the same mimalloc, Rust through Bun's "
        "global allocator and md4c through `md4c_alloc.h`, and one mimalloc configuration keeps the "
        "platforms comparable. The runner's transparent huge page mode is in [host.txt](host.txt).",
        "",
        "### The lock was seeded, not replayed with `--locked`",
        "",
        f"The build seeded Cargo with `{Path(registry['seed_lock']).as_posix()}` through `--bun-lock` "
        "because the v2 path package's version line differs from that lock. `CARGO_NET_OFFLINE=true` "
        "and `--offline` kept the build itself offline, and `prepare.py` fails if Cargo resolves "
        "any registry package outside the union of the engine locks. "
        + (f"All {registry['registry_packages']} resolved registry packages are identical to the "
           "seed lock in name, version, and checksum."
           if registry["identical_to_seed"] else
           f"**The resolution differs from the seed lock**: added {registry['added']}, removed "
           f"{registry['removed']}, checksum changes {registry['checksum_changed']}."),
        "",
        "## Source audit",
        "",
        "`harness/audit_sources.py` checked the exported sources against the pinned Git blobs and "
        "the checksummed archives ([source-audit.json.gz](source-audit.json.gz)):",
        "",
        *audit_lines,
        "",
        "## Measurement conditions",
        "",
        f"{facts['measurement_note']} Load averages and hypervisor steal per process round:",
        "",
        "| Round | Start (UTC) | End | Load before | Load after | Steal |",
        "| ---: | --- | --- | ---: | ---: | ---: |",
        *observations,
        "",
        "Thermal and clock readings, where the virtual machine exposes them, are in `run.json`.",
        "",
        "## Reproduction",
        "",
        "Run the workflow again with the same v2 revision:",
        "",
        "```sh",
        f"gh workflow run native-comparison.yml -f revision={engines['ferromark_v2']['revision']}"
        + (" -f pgo=true" if facts.get("pgo") else ""),
        "```",
        "",
        "or follow the harness README's reproduction commands on a Linux x86-64 host with clang, "
        "using this report's `restore.py` in an empty cache directory.",
        "",
    ]
    (out / "PROVENANCE.md").write_text("\n".join(lines))


def checksums(out: Path) -> None:
    entries = []
    for path in sorted(p for p in out.rglob("*") if p.is_file() and p.name != "SHA256SUMS"):
        entries.append(f"{prepare.sha(path)}  {path.relative_to(out).as_posix()}")
    (out / "SHA256SUMS").write_text("\n".join(entries) + "\n")


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("parent", type=Path, help="directory that receives <date>-native-<platform>/")
    p.add_argument("--results", type=Path, required=True, help="run.py output with report.py tables")
    p.add_argument("--build-dir", type=Path, required=True, help="prepare.py build directory")
    p.add_argument("--seed-lock", type=Path, required=True, help="lock passed to prepare.py --bun-lock/--lockfile")
    p.add_argument("--reference", type=Path, required=True, help="archived report whose HTML is compared")
    p.add_argument("--restore-dir", type=Path, required=True, help="directory holding restore.py and restore.json")
    p.add_argument("--host-file", type=Path, required=True, help="host description, for example lscpu output")
    p.add_argument("--host-summary", required=True, help="one-line host description for the README")
    p.add_argument("--origin", required=True, help="one sentence naming where and how the run happened")
    p.add_argument("--harness-revision", required=True)
    p.add_argument("--measurement-note", help="one sentence on what else ran during timing",
                   default="No other benchmark process was started while `run.py` was timing.")
    p.add_argument("--build-log", type=Path, required=True)
    p.add_argument("--source-audit", type=Path, required=True)
    p.add_argument("--tests-log", type=Path)
    p.add_argument("--pgo-build-dir", type=Path)
    p.add_argument("--pgo-default-results", type=Path)
    p.add_argument("--pgo-results", type=Path)
    p.add_argument("--pgo-build-log", type=Path, help="stored compressed as build-log-pgo.txt.gz")
    p.add_argument("--name", help="directory name; defaults to <run date>-native-<platform>")
    p.add_argument("--step-summary", type=Path, help="append the headline table here")
    args = p.parse_args()
    pgo_inputs = (args.pgo_build_dir, args.pgo_default_results, args.pgo_results)
    if any(pgo_inputs) and not all(pgo_inputs):
        p.error("--pgo-build-dir, --pgo-default-results and --pgo-results go together")

    run = read(args.results, "run.json")
    build = read(args.results, "build.json")
    if build != json.loads((args.build_dir / "build.json").read_text()):
        raise SystemExit(f"{args.results} was not measured with the build in {args.build_dir}")
    date = run["host_before"]["time_utc"][:10]
    slug, label = platform_names(build)
    out = args.parent / (args.name or f"{date}-native-{slug}")
    out.mkdir(parents=True, exist_ok=False)

    copy_results(args.results, out)
    shutil.copyfile(args.results / "build.json", out / "build.json")
    shutil.copyfile(args.results / "Cargo.lock", out / "Cargo.lock")
    shutil.copyfile(args.host_file, out / "host.txt")
    shutil.copyfile(args.build_log, out / "build-log.txt")
    for name in ("restore.py", "restore.json"):
        shutil.copyfile(args.restore_dir / name, out / name)
    write_gzip(out / "source-audit.json.gz", args.source_audit.read_bytes())
    harness = out / "harness"
    harness.mkdir()
    for path in sorted(HERE.iterdir()):
        if path.is_file() and not path.name.startswith("."):
            shutil.copyfile(path, harness / path.name)

    cases = read(args.results, "corpus.json")["cases"]
    verification = read(args.results, "verification.json")
    summary = read(args.results, "summary.json")
    samples = read(args.results, "samples.json")
    windows = audit_windows(run, samples, verification)
    six, five, scores = main_scores(summary, verification)
    reference = read(args.reference, "verification.json")
    outputs = compare_outputs(verification, reference)
    registry = registry_check(args.results / "Cargo.lock", args.seed_lock)
    audit = json.loads(args.source_audit.read_text())
    for name, value in [*audit.items(), *audit.get("native_dependencies", {}).items()]:
        mismatches = value.get("mismatches", [])
        assert not mismatches, ("source audit mismatch", name, mismatches)
    checks = {
        "workflow_origin": args.origin,
        "harness_revision": args.harness_revision,
        "harness_tests": harness_tests(args.tests_log),
        "verify_before_timing": "run.py --verify-only passed on the default build before any timing",
        "timed_windows": {"main": windows, "checksums": "all passed; rechecked from samples.json"},
        "registry": registry,
        "outputs_vs_reference": {"reference": args.reference.name, **outputs},
        "source_audit": {name: {"checked_files": v.get("checked_files"), "mismatches": len(v.get("mismatches", []))}
                         for name, v in audit.items() if name != "native_dependencies"},
        "hypervisor_steal_during_timing": cpu_steal(run["host_before"], run["host_after"]),
    }
    facts = dict(build=build, run=run, cases=cases, six=six, five=five, scores=scores,
                 rounds=round_comparisons(summary, run, six, five), windows=windows,
                 outputs=outputs, reference_name=args.reference.name, registry=registry,
                 steal=cpu_steal(run["host_before"], run["host_after"]), tests=checks["harness_tests"],
                 source=dict(origin=args.origin, host_summary=args.host_summary,
                             harness_revision=args.harness_revision),
                 source_audit=audit, date=date, platform_label=label,
                 measurement_note=args.measurement_note)
    (out / "comparison-rounds.json").write_text(json.dumps(facts["rounds"], indent=2) + "\n")

    if args.pgo_build_dir:
        build_pgo = json.loads((args.pgo_build_dir / "build.json").read_text())
        shutil.copyfile(args.pgo_build_dir / "build.json", out / "build-pgo.json")
        if args.pgo_build_log:
            write_gzip(out / "build-log-pgo.txt.gz", args.pgo_build_log.read_bytes())
        for label, directory in (("default", args.pgo_default_results), ("pgo", args.pgo_results)):
            copy_results(directory, out / "pgo" / label)
            shutil.copyfile(directory / "run.json", out / f"run-pgo-{label}.json")
            held_run = read(directory, "run.json")
            held_windows = audit_windows(held_run, read(directory, "samples.json"),
                                         read(directory, "verification.json"))
            checks["timed_windows"][f"pgo_{label}"] = held_windows
        facts["pgo"] = pgo_comparison(args.pgo_default_results, args.pgo_results, build_pgo)
        default_html = read(args.pgo_default_results, "verification.json")
        pgo_html = read(args.pgo_results, "verification.json")
        facts["pgo_outputs_equal"] = default_html == pgo_html and all(
            default_html[name] == verification[name] for name in default_html)
        checks["pgo"] = {**facts["pgo"], "outputs_equal_across_builds": facts["pgo_outputs_equal"]}
        (out / "pgo-comparison.json").write_text(json.dumps(facts["pgo"], indent=2) + "\n")

    (out / "checks").mkdir()
    (out / "checks" / "commands.json").write_text(json.dumps(checks, indent=2) + "\n")
    write_readme(out, facts)
    write_provenance(out, facts)
    checksums(out)
    if args.step_summary:
        with args.step_summary.open("a") as stream:
            stream.write(f"### Native comparison on {facts['platform_label']}, v2 "
                         f"`{build['engines']['ferromark_v2']['revision'][:10]}`\n\n"
                         f"{run['host_before']['cpu']}; speed relative to v2, higher is faster.\n\n"
                         + main_table(six, five, scores) + "\n\n"
                         f"{sum(e['identical'] for e in outputs['engines'].values())} of "
                         f"{outputs['documents_compared'] * len(tables.ENGINES)} HTML outputs are "
                         f"byte-identical to `{args.reference.name}`.\n")
    print(out)


if __name__ == "__main__":
    main()
