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

const rows = [
  ...nativeFigures.map((figure) => ({
    id: figure.id,
    label: figure.label,
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
      <tbody>
        {rows.map((row) => (
          <tr key={row.id}>
            <th scope="row">
              {row.label}
              <span className="ferromark-comparison-runtime">{row.runtime}</span>
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
        ))}
      </tbody>
    </table>
  );
}
