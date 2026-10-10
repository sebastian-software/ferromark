# Comparison conformance

This untimed runner checks the 20 variants displayed in the homepage comparison,
plus separate Ferromark native and Node references. It measures output agreement
with all 652 CommonMark 0.31.2 examples and the 28 extension examples extracted
from the frozen official GFM website response. **GFM extensions** is a subset
score, not a full GFM conformance percentage.

The private comparison workspace retains the exact library versions from the
performance campaign. Spec adapters select the closest available public
configuration instead of the reduced shared timing profile. The
[profiles](profiles.json) record exact settings and differences. No output is
rewritten to remove IDs, classes, code whitespace, URL differences, or errors.
Native addons use their released Node API, not a substituted Rust API.

## Run locally

On macOS or Linux, use Python 3.11+, the repository Rust toolchain,
`nightly-2026-07-20`, Go 1.27.1, Node, npm, CMake, clang, git, and curl. A full
run needs network access for pinned sources and package archives. Commit the
adapter changes first so the native build archives that source revision.

```sh
./scripts/comparison-conformance run /tmp/ferromark-conformance
```

The output directory must be new. This command restores immutable competitor
commits and checksum-verified native archives, builds the adapters, installs the
private pinned Node workspace, builds the local Ferromark addon, and records
all cases under `results/`. It does not run a timing campaign or modify the
57-document performance corpus, adapters, scoring policy, or published factors.
The build writes only to its output directory and normal ignored dependency,
Cargo target, and local addon locations. It does not publish packages.

After inspecting the raw evidence and profile notes, publish a new retained
report and regenerate the homepage values:

```sh
./scripts/comparison-conformance publish /tmp/ferromark-conformance/results
./scripts/comparison-conformance check
python3 -m unittest discover -s benchmarks/conformance -p 'test_*.py'
```

The publisher refuses to overwrite an existing report. For a future campaign,
update the report destination in `cli.py` before measuring. `measure RESULTS
--build BUILD` can reuse a prepared build for diagnosis; binary and addon
hashes are checked. Use `run` for a retained campaign with fresh build provenance.
Set `FERROMARK_CONFORMANCE_PYTHON` to an explicit Python executable when the
system `python3` is older than 3.11.

## How results are counted

Each supported suite has the same denominator for every variant: 652 or 28.
Exact bytes and conservative serialization equivalence pass. Renderer policy
and parser differences both count as mismatches; a score alone cannot tell them
apart. The raw evidence keeps the example number, section, Markdown, expected
HTML, actual HTML, render error, and classification. Setup failures are
**unmeasured**, with their error retained. Unsupported suites have no percentage.
A render error is a measured failure, not an omitted input. A crashing native
worker is retried per input so it cannot hide later examples.

All variants share the [campaign HTML comparator](../native-comparison/verify.py).
It normalizes entity spelling, normal-flow whitespace, void-tag slashes,
boolean checked/disabled spelling, simple table alignment serialization, and
UTF-8 URL spelling. Code/preformatted whitespace, text, URL destinations,
escaping, IDs, classes, task state, and other attributes remain significant.
Heading-ID-only differences are diagnostic failures. A guard permits only exact
bytes for duplicate attributes and SVG/MathML, where the existing comparator
cannot safely establish equivalence. This is an HTML token comparison, not a
browser DOM proof or a claim about every possible Markdown input.

The ordinary CI benchmark-harness step validates the runner tests and
reclassifies every retained output. It checks fixture and adapter hashes,
profiles, exact counts and percentages, every example's identity, evidence
hashes, and the generated homepage data. It needs no competitor builds or
network access. A full remeasurement is required when measurement inputs change.

## Specification provenance and license

CommonMark inputs come directly from the repository's unchanged
[CommonMark 0.31.2 fixture](../../tests/spec_fixtures/README.md). GFM inputs come
from the unchanged official HTML response retrieved on September 14, 2026:
[provenance and SHA-256](../compatibility-audit/README.md#fixture-provenance).
The older 24-example GFM fixture and inherited conformance baselines are unchanged.

John MacFarlane's CommonMark specification and the GFM specification based on it
are licensed under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
The specification inputs and expected outputs reproduced in reports retain that
license and attribution, separately from this repository's MIT source license.
