import { type ChangeEvent, useState } from "ardo/react";
import { FerraBenchmarkTable, FerraSelect } from "ferramenta-family/ferra";

import conformance from "../data/comparison-conformance.json";
import { groups, platforms, reportUrl } from "./benchmark-data";
import { formatSpeed } from "./native-benchmarks";

export { benchmarkLead, benchmarkRange } from "./benchmark-data";

const conformanceRows = new Map(conformance.rows.map((row) => [row.id, row]));

type SuiteResult = {
  status: string;
  passed?: number;
  total?: number;
  percent?: number;
  reason?: string;
};

function ConformanceResult({
  value,
  name,
  label,
  evidence,
}: {
  value?: SuiteResult;
  name: "CM" | "GFM";
  label: string;
  evidence?: string;
}) {
  if (value?.status !== "measured" || typeof value.percent !== "number") {
    const status = value?.status === "unsupported" ? "Unsupported" : "Unmeasured";
    return (
      <span title={value?.reason ?? status}>
        {name} {status}
      </span>
    );
  }
  const fullName = name === "CM" ? "CommonMark 0.31.2" : "GFM extensions";
  const detail = `${label}: ${fullName}, ${value.passed}/${value.total} examples (${value.percent.toFixed(2)}%). View raw evidence and configuration.`;
  return (
    <a href={reportUrl(`${conformance.report}/${evidence}`)} title={detail} aria-label={detail}>
      {name} {value.percent.toFixed(1)}%
    </a>
  );
}

function ConformanceCells({ id, label }: { id: string; label: string }) {
  const row = conformanceRows.get(id);
  return (
    <>
      <td className="ferra-benchmark-conformance">
        <ConformanceResult
          value={row?.suites.commonmark}
          name="CM"
          label={label}
          evidence={row?.evidence}
        />
      </td>
      <td className="ferra-benchmark-conformance">
        <ConformanceResult
          value={row?.suites.gfm}
          name="GFM"
          label={label}
          evidence={row?.evidence}
        />
      </td>
    </>
  );
}

function LibraryName({
  row,
  differs = false,
}: {
  row: (typeof groups)[number]["rows"][number];
  differs?: boolean;
}) {
  const backend = row.backend.replace(" (syntax subset)", "");
  return (
    <th scope="row">
      <span className="ferra-benchmark-name">
        <a className="ferra-benchmark-source" href={row.github} title={`${row.label} · ${backend}`}>
          {row.label}
        </a>
        {differs && <sup aria-label="Output differs for some inputs">*</sup>}
      </span>
      <span className="ferra-benchmark-detail">{backend}</span>
    </th>
  );
}

export function BenchmarkComparison() {
  const [selected, setSelected] = useState(platforms[0].id);
  return (
    <div id="markdown-ecosystem">
      <FerraSelect
        label="Platform"
        options={platforms.map((platform) => ({ value: platform.id, label: platform.label }))}
        value={selected}
        onChange={(event: ChangeEvent<HTMLSelectElement>) => {
          setSelected(event.currentTarget.value);
        }}
        aria-controls="markdown-ecosystem-results"
      />
      <div id="markdown-ecosystem-results">
        {platforms.map((platform, index) => (
          <PlatformComparison
            key={platform.id}
            platform={platform}
            index={index}
            hidden={selected !== platform.id}
          />
        ))}
      </div>
    </div>
  );
}

function SpeedRow({ row, index }: { row: (typeof groups)[number]["rows"][number]; index: number }) {
  const result = row.results[index];
  return (
    <tr>
      <LibraryName row={row} differs={result?.outputDifferences} />
      <td>
        {result ? (
          <a
            href={reportUrl(result.report)}
            title={result.evidence}
            aria-label={`Ferromark has ${result.speed.toFixed(1)} times the throughput of ${row.label} on ${platforms[index].label}. View measurement report.`}
          >
            {formatSpeed(result.speed)}
          </a>
        ) : (
          <span aria-label="Not measured">—</span>
        )}
      </td>
    </tr>
  );
}

function PlatformComparison({
  platform,
  index,
  hidden,
}: {
  platform: (typeof platforms)[number];
  index: number;
  hidden: boolean;
}) {
  return (
    <div hidden={hidden}>
      <FerraBenchmarkTable
        caption={`Ferromark speedup over each library · ${platform.label} · fresh calls`}
      >
        <thead>
          <tr>
            <th scope="col">Compared with</th>
            <th scope="col">Speedup</th>
          </tr>
        </thead>
        {groups.map((group) => (
          <tbody key={group.id} aria-labelledby={`comparison-${platform.id}-${group.id}`}>
            <tr>
              <th scope="rowgroup" colSpan={2} id={`comparison-${platform.id}-${group.id}`}>
                {group.label}
              </th>
            </tr>
            {group.rows.map((row) => (
              <SpeedRow key={row.id} row={row} index={index} />
            ))}
          </tbody>
        ))}
        <SpeedFootnotes index={index} />
      </FerraBenchmarkTable>
    </div>
  );
}

function SpeedFootnotes({ index }: { index: number }) {
  return (
    <tfoot>
      <tr>
        <td colSpan={2}>
          2× means twice the throughput. Node.js includes binding overhead. Every factor includes
          all 57 frozen inputs.
        </td>
      </tr>
      {groups.some((group) => group.rows.some((row) => row.results[index]?.outputDifferences)) && (
        <tr>
          <td colSpan={2}>
            * Output differs for some of the 57 inputs on this platform. All inputs contribute to
            the performance factor. See the linked reports for syntax and API differences.
          </td>
        </tr>
      )}
    </tfoot>
  );
}

/** Specification agreement is measured separately from the performance profiles. */
export function ConformanceComparison() {
  return (
    <FerraBenchmarkTable caption="Specification agreement · public spec settings">
      <thead>
        <tr>
          <th scope="col">Library</th>
          <th scope="col">CommonMark</th>
          <th scope="col">GFM ext.</th>
        </tr>
      </thead>
      {groups.map((group) => (
        <tbody key={group.id} aria-labelledby={`conformance-${group.id}`}>
          <tr>
            <th scope="rowgroup" colSpan={3} id={`conformance-${group.id}`}>
              {group.label}
            </th>
          </tr>
          <tr className="ferromark-reference-row">
            <th scope="row">
              Ferromark <span className="ferra-benchmark-detail">Spec reference</span>
            </th>
            <ConformanceCells
              id={group.id === "native" ? "ferromark-native" : "ferromark-node"}
              label={`Ferromark (${group.label})`}
            />
          </tr>
          {group.rows.map((row) => (
            <tr key={row.id}>
              <LibraryName row={row} />
              <ConformanceCells id={row.id} label={`${row.label} (${row.runtime})`} />
            </tr>
          ))}
        </tbody>
      ))}
      <tfoot>
        <tr>
          <td colSpan={3}>
            CM: 652 CommonMark 0.31.2 examples. GFM ext.: 28 extension examples, not full GFM. Spec
            settings and output agreement are separate from timing.{" "}
            <a href={reportUrl(conformance.report)}>Method and raw results</a>.
          </td>
        </tr>
      </tfoot>
    </FerraBenchmarkTable>
  );
}
