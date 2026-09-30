import completedBenchmarks from "../data/benchmark-platform-values.json";
import projects from "../data/benchmark-projects.json";
import ecosystemBenchmarks from "../data/markdown-ecosystem-benchmarks.json";
import { formatSpeed, nativeBenchmarks } from "./native-benchmarks";

const platforms = nativeBenchmarks.platforms;
const reportUrl = (report: string) =>
  `https://github.com/sebastian-software/ferromark/tree/main/${report}`;
const ecosystemFigures = new Map(ecosystemBenchmarks.figures.map((figure) => [figure.id, figure]));

const completedFigures = new Map(
  completedBenchmarks.figures.map((figure) => [`${figure.platform}/${figure.id}`, figure]),
);

const nativeFiguresByPlatform = platforms.map(
  (platform) => new Map(platform.figures.map((figure) => [figure.id, figure])),
);

const rows = projects.map((project) => ({
  ...project,
  results: platforms.map((platform, index) => {
    const completed = completedFigures.get(`${platform.id}/${project.id}`);
    if (completed) {
      return {
        speed: completed.fresh,
        report: completed.report,
        evidence: `${completed.documents}/${completed.corpusDocuments} documents · equivalent HTML · ${completed.profileScope} · ${completed.platformLabel} · ${completed.machine} · ${completed.measured} · ${completed.revision}`,
      };
    }
    const ecosystem = ecosystemFigures.get(project.id);
    if (ecosystem && platform.id === "macos-arm64") {
      return {
        speed: ecosystem.fresh,
        report: ecosystemBenchmarks.report,
        evidence: `${ecosystem.documents}/${ecosystem.corpusDocuments} documents · equivalent HTML · ${ecosystem.profileScope} · macOS arm64 · ${ecosystemBenchmarks.measured} · ${ecosystem.revision}`,
      };
    }
    const native =
      project.runtime === "Native" ? nativeFiguresByPlatform[index].get(project.id) : undefined;
    return native
      ? {
          speed: native.fresh,
          report: platform.report,
          evidence: `${native.documents} equivalent documents · ${platform.label} · ${platform.machine} · ${platform.measured} · ${platform.revision}`,
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
        <a className="ferromark-project-link" href={row.github}>
          {row.label}
        </a>
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
