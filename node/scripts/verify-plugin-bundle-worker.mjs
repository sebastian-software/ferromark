/* eslint-disable import/no-extraneous-dependencies -- Copied into a clean project containing the installed tarball. */
import { availableCodePlugins, CodeCompiler, toHtml } from "ferromark";
import pkg from "ferromark/package.json" with { type: "json" };
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const names = availableCodePlugins();
assert.deepEqual(names, pkg.ferromark.plugins);
assert.equal(pkg.private, true);
assert.equal(pkg.optionalDependencies, undefined);
assert.equal(pkg.scripts, undefined);
assert.equal(toHtml("# Installed"), '<h1 id="installed">Installed</h1>\n');
const plugins = names.map((name) =>
  name === "ferriki"
    ? {
        name,
        options: {
          languages: [
            {
              name: "fixture",
              scopeName: "source.fixture",
              patterns: [{ match: "word", name: "keyword" }],
            },
          ],
          theme: {
            name: "fixture",
            settings: [
              { settings: { foreground: "#101010", background: "#ffffff" } },
              { scope: "keyword", settings: { foreground: "#bb1100" } },
            ],
          },
          assets: { remote: false },
        },
      }
    : { name, options: { idPrefix: "installed" } },
);
let compiler = new CodeCompiler({ plugins });
const source = '```mermaid\npie\n"A" : 1\n```\n\n```fixture\nword\n```';
const html = compiler.toHtml(source);
const jsx = compiler.compile(source).body;
if (names.includes("ferrigura")) {
  assert.match(html, /<svg /);
  assert.match(jsx, /<svg /);
  assert.match(jsx, /className=/);
  assert.doesNotMatch(jsx, /dangerouslySetInnerHTML/);
  const fixtures = JSON.parse(
    readFileSync(new URL("mermaid-fixtures.json", import.meta.url), "utf8"),
  );
  const diagrams = new CodeCompiler({
    plugins: [{ name: "ferrigura", options: { idPrefix: "fixture" } }],
  });
  for (const fixture of fixtures) {
    const markdown = `\`\`\`mermaid\n${fixture.source}\n\`\`\``;
    const rendered = diagrams.toHtml(markdown);
    assert.equal(createHash("sha256").update(rendered).digest("hex"), fixture.sha256, fixture.name);
    assert.equal(rendered, diagrams.toHtml(markdown));
    assert.equal(diagrams.prepare(markdown).render().body, diagrams.compile(markdown).body);
  }
  const broken = "```mermaid\nfutureDiagram\nA --> B\n```";
  assert.equal(diagrams.toHtml(broken), toHtml(broken));
  assert.equal(diagrams.takeDiagnostics()[0].start, 0);
  assert.deepEqual(diagrams.takeDiagnostics(), []);
  const notices = JSON.parse(
    readFileSync(
      new URL("node_modules/ferromark/THIRD_PARTY_NOTICES/packages.json", import.meta.url),
      "utf8",
    ),
  );
  const fonts = notices.find((entry) => entry.name === "ferrigura-text");
  assert.ok(fonts.files.some((file) => file.startsWith("fonts/")));
  const css = notices.find((entry) => entry.name === "cssparser");
  assert.equal(css.license, "MPL-2.0");
  assert.ok(css.files.some((file) => /LICENSE/i.test(file)));
  assert.equal(
    css.sourceArchive,
    "https://static.crates.io/crates/cssparser/cssparser-0.37.0.crate",
  );
} else assert.doesNotMatch(html, /<svg /);
if (names.includes("ferriki")) assert.match(html, /#bb1100/i);
const prepared = compiler.prepare(source);
compiler = null;
assert.equal(compiler, null);
globalThis.gc();
assert.equal(prepared.render().codeBlocks.length, 2);
assert.throws(() => new CodeCompiler({ plugins: [{ name: "fixture" }] }), /not available/);
process.stdout.write(
  `Installed plugin bundle passed: ${pkg.ferromark.target} [${names.join(", ")}]\n`,
);
