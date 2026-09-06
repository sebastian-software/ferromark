import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { ContractError, readRepositoryFile } from "./lib/contracts.mjs";

function failContract(message) {
  throw new ContractError(message);
}

function sourceBlock(source, pattern, description) {
  const block = source.match(pattern)?.[1];
  if (block === undefined) {
    failContract(`cannot read ${description}`);
  }
  return block;
}

function rustOptions(rust) {
  const structBody = sourceBlock(
    rust,
    /pub struct Options \{([\s\S]*?)^\}/m,
    "Rust Options fields",
  );
  const defaultBody = rust.match(
    /impl Default for Options \{\s*fn default\(\) -> Self \{\s*Self \{([\s\S]*?)\n        \}\n    \}/,
  )?.[1];
  if (defaultBody === undefined) {
    failContract("cannot read Rust Options defaults");
  }
  const fields = [...structBody.matchAll(/^    pub (\w+): ([^,]+),$/gm)].map((match) => match[1]);
  const defaults = new Map(
    [...defaultBody.matchAll(/^            (\w+): (.+),$/gm)].map((match) => [match[1], match[2]]),
  );
  if (fields.length === 0) {
    failContract("Rust Options must declare at least one field");
  }
  const missingDefaults = fields.filter((field) => !defaults.has(field));
  if (missingDefaults.length > 0) {
    failContract(`Rust Options fields without defaults: ${missingDefaults.join(", ")}`);
  }
  const extraDefaults = [...defaults.keys()].filter((field) => !fields.includes(field));
  if (extraDefaults.length > 0) {
    failContract(`Rust defaults without fields: ${extraDefaults.join(", ")}`);
  }
  return new Map(fields.map((field) => [field, defaults.get(field)]));
}

function camelCase(field) {
  return field
    .split("_")
    .map((part, index) => (index === 0 ? part : part.charAt(0).toUpperCase() + part.slice(1)))
    .join("");
}

function documentedDefault(value) {
  switch (value) {
    case "true":
      return "on";
    case "false":
      return "off";
    case "None":
      return "unset";
    case "RenderPolicy::Untrusted":
      return "`'untrusted'`";
    default:
      return failContract(`unsupported Rust Options default ${JSON.stringify(value)}`);
  }
}

function nodeOptionDocs(types) {
  const body = sourceBlock(
    types,
    /export interface Options \{([\s\S]*?)^\}/m,
    "Node Options declaration",
  );
  return new Map(
    [...body.matchAll(/\/\*\*([\s\S]*?)\*\/\s*(\w+)\?:/g)].map((match) => [match[2], match[1]]),
  );
}

function nativeOptionFields(native) {
  const body = sourceBlock(native, /pub struct Options \{([\s\S]*?)^\}/m, "native Options fields");
  return [...body.matchAll(/^    pub (\w+): /gm)].map((match) => match[1]);
}

function javascriptOptionFields(javascript) {
  const body = sourceBlock(
    javascript,
    /const optionKeys = new Set\(\[([\s\S]*?)^\]\)/m,
    "JavaScript option-key set",
  );
  return [...body.matchAll(/^  '([^']+)',$/gm)].map((match) => match[1]);
}

function listWithAnd(items) {
  if (items.length === 1) {
    return items[0];
  }
  if (items.length === 2) {
    return items.join(" and ");
  }
  return `${items.slice(0, -1).join(", ")}, and ${items[items.length - 1]}`;
}

function validate(types, readme, rust, native, javascript) {
  const rustFields = rustOptions(rust);
  const nodeDocs = nodeOptionDocs(types);
  const nativeFields = nativeOptionFields(native);
  const javascriptFields = javascriptOptionFields(javascript);
  const expectedNodeFields = [...rustFields.keys()].map(camelCase);

  if ([...nodeDocs.keys()].sort().join() !== [...expectedNodeFields].sort().join()) {
    failContract(`Node Options field mismatch: expected ${expectedNodeFields.join(", ")}`);
  }
  if (nativeFields.slice().sort().join() !== [...rustFields.keys()].sort().join()) {
    failContract(`native Options field mismatch: expected ${[...rustFields.keys()].join(", ")}`);
  }
  if (javascriptFields.slice().sort().join() !== [...expectedNodeFields].sort().join()) {
    failContract(`JavaScript option-key mismatch: expected ${expectedNodeFields.join(", ")}`);
  }

  for (const [rustField, defaultValue] of rustFields) {
    const nodeField = camelCase(rustField);
    const documentation = nodeDocs.get(nodeField);
    const documented = documentedDefault(defaultValue);
    if (!documentation.includes(`Default: ${documented}`)) {
      failContract(`${nodeField} TSDoc must state default ${documented}`);
    }
    if (!native.includes(`options.${rustField}`)) {
      failContract(`native mapping missing ${rustField}`);
    }
  }

  if (!readme.includes("[`Options`](./index.d.mts)")) {
    failContract("README must link Options declaration");
  }
  const withDefault = (value) =>
    [...rustFields.entries()]
      .filter(([, defaultValue]) => defaultValue === value)
      .map(([field]) => `\`${camelCase(field)}\``);
  const enabled = withDefault("true");
  const policy = withDefault("RenderPolicy::Untrusted");
  const unset = withDefault("None");
  if (policy.length !== 1) {
    failContract("README summary requires one untrusted policy default");
  }
  if (unset.length !== 1) {
    failContract("README summary requires one unset option default");
  }
  const expectedDefaults = `Defaults on: ${listWithAnd(enabled)}. All other boolean syntax extensions default off; ${policy[0]} defaults to \`'untrusted'\` and ${unset[0]} is unset.`;
  if (!readme.includes(expectedDefaults)) {
    failContract("README must state source-derived defaults");
  }
  if (!readme.includes("require `tables`")) {
    failContract("README must document table constraints");
  }
}

const types = readRepositoryFile("node/ferromark/index.d.mts");
const readme = readRepositoryFile("node/ferromark/README.md");
const rust = readRepositoryFile("src/lib.rs");
const native = readRepositoryFile("node/native/src/lib.rs");
const javascript = readRepositoryFile("node/ferromark/index.mjs");

const futureField = rust
  .replace(
    "    pub link_base_path: Option<Box<str>>,",
    "    pub link_base_path: Option<Box<str>>,\n    pub future_option: bool,",
  )
  .replace(
    "            link_base_path: None,",
    "            link_base_path: None,\n            future_option: false,",
  );
const futureWithDocs = types.replace(
  "  /**\n   * Prefix internal absolute link destinations",
  "  /** Future option. Default: off. */\n  futureOption?: boolean\n  /**\n   * Prefix internal absolute link destinations",
);

describe("node options docs contract", () => {
  it("accepts the current option documentation", () => {
    validate(types, readme, rust, native, javascript);
  });

  it("rejects a stale documented default", () => {
    assert.throws(
      () =>
        validate(types.replace("Default: on", "Default: off"), readme, rust, native, javascript),
      ContractError,
    );
  });

  it("rejects a README without the Options link", () => {
    assert.throws(
      () =>
        validate(
          types,
          readme.replaceAll("[`Options`](./index.d.mts)", "Options"),
          rust,
          native,
          javascript,
        ),
      ContractError,
    );
  });

  it("rejects an undocumented future option", () => {
    assert.throws(() => validate(types, readme, futureField, native, javascript), ContractError);
  });

  it("rejects a future option missing from the native binding", () => {
    assert.throws(
      () => validate(futureWithDocs, readme, futureField, native, javascript),
      ContractError,
    );
  });

  it("rejects a missing JavaScript option key", () => {
    assert.throws(
      () => validate(types, readme, rust, native, javascript.replace("  'linkBasePath',\n", "")),
      ContractError,
    );
  });
});
