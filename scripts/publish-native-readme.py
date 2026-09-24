#!/usr/bin/env python3
"""Publish archived native comparisons to the website and README without changing the archives.

``REPORTS`` names one archived report per platform. Every published figure is
derived from that report's own data files with the aggregation code archived in
its ``harness/report.py``, so a report written by
``benchmarks/native-comparison/archive.py`` and an older one that ships its own
``publish.py`` are read the same way, and no archived file is ever rewritten.

From those reports this script writes three things: the website's native
comparison section (``homepage/app/routes/guide/benchmarks.mdx``), the homepage
figures (``homepage/app/data/native-benchmarks.json``), and the marked sentences
in ``README.md.src`` and its generated ``README.md``. ``--check`` fails when any
of them drifts from the archives.
"""

import argparse
import importlib.util
import json
import re
import sys
import textwrap
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# One archived report per platform, in publishing order. To publish a new
# measurement, archive it under docs/reports/ and point its platform here.
REPORTS = {
    "macos-arm64": Path("docs/reports/2026-09-24-native-macos-arm64"),
    "linux-x86-64": Path("docs/reports/2026-09-24-native-linux-x86-64"),
}

# How each platform is named. `machine` is used only when a report records the
# bare architecture instead of a CPU model (run.py records BENCH_CPU when set).
PLATFORMS = {
    "macos-arm64": dict(label="Apple Silicon", executable="macOS arm64", machine="Apple M1 Pro"),
    "linux-x86-64": dict(label="Linux x86-64", executable="Linux x86-64", machine=None),
}

GUIDE = ROOT / "homepage/app/routes/guide/benchmarks.mdx"
GUIDE_START = "## Native engine comparison\n"
GUIDE_END = "The historical [two-engine broad Markdown comparison]"
FIGURES = ROOT / "homepage/app/data/native-benchmarks.json"
READMES = (ROOT / "README.md.src", ROOT / "README.md")
README_START = "<!-- native-benchmarks -->"
README_END = "<!-- /native-benchmarks -->"
PREFIX = "https://github.com/sebastian-software/ferromark/blob/main/"
HARNESS = PREFIX + "benchmarks/native-comparison/README.md"
CI_CLAIMS = HARNESS + "#what-a-ci-report-can-and-cannot-claim"
HELD_OUT_HEADING = "## Profile-guided optimization, measured on the held-out half"
HELD_OUT_ANCHOR = "#profile-guided-optimization-measured-on-the-held-out-half"
REPEATS_SUFFIX = "-repeats"
# What distinguishes an earlier report once it is no longer published.
EARLIER = {
    "2026-09-21-native-round-4": "after optimization round 4",
    "2026-09-21-native-release-fixed": "the repaired release head, before round 4",
    "2026-09-21-native-release": "the regressed release head",
}
LANDING_ENGINES = ("v1", "pulldown-cmark", "md4c", "bun")
README_NAMES = {"v1": "Ferromark v1", "pulldown-cmark": "pulldown-cmark", "md4c": "md4c",
                "bun": "Bun's native engine"}
ARCHITECTURES = {"arm64": "arm64", "aarch64": "arm64", "x86_64": "x86-64", "amd64": "x86-64"}
GROUPS = (
    ("&lt;512 B", lambda c: c["byte_count"] < 512),
    ("32–128 KiB", lambda c: 32768 <= c["byte_count"] < 131072),
    ("comments", lambda c: c["category"] == "comments"),
    ("technical-docs", lambda c: c["category"] == "technical-docs"),
    ("plain-prose", lambda c: c["category"] == "plain-prose"),
)


def speed(value):
    """The landing page and README state one decimal: 2.07 becomes 2.1×."""
    return f"{value:.1f}×"


LINK_TEXT = re.compile(r"\[([^\]]*)\]\(")


def fill(text):
    """Wrap prose at 80 columns without splitting a link's text across lines."""
    protected = LINK_TEXT.sub(lambda match: "[" + match[1].replace(" ", "\0") + "](", text)
    wrapped = textwrap.fill(protected, width=80, break_long_words=False, break_on_hyphens=False)
    return wrapped.replace("\0", " ")


def load_harness(directory):
    """The aggregation code archived with the report, not the current harness."""
    name = "native_report_" + re.sub(r"\W", "_", directory.name)
    spec = importlib.util.spec_from_file_location(name, directory / "harness/report.py")
    module = importlib.util.module_from_spec(spec)
    # Importing would otherwise write harness/__pycache__ into the archive.
    writes_bytecode, sys.dont_write_bytecode = sys.dont_write_bytecode, True
    try:
        spec.loader.exec_module(module)
    finally:
        sys.dont_write_bytecode = writes_bytecode
    return module


def platform_slug(build, run):
    """The archive.py slug, for example "linux-x86-64", from the recorded host."""
    details = build.get("platform")
    if details:
        system, machine = details["system"], details["machine"]
    else:
        # Reports from before Linux support record only platform.platform().
        system, _, rest = run["host_before"]["platform"].partition("-")
        machine = next((a for a in ARCHITECTURES if a in rest.split("-")), "unknown")
    system = {"Darwin": "macos", "macOS": "macos", "Linux": "linux"}.get(system, system.lower())
    return f"{system}-{ARCHITECTURES.get(machine.lower(), machine.lower())}"


def cpu_model(recorded):
    """ "AMD EPYC 7763 64-Core Processor" becomes "AMD EPYC 7763"."""
    name = re.sub(r"\((?:R|TM)\)", "", recorded)
    name = re.sub(r"\s+CPU\s+@.*$", "", name)
    name = re.sub(r"(?:\s+\d+-Core)?\s+Processor$", "", name)
    return " ".join(name.split())


def machine_name(recorded, fallback, hosted):
    if recorded.strip().lower() in ARCHITECTURES or not recorded.strip():
        if fallback is None:
            raise SystemExit(f"The report records only {recorded!r}; set BENCH_CPU when measuring "
                             "or name the machine in PLATFORMS")
        model = fallback
    else:
        model = cpu_model(recorded)
    return f"GitHub-hosted runner, {model}" if hosted else model


def held_out(tables, directory, verification):
    """PGO speedup and v2's standing within each build, on the held-out half."""
    default, optimized = directory / "pgo/default", directory / "pgo/pgo"
    if not (default / "summary.json").exists() or not (optimized / "summary.json").exists():
        return None
    cases = {c["name"] for c in tables.read(default, "corpus.json")["cases"]}
    default_summary = tables.read(default, "summary.json")
    pgo_summary = tables.read(optimized, "summary.json")
    dmap = {(r["case"], r["mode"]): r for r in default_summary}
    pmap = {(r["case"], r["mode"]): r for r in pgo_summary}
    gain = {mode: tables.geomean(dmap[c, mode]["engines"]["v2"]["ns"] / pmap[c, mode]["engines"]["v2"]["ns"]
                                 for c in cases) for mode in ("fresh", "reuse")}
    six, five = tables.agreement_sets({k: v for k, v in verification.items() if k in cases})
    figures = []
    for engine in (*LANDING_ENGINES, "ox-content"):
        members = (six if engine == "ox-content" else five) & cases
        engines = tables.ENGINES if engine == "ox-content" else tables.CONFIGURABLE
        values = {mode: 1 / tables.aggregate(pgo_summary, members, mode, engines)[engine]
                  for mode in ("fresh", "reuse")}
        figures.append(dict(id=engine, label=tables.LABELS[engine], documents=len(members),
                            fresh=values["fresh"], reuse=values["reuse"]))
    return dict(heldOut=len(cases), v2Gain=gain, figures=figures)


def measure(key, relative):
    """Everything the website and README state about one platform, from its archive."""
    directory = ROOT / relative
    if not (directory / "summary.json").exists():
        raise SystemExit(f"{relative} is not an archived native comparison report; archive the "
                         f"{key} report there or point REPORTS[{key!r}] at one")
    tables = load_harness(directory)
    run = tables.read(directory, "run.json")
    build = tables.read(directory, "build.json")
    slug = platform_slug(build, run)
    if slug != key:
        raise SystemExit(f"{relative} was measured on {slug}, not {key}")
    checks = directory / "checks/commands.json"
    origin = json.loads(checks.read_text()).get("workflow_origin", "") if checks.exists() else ""
    hosted = "GitHub-hosted" in origin
    corpus = tables.read(directory, "corpus.json")["cases"]
    verification = tables.read(directory, "verification.json")
    summary = tables.read(directory, "summary.json")
    six, five = tables.agreement_sets(verification)
    scores = {}
    for name, members, engines in (("six", six, tables.ENGINES), ("five", five, tables.CONFIGURABLE)):
        for mode in ("fresh", "reuse"):
            scores[name, mode] = tables.aggregate(summary, members, mode, engines)
    groups = []
    for label, predicate in GROUPS:
        names = {c["name"] for c in corpus if c["name"] in five and predicate(c)}
        groups.append((label, len(names), tables.aggregate(summary, names, "fresh", tables.CONFIGURABLE)))
    figures = []
    for engine in (*LANDING_ENGINES, "ox-content"):
        group = "six" if engine == "ox-content" else "five"
        figures.append(dict(id=engine, label=tables.LABELS[engine],
                            documents=len(six if group == "six" else five),
                            fresh=1 / scores[group, "fresh"][engine],
                            reuse=1 / scores[group, "reuse"][engine]))
    readme = (directory / "README.md").read_text()
    platform = PLATFORMS[key]
    # Reduced evidence of further runs of the same measurement, archived next to the report.
    repeats = relative.parent / (relative.name + REPEATS_SUFFIX)
    repeats = repeats.as_posix() if (ROOT / repeats / "README.md").exists() else None
    return dict(
        id=key, label=platform["label"], executable=platform["executable"],
        machine=machine_name(run["host_before"].get("cpu", ""), platform["machine"], hosted),
        sharedRunner=hosted, report=relative.as_posix(),
        revision=build["engines"]["ferromark_v2"]["revision"][:8],
        v1Revision=build["engines"]["ferromark_v1"]["revision"][:7],
        measured=run["host_before"]["time_utc"][:10], rounds=run["rounds"], windows=run["samples"],
        corpus=corpus, six=len(six), five=len(five), scores=scores, groups=groups, figures=figures,
        pgo=held_out(tables, directory, verification), labels=tables.LABELS, engines=tables.ENGINES,
        files=[name for name in ("FLAGS.md", "OUTPUT-REVIEW.md", "PROVENANCE.md") if (directory / name).exists()],
        heldOutSection=HELD_OUT_HEADING in readme, repeats=repeats,
    )


def website_json(platforms):
    def figure(row):
        return dict(id=row["id"], label=row["label"], documents=row["documents"],
                    fresh=round(row["fresh"], 2), reuse=round(row["reuse"], 2))

    def record(p):
        pgo = None
        if p["pgo"]:
            pgo = dict(v2Gain={m: round(v, 2) for m, v in p["pgo"]["v2Gain"].items()},
                       figures=[figure(row) for row in p["pgo"]["figures"]])
        return dict(
            id=p["id"], label=p["label"], machine=p["machine"], sharedRunner=p["sharedRunner"],
            report=p["report"], revision=p["revision"], measured=p["measured"],
            documents=dict(all=len(p["corpus"]), fiveEngineAgreement=p["five"], sixEngineAgreement=p["six"],
                           heldOut=p["pgo"]["heldOut"] if p["pgo"] else None),
            figures=[figure(row) for row in p["figures"]],
            pgo=pgo,
        )

    reports = ", ".join(p["report"] for p in platforms)
    website = dict(
        note=f"Generated by scripts/publish-native-readme.py from {reports}. Do not hand-edit figures.",
        platforms=[record(p) for p in platforms],
    )
    return json.dumps(website, indent=2, ensure_ascii=False) + "\n"


def by_id(platform):
    return {row["id"]: row for row in platform["figures"]}


def readme_block(platforms):
    for platform in platforms:
        for engine in LANDING_ENGINES:
            if by_id(platform)[engine]["fresh"] <= 1:
                raise SystemExit(f"The README says v2 is faster than {engine} on {platform['label']}; "
                                 "the archive no longer shows that, so revise the wording")
    counts = {p["five"] for p in platforms}
    shared_count = len(counts) == 1

    def context(p, *extra):
        parts = [p["machine"], *extra] + ([] if shared_count else [f"{p['five']} documents"])
        return ", ".join(parts + [p["measured"]])

    def caveat(p):
        return (f"; that runner is shared, so [compare the ratios, not the times]({CI_CLAIMS})"
                if p["sharedRunner"] else "")

    first, *others = platforms
    figures = by_id(first)
    documents = f"{first['five']} real documents" if shared_count else "real documents"
    sentences = [
        f"On {documents} that five native engines render to equivalent HTML, v2 completes Markdown "
        "to HTML "
        + ", ".join(f"{'and ' if engine == LANDING_ENGINES[-1] else ''}"
                    f"{speed(figures[engine]['fresh'])} faster than {README_NAMES[engine]}"
                    for engine in LANDING_ENGINES)
        + f" on {first['label']} ({context(first, 'fresh parser state')}){caveat(first)}."
    ]
    for platform in others:
        figures = by_id(platform)
        values = [speed(figures[engine]["fresh"]) for engine in LANDING_ENGINES]
        sentences.append(f"On {platform['label']} ({context(platform)}) the four ratios are "
                         f"{', '.join(values[:-1])}, and {values[-1]}{caveat(platform)}.")
    return README_START + "\n" + fill(" ".join(sentences)) + "\n" + README_END


def link(relative):
    return PREFIX + relative


def main_table(p):
    labels, six, five, scores = p["labels"], p["six"], p["five"], p["scores"]
    lines = [f"| Engine | Fresh, {six} agreeing across six | Reuse, same {six} | "
             f"Fresh, {five} agreeing across five | Reuse, same {five} |",
             "| --- | ---: | ---: | ---: | ---: |"]
    for engine in p["engines"]:
        values = [scores[g, m].get(engine) for g, m in
                  (("six", "fresh"), ("six", "reuse"), ("five", "fresh"), ("five", "reuse"))]
        lines.append("| " + labels[engine] + " | " + " | ".join(
            f"{v:.2f}×" if v is not None else "—" for v in values) + " |")
    return "\n".join(lines)


def group_table(p):
    lines = ["| Group, five-engine agreement | N | V1 | md4c | pulldown | Bun native |",
             "| --- | ---: | ---: | ---: | ---: | ---: |"]
    for label, count, values in p["groups"]:
        lines.append(f"| {label} | {count} | " + " | ".join(
            f"{values[e]:.2f}×" for e in ("v1", "md4c", "pulldown-cmark", "bun")) + " |")
    return "\n".join(lines)


def platform_section(p):
    figures = by_id(p)
    report = p["report"]
    parts = [
        f"### {p['label']}",
        "",
        fill(f"{p['machine']}, measured {p['measured']}: v2 `{p['revision']}` and v1 `{p['v1Revision']}` "
             f"in one {p['executable']} executable, {p['rounds']} process rounds with {p['windows']} "
             "rotating windows per round."),
        "",
    ]
    if p["sharedRunner"]:
        parts += [fill(
            "The runner is a virtual machine on shared hardware. All six engines alternated in the "
            "same process rounds and rotating windows on that one host, so the ratios between "
            "engines are meaningful; absolute times are not comparable with any other run. See "
            f"[what a CI report can and cannot claim]({CI_CLAIMS})."), ""]
    ox = figures["ox-content"]
    parts += [
        main_table(p),
        "",
        fill(f"On the {p['five']} documents all five configurable engines render alike, v2 runs at "
             f"{figures['v1']['fresh']:.2f}× v1's speed fresh and {figures['v1']['reuse']:.2f}× with reuse, "
             f"{figures['pulldown-cmark']['fresh']:.2f}× pulldown-cmark's, {figures['md4c']['fresh']:.2f}× "
             f"md4c's, and {figures['bun']['fresh']:.2f}× Bun's native engine's fresh. On the {p['six']} "
             f"documents all six engines agree on, it runs at {ox['fresh']:.2f}× OX's throughput fresh "
             f"and {ox['reuse']:.2f}× with reuse."),
        "",
        fill("Selected groups, **fresh**, using only the five-engine agreement subset and the same "
             "speed scale:"),
        "",
        group_table(p),
        "",
    ]
    if p["pgo"]:
        pgo = by_id(p["pgo"])
        gain = p["pgo"]["v2Gain"]
        parts += [fill(
            f"On the {p['pgo']['heldOut']} held-out documents no profile saw, profile-guided "
            f"optimization gives v2 {gain['fresh']:.2f}× fresh and {gain['reuse']:.2f}× with reuse over "
            "its default build; against the PGO builds of the other engines, v2 stands at "
            f"{pgo['v1']['fresh']:.2f}× v1, {pgo['pulldown-cmark']['fresh']:.2f}× pulldown-cmark, "
            f"{pgo['md4c']['fresh']:.2f}× md4c, and {pgo['bun']['fresh']:.2f}× Bun's native engine fresh."),
            ""]
    links = [f"[Full results and per-document timings]({link(report + '/README.md')})"]
    if p["pgo"] and p["heldOutSection"]:
        links.append(f"[held-out PGO comparison]({link(report + '/README.md' + HELD_OUT_ANCHOR)})")
    names = {"FLAGS.md": "exact flags", "OUTPUT-REVIEW.md": "HTML differences",
             "PROVENANCE.md": "source and build provenance"}
    links += [f"[{names[name]}]({link(report + '/' + name)})" for name in p["files"]]
    listed = ", ".join(links[:-1]) + ", and " + links[-1] if len(links) > 1 else links[0]
    parts += [fill(f"{listed} include raw measurements, source hashes, and reproducible configuration."), ""]
    if p["repeats"]:
        parts += [fill(f"[Repeat runs on other runners]({link(p['repeats'] + '/README.md')}) show how "
                       "much these figures move between runs and runner CPUs."), ""]
    return "\n".join(parts)


def earlier_reports(published):
    """Every other archived native comparison, newest first, as historical evidence."""
    entries = []
    for directory in sorted((ROOT / "docs/reports").iterdir(), reverse=True):
        relative = directory.relative_to(ROOT).as_posix()
        build = directory / "build.json"
        if "native" not in directory.name or relative in published or not build.exists():
            continue
        if not (directory / "README.md").exists():
            continue
        data = json.loads(build.read_text())
        revision = data.get("engines", {}).get("ferromark_v2", {}).get("revision")
        if not revision:
            continue
        details = [directory.name[:10]]
        if data.get("platform"):
            system, machine = platform_slug(data, {}).split("-", 1)
            details.append({"macos": "macOS", "linux": "Linux"}.get(system, system) + " " + machine)
        if directory.name in EARLIER:
            details.append(EARLIER[directory.name])
        entries.append(f"[`{revision[:8]}`]({link(relative + '/README.md')}) ({', '.join(details)})")
    return entries


def guide_section(platforms):
    corpus = platforms[0]["corpus"]
    sizes = [c["byte_count"] for c in corpus]
    for p in platforms[1:]:
        if [(c["name"], c["byte_count"]) for c in p["corpus"]] != [(c["name"], c["byte_count"]) for c in corpus]:
            raise SystemExit(f"{p['report']} measured a different corpus than {platforms[0]['report']}")
    intro = [
        f"Ferromark v2, the original OX-Content core, Ferromark v1, md4c, pulldown-cmark, and Bun's "
        f"native `bun_md` engine parse and render the frozen **{len(corpus)} documents "
        f"({min(sizes):,}–{max(sizes):,} UTF-8 bytes)** with explicit syntax and renderer flags, in one "
        "native executable per platform. Each platform below has its own run, its own archived "
        "report, and its own figures: an Apple Silicon figure does not describe x86-64 or the "
        "reverse, because SIMD paths, compiler back ends, and cache hierarchies differ. Each scored "
        "column uses the same input set and equivalent HTML for every included engine. **Speed "
        "relative to v2: higher is faster; v2 = 1.00×.**",
        "Only documents on which the engines produce equivalent HTML are scored. The all-six "
        "columns require agreement among all six engines; the broader five-engine columns require "
        "it among v2, v1, md4c, pulldown-cmark, and Bun. OX has no score in the five-engine "
        "columns: its original renderer cannot disable heading IDs, callouts, or fence metadata "
        "cleanup, and its heading-ID-only differences are not normalized away. Documents outside "
        "an agreement subset remain measured as diagnostics in each report.",
        "All engines parse and render natively. The CommonMark lane disables optional syntax; the "
        "extension lane enables only **tables, strikethrough, and task lists**. It is not full GFM. "
        "Bare URL autolinking, footnotes, frontmatter, line comments, definition lists, MDX, and "
        "optional renderer extras are off where configurable. Raw HTML passes through; these "
        "explicit benchmark settings differ from library defaults.",
        "Fresh includes parser/renderer setup, complete processing, owned output, and destruction. "
        "Reuse retains state where the public API permits; Bun's native `bun_md` still uses its "
        "fresh owned-output API. No JavaScript, WASM, process startup, file I/O, or output "
        "normalization is timed. On each platform all six engines share one executable, one pinned "
        "Rust compiler, mimalloc, and the same pinned registry resolution, and alternate in "
        "rotating windows within the same process rounds; the few build differences between the "
        f"platforms are listed in the [harness README]({HARNESS}#platforms). These are measured "
        "corpus aggregates on one machine per platform, not a universal ranking or a significance "
        "claim.",
        "Ferromark's published binaries are built with profile-guided optimization. Where a report "
        "includes it, the same recipe was applied to every Rust engine in the shared executable and "
        "measured on held-out documents no profile saw. md4c is C and is not PGO-built; a PGO row "
        "is compared only with PGO rows.",
    ]
    parts = [GUIDE_START] + [fill(paragraph) + "\n" for paragraph in intro]
    parts += [platform_section(p) for p in platforms]
    earlier = earlier_reports({p["report"] for p in platforms})
    if earlier:
        listed = ", ".join(earlier[:-1]) + ", and " + earlier[-1] if len(earlier) > 1 else earlier[0]
        suite = link("docs/reports/2026-09-15-arm-full-suite/README.md")
        parts += ["### Earlier comparisons\n",
                  fill(f"Earlier native comparisons remain archived as historical evidence: {listed}. "
                       f"The [full 207-case before/after suite]({suite}) uses a separate harness and "
                       "reports all four stages.") + "\n"]
    return "\n".join(parts) + "\n"


def replace_guide(text, section):
    start = text.find(GUIDE_START)
    end = text.find(GUIDE_END, start)
    if start == -1 or end == -1:
        raise SystemExit(f"{GUIDE.name} has no {GUIDE_START.strip()!r} section before {GUIDE_END!r}")
    return text[:start] + section + text[end:]


def readme_span(text, path):
    start = text.find(README_START)
    end = text.find(README_END, start)
    if start == -1 or end == -1:
        raise SystemExit(f"{path} has no {README_START} … {README_END} block")
    return start, end + len(README_END)


def replace_readme(text, path, block):
    start, end = readme_span(text, path)
    return text[:start] + block + text[end:]


def render(reports=None):
    """The expected content of every published file, keyed by path."""
    platforms = [measure(key, relative) for key, relative in (reports or REPORTS).items()]
    block = readme_block(platforms)
    expected = {GUIDE: replace_guide(GUIDE.read_text(), guide_section(platforms)),
                FIGURES: website_json(platforms)}
    for path in READMES:
        expected[path] = replace_readme(path.read_text(), path, block)
    return expected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected = render()
    if args.check:
        drifted = [path.relative_to(ROOT).as_posix() for path, text in expected.items()
                   if not path.exists() or path.read_text() != text]
        if drifted:
            raise SystemExit("Benchmark drift in " + ", ".join(drifted)
                             + ": run python3 scripts/publish-native-readme.py")
        print("Website, homepage and README benchmark figures match the archived measurements")
    else:
        for path, text in expected.items():
            path.write_text(text)
        print("Updated the website benchmark guide, the homepage figures and the README sentences")


if __name__ == "__main__":
    sys.exit(main())
