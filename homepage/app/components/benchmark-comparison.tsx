import completedBenchmarks from "../data/benchmark-platform-values.json";
import projects from "../data/benchmark-projects.json";
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

/** Marketing ranges use the same selected measurements as the table. */
export function benchmarkRange(ids: string[]): string {
  const selected = completedBenchmarks.figures.filter((figure) => ids.includes(figure.id));
  if (
    selected.length !== ids.length * platforms.length ||
    selected.some(
      (figure) =>
        figure.fresh <= 1 || figure.documents !== 57 || figure.scoringScope !== "all-documents",
    )
  ) {
    throw new Error("A speed claim needs complete measurements above the baseline");
  }
  const values = selected.map((figure) => figure.fresh);
  const low = Math.min(...values).toFixed(1);
  const high = Math.max(...values).toFixed(1);
  return low === high ? `${low}×` : `${low}–${high}×`;
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

const rows = projects.map((project) => ({
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

function ComparisonRow({ row }: { row: (typeof rows)[number] }) {
  return (
    <tr>
      <th scope="row">
        <a
          className="ferromark-project-link"
          href={row.github}
          title={`${row.label} · ${row.backend}`}
        >
          {row.label}
        </a>
        <span className="ferromark-project-backend">{row.backend}</span>
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
              {result.outputDifferences && <sup aria-label="Output differs for some inputs">*</sup>}
            </a>
          ) : (
            <span aria-label="Not measured">—</span>
          )}
        </td>
      ))}
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
        </tr>
      </thead>
      {groups.map((group) => (
        <tbody key={group.id} aria-labelledby={`comparison-${group.id}`}>
          <tr className="ferromark-comparison-group">
            <th scope="rowgroup" colSpan={platforms.length + 1} id={`comparison-${group.id}`}>
              {group.label}
            </th>
          </tr>
          {group.rows.map((row) => (
            <ComparisonRow key={row.id} row={row} />
          ))}
        </tbody>
      ))}
      {rows.some((row) => row.results.some((result) => result?.outputDifferences)) && (
        <tfoot>
          <tr>
            <td colSpan={platforms.length + 1}>
              * Same 57 inputs, different output for some documents. All inputs contribute to the
              performance factor. See the linked reports for syntax and API differences.
            </td>
          </tr>
        </tfoot>
      )}
    </table>
  );
}
