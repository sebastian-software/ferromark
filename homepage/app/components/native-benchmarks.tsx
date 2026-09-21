// The landing page states measured figures only. They come from one file that
// `scripts/publish-native-readme.py` derives from the archived native comparison
// report, so the homepage, the benchmark guide and the README cannot drift apart.
import benchmarks from "../data/native-benchmarks.json";

export { benchmarks as nativeBenchmarks };

export type NativeBenchmarkFigure = (typeof benchmarks.figures)[number];

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

export function nativeBenchmarkFigure(id: string): NativeBenchmarkFigure {
  const figure = benchmarks.figures.find((candidate) => candidate.id === id);
  if (!figure) {
    throw new Error(`Unknown native benchmark figure: ${id}`);
  }
  return figure;
}

export function NativeBenchmarkFigures() {
  return (
    <dl className="landing-figures">
      {landingFigures.map(([id, label]) => {
        const figure = nativeBenchmarkFigure(id);
        return (
          <div key={figure.id}>
            <dt>{label}</dt>
            <dd>{formatSpeed(figure.fresh)}</dd>
          </div>
        );
      })}
    </dl>
  );
}
