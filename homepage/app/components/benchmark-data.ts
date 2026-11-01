import completedBenchmarks from "../data/benchmark-platform-values.json";
import projects from "../data/benchmark-projects.json";
import ecosystemBenchmarks from "../data/markdown-ecosystem-benchmarks.json";
import { nativeBenchmarks } from "./native-benchmarks";

export const platforms = [
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
export const reportUrl = (report: string) =>
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

/** The homepage examples come from the same complete campaigns as the detail tables. */
export function benchmarkRange(id: string): string {
  const measured = platforms.map((platform) => {
    const value = completedFigures.get(`${platform.id}/${id}`)?.fresh;
    if (value === undefined || !Number.isFinite(value) || value <= 1) {
      throw new Error("A homepage example needs measured speedups on every platform");
    }
    return value;
  });
  const low = Math.min(...measured).toFixed(1);
  const high = Math.max(...measured).toFixed(1);
  return low === high ? `${low}×` : `${low}–${high}×`;
}

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

export const groups = [
  { id: "native", label: "Native", rows: rows.filter((row) => row.runtime === "Native") },
  { id: "node", label: "Node.js", rows: rows.filter((row) => row.runtime === "Node.js") },
];
