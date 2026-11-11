// Match the repository's stable and -rc.N release versions. The generator
// produces the samples; this checks their recorded versions and visible attribution.
const number = (value) => /^(?:0|[1-9]\d*)$/.test(value);

function exactVersion(value) {
  if (typeof value !== "string") return false;
  const [core, rc, extra] = value.split("-rc.");
  const parts = core.split(".");
  return (
    extra === undefined &&
    parts.length === 3 &&
    parts.every((part) => number(part)) &&
    (rc === undefined || number(rc))
  );
}

export function verifyHomepageSamples(samples, homepage) {
  const versions = samples?.renderedWith;
  for (const name of ["ferromark", "@ferriki/core", "@ferrigura/core"]) {
    if (!exactVersion(versions?.[name])) {
      throw new Error(`Sample ${name} must record a stable or rc release version`);
    }
  }
  for (const label of [
    `Real output of ferromark ${versions.ferromark}`,
    `@ferriki/core ${versions["@ferriki/core"]}, theme ${samples.code.theme}`,
    `@ferrigura/core ${versions["@ferrigura/core"]}, static SVG`,
  ]) {
    if (!homepage.includes(label)) {
      throw new Error(
        `Prerendered homepage is missing sample attribution ${JSON.stringify(label)}`,
      );
    }
  }
}
