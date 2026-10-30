import type { ComponentPropsWithoutRef } from "ardo/react";

// Ardo 4's published URLs remain aliases of Ardo 6's native heading slugs.
// Keep these in presentation so generated benchmark content stays reproducible.
const aliases: Record<string, string | undefined> = {
  "node-js-projects": "nodejs-projects",
  "prepare-once-in-node-js": "prepare-once-in-nodejs",
  "to-component": "tocomponent",
  "node-js-ecosystem-coverage": "nodejs-ecosystem-coverage",
  "node-js": "nodejs",
};

export function DocumentHeading({ id, ...props }: ComponentPropsWithoutRef<"h2">) {
  const alias = id === undefined ? undefined : aliases[id];
  return (
    <>
      {alias !== undefined && <span id={alias} />}
      <h2 {...props} id={id} />
    </>
  );
}
