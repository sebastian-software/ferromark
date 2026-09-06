import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, realpathSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, it } from "node:test";

import { ContractError, repositoryPath, repositoryRoot } from "./lib/contracts.mjs";

function failContract(message) {
  throw new ContractError(`docs.rs metadata contract: ${message}`);
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: repositoryRoot,
    encoding: "utf8",
    ...options,
    env: { ...process.env, ...options.env },
  });
  if (result.error) {
    failContract(`${command} could not be started: ${result.error.message}`);
  }
  return result;
}

function cargoMetadata(manifestPath) {
  const result = run("cargo", [
    "metadata",
    "--locked",
    "--no-deps",
    "--format-version=1",
    "--manifest-path",
    manifestPath,
  ]);
  if (result.status !== 0) {
    failContract(`cargo metadata failed:\n${result.stdout}${result.stderr}`);
  }
  return JSON.parse(result.stdout);
}

function packageForManifest(metadata, manifest) {
  const canonicalManifest = realpathSync(manifest);
  const found = metadata.packages.find(
    (candidate) => realpathSync(candidate.manifest_path) === canonicalManifest,
  );
  if (!found) {
    failContract(`cargo metadata did not return ${canonicalManifest}`);
  }
  return found;
}

function docsRsTable(manifest, document) {
  const header = "[package.metadata.docs.rs]";
  const start = document.indexOf(`${header}\n`);
  if (start === -1) {
    failContract(`${manifest} must contain ${header}`);
  }
  return document.slice(start + header.length + 1).split(/^\[/m)[0];
}

function docsRsFeatures(manifest, document) {
  const match = docsRsTable(manifest, document).match(/^features\s*=\s*\[([^\]]*)\]\s*$/m);
  if (!match) {
    failContract(`${manifest} docs.rs metadata must define features`);
  }
  return [...match[1].matchAll(/"([^"]+)"/g)].map((entry) => entry[1]);
}

function docsRsNoDefaultFeatures(manifest, document) {
  const match = docsRsTable(manifest, document).match(
    /^no-default-features\s*=\s*(true|false)\s*$/m,
  );
  if (!match) {
    failContract(`${manifest} docs.rs metadata must define no-default-features`);
  }
  return match[1] === "true";
}

function assertDocsRsFeatures(metadata, manifest, document = readFileSync(manifest, "utf8")) {
  const features = docsRsFeatures(manifest, document);
  const expected = ["mdx"];
  if (
    features.length !== expected.length ||
    features.some((name, index) => name !== expected[index])
  ) {
    failContract(
      `${manifest} docs.rs features must be ${JSON.stringify(expected)}, got ${JSON.stringify(features)}`,
    );
  }
  if (!docsRsNoDefaultFeatures(manifest, document)) {
    failContract(`${manifest} docs.rs must disable default features`);
  }
  if (!("mdx" in packageForManifest(metadata, manifest).features)) {
    failContract(`${manifest} must declare the mdx feature`);
  }
}

function packageCrate(targetDirectory) {
  const result = run("cargo", [
    "package",
    "--allow-dirty",
    "--locked",
    "--offline",
    "--no-verify",
    "--target-dir",
    targetDirectory,
  ]);
  if (result.status !== 0) {
    failContract(`cargo package failed:\n${result.stdout}${result.stderr}`);
  }
  const manifest = repositoryPath("Cargo.toml");
  const packageInfo = packageForManifest(cargoMetadata(manifest), manifest);
  const crate = path.join(
    targetDirectory,
    "package",
    `${packageInfo.name}-${packageInfo.version}.crate`,
  );
  if (!statSync(crate, { throwIfNoEntry: false })?.isFile()) {
    failContract(`cargo package did not create ${crate}`);
  }
  return crate;
}

function buildMdxDocs(targetDirectory) {
  const result = run(
    "cargo",
    [
      "doc",
      "--locked",
      "--no-deps",
      "--no-default-features",
      "--features",
      "mdx",
      "--target-dir",
      targetDirectory,
    ],
    { env: { RUSTDOCFLAGS: "-D warnings" } },
  );
  if (result.status !== 0) {
    failContract(`MDX documentation build failed:\n${result.stdout}${result.stderr}`);
  }
  const page = path.join(targetDirectory, "doc", "ferromark", "mdx", "index.html");
  if (!statSync(page, { throwIfNoEntry: false })?.isFile()) {
    failContract(`MDX documentation build did not generate ${page}`);
  }
}

function extractCrate(crate, destination) {
  const result = spawnSync("tar", ["-xzf", crate, "-C", destination], { encoding: "utf8" });
  if (result.error || result.status !== 0) {
    failContract(`could not extract ${crate}: ${result.error?.message ?? result.stderr}`);
  }
}

function withTemporaryDirectory(prefix, callback) {
  const directory = mkdtempSync(path.join(tmpdir(), prefix));
  try {
    return callback(directory);
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
}

const sourceManifest = repositoryPath("Cargo.toml");
const sourceDocument = readFileSync(sourceManifest, "utf8");
const metadata = cargoMetadata(sourceManifest);

describe("docs.rs metadata contract", () => {
  it("publishes the mdx feature on docs.rs", () => {
    assertDocsRsFeatures(metadata, sourceManifest);
  });

  it("builds the mdx documentation without rustdoc warnings", () => {
    withTemporaryDirectory("ferromark-docs-rs-docs-target.", buildMdxDocs);
  });

  it("keeps the docs.rs metadata in the packaged manifest", () => {
    withTemporaryDirectory("ferromark-docs-rs-package.", (destination) => {
      withTemporaryDirectory("ferromark-docs-rs-target.", (targetDirectory) => {
        extractCrate(packageCrate(targetDirectory), destination);
        const packagedManifest = path.join(
          destination,
          `ferromark-${packageForManifest(metadata, sourceManifest).version}`,
          "Cargo.toml",
        );
        assert.ok(
          statSync(packagedManifest, { throwIfNoEntry: false })?.isFile(),
          `expected a packaged Cargo.toml at ${packagedManifest}`,
        );
        assertDocsRsFeatures(cargoMetadata(packagedManifest), packagedManifest);
      });
    });
  });

  it("rejects a missing docs.rs table", () => {
    assert.throws(
      () =>
        assertDocsRsFeatures(
          metadata,
          sourceManifest,
          sourceDocument.replace(
            '[package.metadata.docs.rs]\nfeatures = ["mdx"]\nno-default-features = true\n\n',
            "",
          ),
        ),
      ContractError,
    );
  });

  it("rejects a broader docs.rs feature set", () => {
    assert.throws(
      () =>
        assertDocsRsFeatures(
          metadata,
          sourceManifest,
          sourceDocument.replace('features = ["mdx"]', 'features = ["mdx", "profiling"]'),
        ),
      ContractError,
    );
  });

  it("rejects default features on docs.rs", () => {
    assert.throws(
      () =>
        assertDocsRsFeatures(
          metadata,
          sourceManifest,
          sourceDocument.replace("no-default-features = true", "no-default-features = false"),
        ),
      ContractError,
    );
  });

  it("rejects an undeclared mdx feature", () => {
    const withoutMdx = structuredClone(metadata);
    delete packageForManifest(withoutMdx, sourceManifest).features.mdx;
    assert.throws(() => assertDocsRsFeatures(withoutMdx, sourceManifest), ContractError);
  });
});
