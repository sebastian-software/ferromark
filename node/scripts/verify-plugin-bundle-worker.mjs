/* eslint-disable import/no-extraneous-dependencies -- Copied into a clean project containing the installed tarball. */
import { availableCodePlugins, CodeCompiler, toHtml } from "ferromark";
import pkg from "ferromark/package.json" with { type: "json" };
import assert from "node:assert/strict";
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
  const notices = JSON.parse(
    readFileSync(
      new URL("node_modules/ferromark/THIRD_PARTY_NOTICES/packages.json", import.meta.url),
      "utf8",
    ),
  );
  const fonts = notices.find((entry) => entry.name === "ferrigura-text");
  assert.ok(fonts.files.some((file) => file.startsWith("fonts/")));
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
