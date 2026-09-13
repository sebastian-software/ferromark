# The complete native engine field on practical documents

This follow-up measures the same frozen documentation collection as the
[workflow suite](../README.md), with all engines in the existing published
comparisons. Each operation renders all twelve files and either releases each
owned HTML result or keeps every result until the collection ends. Both choices
include their output allocation and lifetime container in the timer.

The field includes Ferromark, pulldown-cmark, Comrak, Bun's native Markdown code,
md4c, cmark, cmark-gfm, Goldmark, Sätteri, Rushdown, Markdig, markdown-rs, and Ox
Content. Dependency locks and upstream revisions are inherited from the
existing checked-in comparison adapters. The frozen document text is unchanged.

## Completed work before timing

Every engine renders every input with both lifetimes before admission. An engine
receives collection timings only if **all twelve** documents do comparable work.
The original HTML, options, and every failed input remain in the archive. There
is no partial corpus selected by speed or by output agreement.

The existing task/table presentation review still applies. The field additionally
allows Ox Content's extra generated heading IDs: this is additional rendered
navigation, not missing content or a disabled feature. Only the `id` on heading
elements is projected out for this documented case; heading levels, text, other
attributes, links, code, tables, and task states remain significant. IDs must be
nonempty and unique. The historical microbenchmarks retain their original rules.
cmark's core-only API is exercised with its supported flags and cannot complete
files needing GFM tables/tasks/strikethrough; it is shown as not comparable.

This broader panel measures the trusted HTML collection. Full untrusted-preview
and front-matter/heading-result integrations are implemented and measured for
Ferromark, pulldown-cmark, and Comrak in the parent suite. A native HTML-only API
is not silently counted as completing those extra jobs. Missing integration
adapters are a limitation of this panel, not proof that an engine cannot be
integrated that way.

## Runtime and process memory

Each native worker uses its public parser/renderer APIs, fresh document state,
and normal owned HTML output. Rust/C use their normal allocators; Go and .NET
retain automatic GC. Markdig returns native UTF-16 strings. Counts validate those
native code units without adding UTF-8 serialization to only one candidate.
Input loading, startup, configuration, verification, JSON/IPC, and reporting are
outside timing. Go and .NET GC work occurring in the timed window is included.

Bun's pinned native support and mimalloc build use a distinct worker environment
and a separately measured Ferromark baseline. The stable/system and Bun/nightly/
mimalloc panels are not combined into a global speed ranking.

Three fresh process rounds per engine and output lifetime each warm for three
seconds and collect 80 rotating/reversing windows of at least 63 ms. The timer
checks the clock after four complete collections. The headline time is the median
of the three round medians; all windows and round variation are retained.

`wait4` obtains the kernel's whole-process **maximum resident set size** after
each timing worker exits. On the initial macOS host `ru_maxrss` is in bytes.
This needs no counting allocator and does not instrument the renderer. Publish
the median and full range of all three process peaks. The scope includes startup,
warmup, input, worker infrastructure, JIT/runtime, stacks, allocator/GC reserves,
and the repeated workload. RSS is not incremental parser heap, live requested
allocation size, single-request memory, or a concurrency capacity estimate.
The parent's precise Rust heap figures remain separately labeled evidence.

The table compares the selected native integrations, not identical scratch
lifetimes: Ferromark calls fresh `to_html_with_options`; Ox retains HTML-renderer
scratch while rebuilding its parser and arena/AST for every document. Its owned
HTML moves out of the renderer and is dropped by the workload. The
[lifecycle and cache audit](../../../docs/reports/2026-09-13-ox-workflow-study/REPORT.md)
measures both fresh and reusable renderers, verifies state isolation against
fresh processes, and checks sensitivity to changing content and input addresses.

## Reproduce

Use the existing harness instructions to obtain the pinned md4c, cmark and
cmark-gfm sources and prepare a **dedicated** Bun workspace with its native support
archives. Bun preparation's pinned compiler and allocator remain unchanged.
Go 1.27.1 and .NET SDK 10.0.401 are required; .NET is published self-contained.
The initial builder/measurement host is macOS on Apple Silicon.

Commit the worker source before making a publication build. The builder records
source/adapter hashes, locks, commands, upstream revisions, executable hashes,
and build logs. Set the following arguments to your dedicated source directories:

```sh
python3 -m unittest discover -s benchmarks/workflows/field -p 'test_*.py'
python3 benchmarks/workflows/field/prepare.py /private/tmp/workflow-field-build \
  --bun /private/tmp/prepared-bun \
  --md4c /private/tmp/md4c --cmark /private/tmp/cmark --cmark-gfm /private/tmp/cmark-gfm \
  --dotnet /path/to/dotnet
python3 benchmarks/workflows/field/run.py /private/tmp/workflow-field-build /private/tmp/workflow-field-check --verify-only
python3 benchmarks/workflows/field/run.py /private/tmp/workflow-field-build docs/reports/NEW-FIELD-RUN
python3 benchmarks/workflows/field/publish.py docs/reports/NEW-FIELD-RUN
python3 benchmarks/workflows/field/publish.py docs/reports/NEW-FIELD-RUN --check
```

Use fresh result directories, keep AC power connected, and run no competing
builds, tests, or benchmarks while measuring. The publisher verifies archive
checksums, complete-collection admission, all timing counts/durations/output
lengths, rotation, and all RSS observations before generating figures.
