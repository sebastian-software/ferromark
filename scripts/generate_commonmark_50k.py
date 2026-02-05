#!/usr/bin/env python3
from __future__ import annotations

from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES_DIR = ROOT / "benches" / "fixtures"

SAMPLES = [
    ("commonmark-5k.md", 5 * 1024),
    ("commonmark-20k.md", 20 * 1024),
    ("commonmark-50k.md", 50 * 1024),
]


def build_article(target_bytes: int, label: str) -> str:
    parts: list[str] = []
    current_bytes = 0

    header = (
        "<!-- Generated on "
        f"{date.today().isoformat()} to ~{label}. Synthetic wiki-style article. "
        "Do not edit by hand. -->\n\n"
    )
    parts.append(header)
    current_bytes += len(header.encode("utf-8"))

    def add(text: str) -> None:
        nonlocal current_bytes
        parts.append(text)
        current_bytes += len(text.encode("utf-8"))

    def add_para(text: str) -> None:
        add(text.rstrip("\n") + "\n\n")

    def compact_base_blocks() -> list[str]:
        return [
            (
                "# Northbridge District\n\n"
                "Northbridge District is a fictional metropolitan area used in technical "
                "documentation to describe planning, governance, and infrastructure patterns.\n\n"
                "The article is written in a Wikipedia-like style, with many paragraphs, "
                "lists, and code blocks. It also demonstrates CommonMark features such as "
                "reference links, autolinks, inline HTML, and blockquotes.\n\n"
            ),
            (
                "Overview\n========\n\n"
                "Northbridge sits along a river delta and connects industrial corridors to "
                "residential zones. The local charter emphasizes long-term planning and "
                "public access.\n\n"
                "Key characteristics:\n\n"
                "- Mixed-use zoning across historic corridors\n"
                "- **High-frequency** transit loops with *steady* headways\n"
                "- Public archives with long-term audit trails\n"
                "  - Urban catalog revisions\n"
                "  - Civic ledger backups\n\n"
            ),
            (
                "## History\n\n"
                "Settlement began near the riverbank warehouses, followed by rail expansion "
                "and market squares. Records refer to the \\*operator\\* policy and "
                "Northbridge&nbsp;District as a formal term.\n\n"
                "> \"We planned for continuity as much as for growth.\" — Committee Minutes, 1987\n\n"
                "### Milestones\n\n"
                "1. 1889 — First bridge completed.\n"
                "2. 1957 — Flood response plan adopted.\n"
                "3. 1984 — Density plan revised.\n\n"
            ),
            (
                "## Infrastructure\n\n"
                "Operators publish schedules at <https://example.org/northbridge/transit> "
                "and accept inquiries at <mailto:info@example.org>.\n\n"
                "In maintenance guides, a hard line break is used to separate shift notes.  \n"
                "This line continues on the next row with explicit formatting.\n\n"
            ),
            (
                "## Notes on Formatting\n\n"
                "Some records use character entities like &copy; and &amp; to preserve "
                "licensing text. Backslash escapes such as \\[brackets\\] and \\_underscores\\_ "
                "appear in transcripts.\n\n"
                "Inline tags like <span class=\"label\">draft</span> show up in civic memos.\n\n"
                "<div class=\"infobox\">\n"
                "<p><strong>Northbridge Gallery</strong></p>\n"
                "<p>Founded: 1979</p>\n"
                "</div>\n\n"
            ),
            (
                "## Education and Research\n\n"
                "A representative data transformation is included below:\n\n"
                "```rust\n"
                "fn normalize_score(score: i32) -> i32 {\n"
                "    if score < 0 { 0 } else { score }\n"
                "}\n"
                "```\n\n"
                "Legacy memos still use indented code blocks for configuration samples:\n\n"
                "    [archive]\n"
                "    retention_years = 25\n"
                "    checksum = true\n\n"
            ),
            (
                "## See also\n\n"
                "- [Regional planning overview](https://example.org/region)\n"
                "- [Northbridge data portal][portal]\n"
                "- ![District map](https://example.org/assets/map.png \"Map\")\n\n"
                "---\n\n"
            ),
        ]

    def full_base_blocks() -> list[str]:
        blocks = compact_base_blocks()
        blocks.extend(
            [
                (
                    "## Geography\n\n"
                    "Northbridge occupies a low-lying delta bordered by tidal flats and a "
                    "steep northern ridge. The main river splits into distributaries that "
                    "form natural boundaries between neighborhoods.\n\n"
                    "### Neighborhoods\n\n"
                    "- Harbor Row\n"
                    "- Eastbank\n"
                    "- Civic Plateau\n"
                    "- Archive Ward\n"
                    "  - Old Registry\n"
                    "  - Survey Annex\n\n"
                ),
                (
                    "## Climate\n\n"
                    "The climate is temperate with long wet seasons and mild summers. "
                    "Annual rainfall supports an extensive greenway system along the river.\n\n"
                ),
                (
                    "## Demographics\n\n"
                    "The population is diverse, with several long-established communities "
                    "and a steady influx of students and researchers.\n\n"
                ),
                (
                    "## Economy\n\n"
                    "The local economy centers on logistics, planning services, and archival "
                    "technology. Small manufacturing remains present along the rail corridor.\n\n"
                    "Several firms specialize in planning software. One dataset is described "
                    "in the [Municipal Archive][archive].\n\n"
                ),
                (
                    "## Culture\n\n"
                    "Northbridge has a strong public arts program, with murals documenting "
                    "planning eras and neighborhood transitions.\n\n"
                ),
                (
                    "## Governance\n\n"
                    "The governing council publishes a yearly plan outlining priorities for "
                    "public transit, housing stability, and record preservation.\n\n"
                    "The most cited policy is the [Civic Charter][charter].\n\n"
                    "### Council structure\n\n"
                    "- Chair\n"
                    "- Planning director\n"
                    "- Archive commissioner\n\n"
                ),
                (
                    "## Media\n\n"
                    "Local media outlets focus on infrastructure, planning decisions, and "
                    "archival releases. A common phrase in editorials is **\"plan for today\"**.\n\n"
                ),
            ]
        )
        return blocks

    base_blocks = compact_base_blocks() if target_bytes <= 8 * 1024 else full_base_blocks()
    for block in base_blocks:
        add(block)

    add("## Supplementary Notes\n\n")

    def block_transit(idx: int) -> str:
        return (
            f"### Transit Loop {idx}\n\n"
            "The loop schedule is coordinated with river traffic and freight windows. "
            "Operators track **turnaround time** and *dwell time* at major stations.\n\n"
            "- Peak interval: 6 minutes\n"
            "- Off-peak interval: 12 minutes\n"
            "- Night service: 20 minutes\n"
            "  - Limited stops at Harbor Row\n"
            "  - Signal priority near Eastbank\n\n"
        )

    def block_archives(idx: int) -> str:
        return (
            f"### Archive Protocol {idx}\n\n"
            "Archival metadata is recorded with a consistent schema. A typical entry "
            "includes `series_id`, a retention flag, and a public access marker.\n\n"
            "> Records are validated nightly, and only then indexed for search.\n"
            "> Use [Municipal Archive][archive] for official requests.\n\n"
        )

    def block_lists(idx: int) -> str:
        return (
            f"### Field Survey {idx}\n\n"
            "Survey teams use a shared checklist for neighborhood audits:\n\n"
            "1. Measure sidewalk clearance.\n"
            "2. Record lighting levels.\n"
            "3. Note accessibility markers.\n"
            "\n"
            "   - ADA ramp condition\n"
            "   - Crosswalk timing\n\n"
        )

    def block_fenced(idx: int) -> str:
        return (
            f"### Routing Script {idx}\n\n"
            "A small script is used to normalize incoming schedules:\n\n"
            "```python\n"
            "def normalize(minutes):\n"
            "    return max(0, int(minutes))\n"
            "```\n\n"
        )

    def block_indented(idx: int) -> str:
        return (
            f"### Legacy Config {idx}\n\n"
            "Older maintenance forms embed raw configuration blocks:\n\n"
            "    [loop]\n"
            "    interval = 12\n"
            "    priority = true\n\n"
        )

    def block_inline_html(idx: int) -> str:
        return (
            f"### Exhibit Note {idx}\n\n"
            "Curators often mark temporary exhibits with inline tags like "
            "<span class=\"tag\">rotating</span> to aid indexing.\n\n"
            "<aside>\n"
            "<p>Exhibit schedule updates are posted monthly.</p>\n"
            "</aside>\n\n"
        )

    def block_emphasis(idx: int) -> str:
        return (
            f"### Planning Memo {idx}\n\n"
            "Reports include escaped markers such as \\*priority\\* and "
            "\\_status\\_ in transcripts. Several lines use forced breaks.  \n"
            "The second line keeps the break visible in the output.\n\n"
        )

    def block_links(idx: int) -> str:
        return (
            f"### Public Outreach {idx}\n\n"
            "Notices refer to the [Civic Charter][charter] and an online portal at "
            "<https://example.org/northbridge/outreach>. A separate memo references "
            "[Planning Office][planning].\n\n"
        )

    def block_quote(idx: int) -> str:
        return (
            f"### Council Excerpt {idx}\n\n"
            "> We will revisit the flood plan after the next cycle.\n"
            "> - Follow-up audits\n"
            "> - Community feedback\n\n"
        )

    def block_separator(_: int) -> str:
        return "---\n\n"

    blocks = [
        block_transit,
        block_archives,
        block_lists,
        block_fenced,
        block_indented,
        block_inline_html,
        block_emphasis,
        block_links,
        block_quote,
        block_separator,
    ]

    ref_defs = (
        "[archive]: https://example.org/archive \"Municipal Archive\"\n"
        "[planning]: https://example.org/planning 'Planning Office'\n"
        "[charter]: https://example.org/charter (Civic Charter)\n"
        "[portal]: https://example.org/data \"Data Portal\"\n"
    )
    ref_defs_len = len(ref_defs.encode("utf-8"))

    idx = 1
    while current_bytes < target_bytes - ref_defs_len:
        block = blocks[(idx - 1) % len(blocks)]
        add(block(idx))
        idx += 1

    add(ref_defs)

    return "".join(parts)


def main() -> None:
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)
    for filename, target in SAMPLES:
        label = filename.split(".")[0].replace("commonmark-", "")
        sample = build_article(target, label)
        path = FIXTURES_DIR / filename
        path.write_text(sample, encoding="utf-8")
        print(f"Wrote {path} ({path.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
