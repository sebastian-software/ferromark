import { NATIVE_TARGETS } from "ferramenta-native/targets";
import path from "node:path";
const root = import.meta.dirname;
export default {
  name: "ferromark",
  packageDir: path.join(root, "ferromark"),
  platformDir: path.join(root, "ferromark/npm"),
  packagePattern: "ferromark-{target}",
  binaryPattern: "ferromark.{target}.node",
  runtimeFile: "native-target.mjs",
  repositoryPlatformDir: "node/ferromark/npm",
  licenses: ["LICENSE", "LICENSE-MIT"],
  workspaceProtocol: true,
  pack: {
    platformRequired: ["README.md"],
    allowed: [
      "package.json",
      "LICENSE",
      "LICENSE-MIT",
      "README.md",
      "index.d.mts",
      "index.mjs",
      "native-target.mjs",
      "native-target.d.mts",
      "native.d.ts",
    ],
    maxUnpackedSize: 131_000,
  },
  consumer: {
    smoke: `import { availableCodePlugins, toHtml } from "ferromark"; if (availableCodePlugins().length !== 0) throw new Error("Standard addon must be engine-free"); if (toHtml("# Native") !== '<h1 id="native">Native</h1>\\n') throw new Error("HTML mismatch");`,
    tests: [
      "index.test.mjs",
      "native-plugins.test.mjs",
      "references.test.mjs",
      "jsx-module.test.mjs",
      "jsx-title.test.mjs",
      "jsx.test.mjs",
    ].map((file) => `test/${file}`),
  },
  release: {
    repository: "https://github.com/sebastian-software/ferromark",
    tagPrefix: "v",
    ci: {
      requiredJobs: ["fmt", "clippy", "Assemble all npm packages", "Alpine musl runtime test"],
      platformJobs: NATIVE_TARGETS.map(({ triple }) => ({
        pattern: `^native \\(${triple}\\)$`,
        label: `native (${triple})`,
      })),
    },
    cargoFiles: ["Cargo.toml", "transforms/Cargo.toml", "node/native/Cargo.toml"],
    exactCargoDependencies: [{ file: "transforms/Cargo.toml", name: "ferromark" }],
    legacyRecoveryThrough: "3.6.0",
  },
};
