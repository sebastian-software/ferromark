#!/usr/bin/env python3
from __future__ import annotations
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT_PATH = ROOT / "benches" / "fixtures" / "commonmark-50k.md"
TARGET_BYTES = 50 * 1024


def build_article() -> str:
    parts: list[str] = []
    current_bytes = 0
    header = (
        "<!-- Generated on "
        f"{date.today().isoformat()} to ~50KB. Synthetic wiki-style article. "
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

    title = "# Northbridge District\n\n"
    add(title)
    add_para(
        "Northbridge District is a fictional metropolitan area used in technical "
        "documentation to describe planning, governance, and infrastructure patterns. "
        "The district is known for its layered transit network, long-form planning "
        "documents, and a large archive of public records."
    )
    add_para(
        "The article is written in a Wikipedia-like style, with many paragraphs, "
        "lists, and code blocks. It also demonstrates CommonMark features such as "
        "reference links, autolinks, inline HTML, and blockquotes."
    )

    # Setext heading
    add("Overview\n========\n\n")
    add_para(
        "Northbridge sits along a wide river delta and connects multiple industrial "
        "corridors to residential zones. The local charter emphasizes long-term "
        "planning, with a focus on resilience, public access, and shared utilities."
    )
    add_para(
        "In planning documents the word *district* is often emphasized, while _region_ "
        "is used for statistical reporting. The term \\*operator\\* appears in some "
        "technical manuals, and the name is sometimes written as Northbridge&nbsp;District."
    )
    add("Key characteristics:\n\n")
    add(
        "- Mixed-use zoning across historic and modern corridors\n"
        "- Extensive transit loops with **high-frequency** service\n"
        "- Public archives and open data portals\n"
        "  - An urban catalog with yearly revisions\n"
        "  - A civic ledger with long-term audit trails\n\n"
    )

    add("## Etymology\n\n")
    add_para(
        "The name Northbridge refers to the original arched crossing built in the "
        "late 19th century. Early records used a hyphenated form, but modern usage "
        "standardized the compound name."
    )
    add_para(
        "In printed materials, the name sometimes appears in monospace as "
        "`NBR-01`, which was the archival code in the 1932 ledger."
    )

    add("## History\n\n")
    add_para(
        "Settlement began near the riverbank warehouses, followed by the expansion "
        "of rail lines and market squares. The district charter was revised "
        "multiple times to account for new transport policies."
    )
    add_para(
        "Major redevelopment occurred after the 1957 flood, which prompted a "
        "revision of building codes and a reassessment of critical infrastructure."
    )
    add_para(
        "A 1984 policy report noted that population density rose faster than the "
        "available housing stock, resulting in an emphasis on mixed-use corridors."
    )
    add(
        "> \"We planned for continuity as much as for growth, and the archives show\n"
        "> the district learning to pace itself.\" — Planning Committee Minutes, 1987\n\n"
    )
    add("### Milestones\n\n")
    add(
        "1. 1889 — First bridge completed, enabling cross-river trade.\n"
        "2. 1932 — Central record office established.\n"
        "3. 1957 — Flood response plan adopted.\n"
        "4. 1984 — Density plan revised.\n\n"
    )

    add("## Geography\n\n")
    add_para(
        "Northbridge occupies a low-lying delta bordered by tidal flats and a "
        "steep northern ridge. The main river splits into distributaries that "
        "form natural boundaries between neighborhoods."
    )
    add_para(
        "The district is subdivided into terraces, embankments, and reclaimed land. "
        "Maps typically highlight the levee system and the primary drainage basins."
    )
    add_para(
        "Key districts include Harbor Row, Eastbank, and the Civic Plateau. "
        "Each zone follows distinct land-use patterns and mobility priorities."
    )
    add("### Neighborhoods\n\n")
    add(
        "- Harbor Row\n"
        "- Eastbank\n"
        "- Civic Plateau\n"
        "- Archive Ward\n"
        "  - Old Registry\n"
        "  - Survey Annex\n\n"
    )

    add("## Climate\n\n")
    add_para(
        "The climate is temperate with long wet seasons and mild summers. "
        "Annual rainfall supports an extensive greenway system along the river."
    )
    add_para(
        "Seasonal flooding is rare after the levee upgrades, but contingency "
        "plans are maintained and updated annually."
    )

    add("## Demographics\n\n")
    add_para(
        "The population is diverse, with several long-established communities "
        "and a steady influx of students and researchers."
    )
    add_para(
        "Census summaries show a gradual increase in multi-generational housing "
        "and a shift toward mixed residential-commercial buildings."
    )

    add("## Economy\n\n")
    add_para(
        "The local economy centers on logistics, planning services, and archival "
        "technology. Small manufacturing remains present along the rail corridor."
    )
    add_para(
        "A central export is documentation: policy drafts, engineering reports, "
        "and standards used by neighboring districts."
    )
    add_para(
        "Several local firms specialize in software for municipal planning. One "
        "popular dataset is described in the [Municipal Archive][archive]."
    )

    add("## Infrastructure\n\n")
    add_para(
        "Transit relies on a looped system that connects each neighborhood in "
        "consistent intervals. Service frequency is described in the technical "
        "manuals and in the `route_sync` configuration."
    )
    add_para(
        "Operators publish schedules at <https://example.org/northbridge/transit> "
        "and accept inquiries at <mailto:info@example.org>."
    )
    add_para(
        "In maintenance guides, a hard line break is used to separate shift notes.  \n"
        "This line continues on the next row with explicit formatting."
    )

    add("## Culture\n\n")
    add_para(
        "Northbridge has a strong public arts program, with murals documenting "
        "planning eras and neighborhood transitions."
    )
    add_para(
        "The main gallery features rotating exhibits and a <span class=\"label\">"
        "research wing</span> for archival visitors."
    )
    add(
        "<div class=\"infobox\">\n"
        "<p><strong>Northbridge Gallery</strong></p>\n"
        "<p>Founded: 1979</p>\n"
        "<p>Visitors: 320,000/year</p>\n"
        "</div>\n\n"
    )

    add("## Education and Research\n\n")
    add_para(
        "The district hosts a planning institute that publishes annual reviews "
        "and a quarterly research digest."
    )
    add_para(
        "A representative data transformation is included below, as used in a "
        "local data standard:"
    )
    add(
        "```rust\n"
        "fn normalize_score(score: i32) -> i32 {\n"
        "    if score < 0 { 0 } else { score }\n"
        "}\n"
        "```\n\n"
    )
    add_para(
        "Legacy memos still use indented code blocks for configuration samples:"
    )
    add(
        "    [archive]\n"
        "    retention_years = 25\n"
        "    checksum = true\n\n"
    )

    add("## Governance\n\n")
    add_para(
        "The governing council publishes a yearly plan outlining priorities for "
        "public transit, housing stability, and record preservation."
    )
    add_para(
        "The most cited policy is the [Civic Charter][charter], a document that "
        "balances growth with long-term sustainability."
    )
    add("### Council structure\n\n")
    add(
        "- Chair\n"
        "- Planning director\n"
        "- Archive commissioner\n"
        "\n"
        "- Committee members\n"
        "\n"
        "  The committee includes representatives from each neighborhood and\n"
        "  meets quarterly to review project progress.\n\n"
    )

    add("## Media\n\n")
    add_para(
        "Local media outlets focus on infrastructure, planning decisions, and "
        "archival releases. A common phrase in editorials is **\"plan for today\"**."
    )

    add("## See also\n\n")
    add(
        "- [Regional planning overview](https://example.org/region)\n"
        "- [Northbridge data portal][portal]\n"
        "- ![District map](https://example.org/assets/map.png \"Map\")\n\n"
    )

    add("---\n\n")

    add("## Notes on Formatting\n\n")
    add_para(
        "Some records use character entities like &copy; and &amp; to preserve "
        "licensing text. Backslash escapes such as \\[brackets\\] and \\_underscores\\_ "
        "appear in legacy transcripts."
    )
    add_para(
        "Reference links are preferred in long documents. For example, the "
        "[Municipal Archive][archive] maintains historical records, while the "
        "[Planning Office][planning] publishes updated zoning maps."
    )

    # Append filler paragraphs to reach target size.
    filler = [
        "The district's reports emphasize readability, with long-form narrative "
        "sections followed by concise summaries. Staff writers tend to keep "
        "sentences short and avoid jargon where possible.",
        "Public hearings are typically documented in verbatim transcripts, but "
        "final summaries are edited for clarity and accessibility.",
        "Maintenance logs include frequent references to stormwater pumps, "
        "bridge joints, and electrical substations that keep the grid stable.",
        "Annual surveys note that residents value reliable transit and walkable "
        "corridors over short-term development gains.",
        "Several study groups compare Northbridge to other river districts, "
        "highlighting shared challenges in flood management and archival care.",
        "Planning staff often cite the 'long view' as the main operating principle "
        "for infrastructure decisions and public investment.",
    ]

    notes_heading_added = False
    idx = 0
    while current_bytes < TARGET_BYTES:
        if not notes_heading_added:
            add("## Supplementary Notes\n\n")
            notes_heading_added = True
        add_para(filler[idx % len(filler)])
        idx += 1

    # Reference definitions (CommonMark feature)
    add(
        "[archive]: https://example.org/archive \"Municipal Archive\"\n"
        "[planning]: https://example.org/planning 'Planning Office'\n"
        "[charter]: https://example.org/charter (Civic Charter)\n"
        "[portal]: https://example.org/data \"Data Portal\"\n"
    )

    return "".join(parts)


def main() -> None:
    sample = build_article()
    OUT_PATH.write_text(sample, encoding="utf-8")
    size = OUT_PATH.stat().st_size
    print(f"Wrote {OUT_PATH} ({size} bytes)")


if __name__ == "__main__":
    main()
