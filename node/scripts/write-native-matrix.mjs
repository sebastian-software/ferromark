import { execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
const matrix = JSON.parse(
  execFileSync(process.execPath, [fileURLToPath(new URL("native-matrix.mjs", import.meta.url))], {
    encoding: "utf8",
  }),
);
appendFileSync(
  process.env.GITHUB_OUTPUT,
  `matrix=${JSON.stringify(matrix)}\nplatform-ids=${matrix.include.map((target) => target.id).join(" ")}\n`,
);
