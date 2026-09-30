import ecosystemBenchmarks from "../data/markdown-ecosystem-benchmarks.json";
import { formatSpeed, nativeBenchmarks } from "./native-benchmarks";

const platforms = nativeBenchmarks.platforms;
const reportUrl = (report: string) =>
  `https://github.com/sebastian-software/ferromark/tree/main/${report}`;

const figuresByPlatform = platforms.map(
  (platform) => new Map(platform.figures.map((figure) => [figure.id, figure])),
);
const nativeFigures = [
  ...new Map(
    platforms.flatMap((platform) => platform.figures).map((figure) => [figure.id, figure]),
  ).values(),
];

const libraryLabels = new Map([
  ["bun", "Bun MD"],
  ["ox-content", "OX-Content"],
]);

const rows = [
  ...nativeFigures
    .filter((figure) => figure.id !== "v1")
    .map((figure) => ({
      id: figure.id,
      label: libraryLabels.get(figure.id) ?? figure.label,
      runtime: "Native",
      results: platforms.map((platform, index) => {
        const result = figuresByPlatform[index].get(figure.id);
        return result
          ? {
              speed: result.fresh,
              report: platform.report,
              evidence: `${result.documents} equivalent documents · ${platform.label} · ${platform.machine} · ${platform.measured} · ${platform.revision}`,
            }
          : null;
      }),
    })),
  ...ecosystemBenchmarks.figures.map((figure) => ({
    id: figure.label,
    label: figure.label,
    runtime: figure.runtime,
    results: platforms.map((platform) =>
      platform.id === "macos-arm64"
        ? {
            speed: figure.fresh,
            report: ecosystemBenchmarks.report,
            evidence: `${figure.documents}/${figure.corpusDocuments} documents · equivalent HTML · macOS arm64 · ${ecosystemBenchmarks.measured} · ${figure.revision}`,
          }
        : null,
    ),
  })),
];

const groups = [
  { id: "native", label: "Native", rows: rows.filter((row) => row.runtime === "Native") },
  { id: "node", label: "Node.js", rows: rows.filter((row) => row.runtime === "Node.js") },
];

function ComparisonRow({ row }: { row: (typeof rows)[number] }) {
  return (
    <tr>
      <th scope="row">{row.label}</th>
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
              {platform.id === "macos-arm64" ? "macOS arm64" : platform.label}
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
    </table>
  );
}
