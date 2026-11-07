/*
 * The homepage's figures, derived from the repository's generated data. Nothing
 * here is typed by hand except which libraries the overview names; a new
 * campaign that breaks a claim the copy makes fails the build instead of
 * quietly changing its meaning.
 */
import platformValues from "../data/benchmark-platform-values.json";
import projects from "../data/benchmark-projects.json";
import conformance from "../data/comparison-conformance.json";
import samples from "../data/homepage-samples.json";

export { samples };

/** The best-known alternatives the chart and the agreement table name; the guides list all. */
export const featured = {
  native: ["pulldown-cmark", "comrak", "cmark"],
  node: ["marked", "markdown-it", "remark"],
  conformance: [
    "ferromark-native",
    "ferromark-node",
    "pulldown-cmark",
    "comrak",
    "cmark",
    "marked",
    "markdown-it",
    "remark",
  ],
} as const;

export type Runtime = "Native" | "Node.js";
export type Figure = {
  id: string;
  label: string;
  backend: string;
  runtime: Runtime;
  fresh: number;
  reuse: number;
  /** Some of the library's outputs differ from Ferromark's for the timed documents. */
  differs: boolean;
};

const projectById = new Map(projects.map((project) => [project.id, project]));

export const platforms = [
  ...new Map(platformValues.figures.map((figure) => [figure.platform, figure.platformLabel])),
].map(([id, label]) => ({ id, label }));

const first = platformValues.figures[0];
export const measured = {
  date: first.measured,
  documents: first.corpusDocuments,
};

for (const figure of platformValues.figures) {
  if (
    figure.measured !== measured.date ||
    figure.sampling.rounds !== 3 ||
    figure.corpusDocuments !== measured.documents ||
    figure.documents !== measured.documents ||
    figure.scoringScope !== "all-documents" ||
    !figure.overviewReport.includes("blacksmith")
  ) {
    throw new Error(
      "The homepage describes one three-round Blacksmith campaign over the full corpus",
    );
  }
}

/** Every library led on every platform: the copy says Ferromark led all of them. */
export const libraryCount = projects.length;
for (const project of projects) {
  for (const platform of platforms) {
    const figure = platformValues.figures.find(
      (candidate) => candidate.id === project.id && candidate.platform === platform.id,
    );
    if (figure === undefined || !Number.isFinite(figure.fresh) || figure.fresh <= 1) {
      throw new Error(`Ferromark must lead ${project.id} on ${platform.id} for the homepage claim`);
    }
  }
}
// The copy spells these out ("twenty libraries", "two machines").
if (libraryCount !== 20 || platforms.length !== 2) {
  throw new Error("Update the homepage copy: it names twenty libraries on two machines");
}

export function libraryFigure(platform: string, id: string): Figure {
  const found = platformValues.figures.find(
    (candidate) => candidate.platform === platform && candidate.id === id,
  );
  const project = projectById.get(id);
  if (found === undefined || project === undefined) {
    throw new Error(`Missing benchmark figure ${platform}/${id}`);
  }
  return {
    id,
    label: project.label,
    backend: project.backend,
    runtime: project.runtime === "Native" ? "Native" : "Node.js",
    fresh: found.fresh,
    reuse: found.reuse,
    differs: found.agreeingDocuments < found.corpusDocuments,
  };
}

export const platformLabel = (id: string) =>
  platforms.find((platform) => platform.id === id)?.label ?? id;

/* ---------- Specification agreement ---------- */

type Suite = { status: string; passed?: null | number; total?: null | number };
export type Agreement = { passed: number; total: number } | null;
const agreement = (suite: Suite): Agreement =>
  suite.status === "measured" && typeof suite.passed === "number" && typeof suite.total === "number"
    ? { passed: suite.passed, total: suite.total }
    : null;

export const conformanceDate = conformance.measured.slice(0, 10);
export const suiteLabels = conformance.suiteLabels;
export const agreementRows = featured.conformance.map((id) => {
  const row = conformance.rows.find((candidate) => candidate.id === id);
  if (row === undefined) throw new Error(`Missing conformance row ${id}`);
  return {
    id,
    label: row.label,
    runtime: row.runtime,
    own: id.startsWith("ferromark-"),
    commonmark: agreement(row.suites.commonmark),
    gfm: agreement(row.suites.gfm),
  };
});

const own = agreementRows[0];
if (own.commonmark === null || own.gfm === null)
  throw new Error("Ferromark's agreement is missing");
export const ownAgreement = { commonmark: own.commonmark, gfm: own.gfm };
