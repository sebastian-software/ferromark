import completedBenchmarks from "../data/benchmark-platform-values.json";
import projects from "../data/benchmark-projects.json";
import conformance from "../data/comparison-conformance.json";
import ecosystemBenchmarks from "../data/markdown-ecosystem-benchmarks.json";
import { formatSpeed, nativeBenchmarks } from "./native-benchmarks";

const platforms = [
  ...new Map(
    [
      ...nativeBenchmarks.platforms.map((platform) => ({ id: platform.id, label: platform.label })),
      ...completedBenchmarks.figures.map((figure) => ({
        id: figure.platform,
        label: figure.platformLabel,
      })),
    ].map((platform) => [platform.id, platform]),
  ).values(),
];
const reportUrl = (report: string) =>
  `https://github.com/sebastian-software/ferromark/tree/main/${report}`;
const ecosystemFigures = new Map(ecosystemBenchmarks.figures.map((figure) => [figure.id, figure]));

type CompletedFigure = {
  scoringScope?: string;
  agreeingDocuments?: number;
  overviewReport?: string;
} & Omit<
  (typeof completedBenchmarks.figures)[number],
  "agreeingDocuments" | "overviewReport" | "scoringScope"
>;

const completedFigures = new Map<string, CompletedFigure>(
  completedBenchmarks.figures.map((figure) => [`${figure.platform}/${figure.id}`, figure]),
);

/** The overall claim must hold for every displayed library on every platform. */
export function benchmarkLead(): string {
  for (const project of projects) {
    for (const platform of platforms) {
      const figure = completedFigures.get(`${platform.id}/${project.id}`);
      if (
        !figure ||
        !Number.isFinite(figure.fresh) ||
        figure.fresh <= 1 ||
        figure.documents !== 57 ||
        figure.corpusDocuments !== 57 ||
        figure.scoringScope !== "all-documents"
      ) {
        throw new Error("An overall speed claim needs complete measurements above the baseline");
      }
    }
  }
  return "Ahead of every measured library.";
}

const measuredHosts = new Map(
  completedBenchmarks.figures.map((figure) => [
    figure.platform,
    `${figure.platformLabel}: ${figure.machine.split(", ")[0]} · ${figure.measured}`,
  ]),
);
export const benchmarkHosts = platforms
  .map((platform) => measuredHosts.get(platform.id))
  .filter(Boolean)
  .join("; ");

const nativePlatforms = new Map(
  nativeBenchmarks.platforms.map((platform) => [platform.id, platform]),
);

const nativeFiguresByPlatform = new Map(
  nativeBenchmarks.platforms.map((platform) => [
    platform.id,
    new Map(platform.figures.map((figure) => [figure.id, figure])),
  ]),
);

function completedResult(completed: CompletedFigure) {
  return {
    speed: completed.fresh,
    outputDifferences:
      completed.scoringScope === "all-documents" &&
      (completed.agreeingDocuments ?? completed.documents) < completed.corpusDocuments,
    report: completed.overviewReport ?? completed.report,
    evidence: `${completed.documents}/${completed.corpusDocuments} ${completed.scoringScope === "all-documents" ? `timed documents · ${completed.agreeingDocuments} equivalent outputs` : "documents · equivalent HTML"} · ${completed.profileScope} · ${completed.platformLabel} · ${completed.machine} · ${completed.measured} · ${completed.revision}`,
  };
}

const rows = [...projects]
  .sort((left, right) => left.label.localeCompare(right.label, "en", { sensitivity: "base" }))
  .map((project) => ({
    ...project,
    results: platforms.map((platform) => {
      const completed = completedFigures.get(`${platform.id}/${project.id}`);
      if (completed) {
        return completedResult(completed);
      }
      const ecosystem = ecosystemFigures.get(project.id);
      if (ecosystem && platform.id === "macos-arm64") {
        return {
          speed: ecosystem.fresh,
          outputDifferences: false,
          report: ecosystemBenchmarks.report,
          evidence: `${ecosystem.documents}/${ecosystem.corpusDocuments} documents · equivalent HTML · ${ecosystem.profileScope} · macOS arm64 · ${ecosystemBenchmarks.measured} · ${ecosystem.revision}`,
        };
      }
      const historical = nativePlatforms.get(platform.id);
      const native =
        project.runtime === "Native"
          ? nativeFiguresByPlatform.get(platform.id)?.get(project.id)
          : undefined;
      return native && historical
        ? {
            speed: native.fresh,
            outputDifferences: false,
            report: historical.report,
            evidence: `${native.documents} equivalent documents · ${platform.label} · ${historical.machine} · ${historical.measured} · ${historical.revision}`,
          }
        : null;
    }),
  }));

const groups = [
  { id: "native", label: "Native", rows: rows.filter((row) => row.runtime === "Native") },
  { id: "node", label: "Node.js", rows: rows.filter((row) => row.runtime === "Node.js") },
];

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

function ConformanceCell({ id, label }: { id: string; label: string }) {
  const row = conformanceRows.get(id);
  return (
    <td className="ferromark-conformance">
      <ConformanceResult
        value={row?.suites.commonmark}
        name="CM"
        label={label}
        evidence={row?.evidence}
      />
      <ConformanceResult
        value={row?.suites.gfm}
        name="GFM"
        label={label}
        evidence={row?.evidence}
      />
    </td>
  );
}

function ReferenceRow({ runtime }: { runtime: string }) {
  const id = runtime === "Native" ? "ferromark-native" : "ferromark-node";
  return (
    <tr className="ferromark-reference-row">
      <th scope="row">
        Ferromark <span className="ferromark-project-backend">Spec reference</span>
      </th>
      <td className="ferromark-reference-note" colSpan={platforms.length}>
        Spec configuration
      </td>
      <ConformanceCell id={id} label={`Ferromark (${runtime})`} />
    </tr>
  );
}

function ComparisonRow({ row }: { row: (typeof rows)[number] }) {
  const backend = row.backend.replace(" (syntax subset)", "");
  return (
    <tr>
      <th scope="row">
        <span className="ferromark-project-name">
          <a
            className="ferromark-project-link"
            href={row.github}
            title={`${row.label} · ${backend}`}
          >
            {row.label}
          </a>
          {row.results.some((result) => result?.outputDifferences) && (
            <sup aria-label="Output differs for some inputs">*</sup>
          )}
        </span>
        <span className="ferromark-project-backend">{backend}</span>
      </th>
      {row.results.map((result, index) => (
        <td key={platforms[index].id}>
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
      ))}
      <ConformanceCell id={row.id} label={`${row.label} (${row.runtime})`} />
    </tr>
  );
}

export function BenchmarkComparison() {
  return (
    <table className="ferromark-comparison" id="markdown-ecosystem">
      <caption>Ferromark speedup over each library · fresh calls</caption>
      <thead>
        <tr>
          <th scope="col">Compared with</th>
          {platforms.map((platform) => (
            <th scope="col" key={platform.id}>
              {platform.label}
            </th>
          ))}
          <th scope="col">CM / GFM ext.</th>
        </tr>
      </thead>
      {groups.map((group) => (
        <tbody key={group.id} aria-labelledby={`comparison-${group.id}`}>
          <tr className="ferromark-comparison-group">
            <th scope="rowgroup" colSpan={platforms.length + 2} id={`comparison-${group.id}`}>
              {group.label}
            </th>
          </tr>
          <ReferenceRow runtime={group.label} />
          {group.rows.map((row) => (
            <ComparisonRow key={row.id} row={row} />
          ))}
        </tbody>
      ))}
      <tfoot>
        <tr>
          <td colSpan={platforms.length + 2}>
            CM: 652 CommonMark 0.31.2 examples. GFM ext.: 28 extension examples, not full GFM. Spec
            settings and output agreement are separate from timing.{" "}
            <a href={reportUrl(conformance.report)}>Method and raw results</a>.
          </td>
        </tr>
        {rows.some((row) => row.results.some((result) => result?.outputDifferences)) && (
          <tr>
            <td colSpan={platforms.length + 2}>
              * Output differs for some of the 57 inputs on at least one shown platform. All inputs
              contribute to the performance factor. See the linked reports for syntax and API
              differences.
            </td>
          </tr>
        )}
      </tfoot>
    </table>
  );
}
