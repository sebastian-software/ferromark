// The landing page states measured figures only. They come from one file that
// `scripts/publish-native-readme.py` derives from one archived native comparison
// report per platform, so the homepage, the benchmark guide and the README
// cannot drift apart.
import benchmarks from "../data/native-benchmarks.json";

export { benchmarks as nativeBenchmarks };

export type NativeBenchmarkPlatform = (typeof benchmarks.platforms)[number];
export type NativeBenchmarkFigure = NativeBenchmarkPlatform["figures"][number];

/** The landing page states one decimal: 2.07 becomes "2.1×". */
export function formatSpeed(value: number): string {
  return `${value.toFixed(1)}×`;
}

/** The engines the landing page states, with the names readers know them by. */
const landingFigures = [
  ["pulldown-cmark", "pulldown-cmark"],
  ["md4c", "md4c"],
  ["v1", "Ferromark v1"],
  ["bun", "Bun's native engine"],
] as const;

export function nativeBenchmarkFigure(
  platform: NativeBenchmarkPlatform,
  id: string,
): NativeBenchmarkFigure {
  const figure = platform.figures.find((candidate) => candidate.id === id);
  if (!figure) {
    throw new Error(`Unknown native benchmark figure: ${id}`);
  }
  return figure;
}

/** "50", or "50 on Apple Silicon and 49 on Linux x86-64" when the agreement sets differ. */
export function agreementDocuments(): string {
  const counts = benchmarks.platforms.map((platform) => platform.documents.fiveEngineAgreement);
  if (counts.every((count) => count === counts[0])) {
    return String(counts[0]);
  }
  return benchmarks.platforms
    .map((platform) => `${platform.documents.fiveEngineAgreement} on ${platform.label}`)
    .join(" and ");
}

export function NativeBenchmarkFigures() {
  return (
    <div className="landing-platforms">
      {benchmarks.platforms.map((platform) => (
        <div className="landing-platform" key={platform.id}>
          <div className="landing-platform-head">
            <h3>{platform.label}</h3>
            <p>
              {/* "GitHub-hosted runner, AMD EPYC 9V74": host and CPU each on their own line. */}
              {platform.machine.split(", ").map((part) => (
                <span key={part}>
                  {part}
                  <br />
                </span>
              ))}
              {platform.measured} · <code>{platform.revision}</code>
            </p>
          </div>
          <dl className="landing-figures">
            {landingFigures.map(([id, label]) => {
              const figure = nativeBenchmarkFigure(platform, id);
              return (
                <div key={figure.id}>
                  <dt>{label}</dt>
                  <dd>{formatSpeed(figure.fresh)}</dd>
                </div>
              );
            })}
          </dl>
        </div>
      ))}
    </div>
  );
}
