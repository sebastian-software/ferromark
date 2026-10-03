import assert from "node:assert/strict";
import { cp, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";

import { createHighlighterCoreSync, ferrikiVersion } from "@ferriki/core";

const [casesPath, facadePath, output, assetRoot] = process.argv.slice(2);
const cases = JSON.parse(await readFile(casesPath, "utf8"));
assert.equal(ferrikiVersion(), cases.ferrikiVersion, "published native version");
const { toHtmlWithHighlighter } = await import(pathToFileURL(facadePath).href);
const cacheDir = join(assetRoot, "by-digest");
const highlighter = createHighlighterCoreSync({
  assets: { remote: false, cacheDir },
});
highlighter.loadLanguageSync(cases.customLanguage);
highlighter.loadThemeSync(cases.customTheme);

function highlightingCase(case_) {
  const { id, code, lang, theme } = case_;
  try {
    return { id, html: highlighter.codeToHtml(code, { lang, theme }) };
  } catch (error) {
    if (error.code !== "ERR_UNSUPPORTED") throw error;
    return { id, error: error.code };
  }
}

function markdownCase(case_) {
  const errors = [];
  const html = toHtmlWithHighlighter(case_.source, highlighter, {
    theme: case_.theme,
    onHighlightError: (error) => errors.push(error.code),
  });
  return { id: case_.id, html, errors };
}

function pass() {
  return {
    highlighting: cases.tokens.map(highlightingCase),
    markdown: cases.markdown.map(markdownCase),
  };
}

const result = pass();
const languages = highlighter.getLoadedLanguages().sort();
const themes = highlighter.getLoadedThemes().sort();
assert.deepEqual(pass(), result, "reuse must preserve HTML and Markdown output");
assert.deepEqual(highlighter.getLoadedLanguages().sort(), languages);
assert.deepEqual(highlighter.getLoadedThemes().sort(), themes);
result.reuse = { equal: true };

// Dual-theme HTML remains part of the public Node contract. This is not a
// Rust parity claim because the Rust API accepts one theme at a time.
const dual = cases.nodeDualTheme;
result.dualThemeHtml = highlighter.codeToHtml(dual.code, {
  lang: dual.lang,
  themes: dual.themes,
});
assert.match(result.dualThemeHtml, /--shiki-dark/);

// Exercise the public offline cache contract. The main peer run uses a fresh
// cache populated only after each CDN payload matches the release manifest.
const require = createRequire(import.meta.url);
const core = dirname(require.resolve("@ferriki/core/package.json"));
const release = JSON.parse(await readFile(join(assetRoot, "release-manifest.json"), "utf8"));
const nord = release.assets["themes/nord.fktheme"];

async function assetError(name, corrupt) {
  const isolatedCache = join(output, "node-cache", name);
  await cp(cacheDir, isolatedCache, { recursive: true });
  const nordPath = join(isolatedCache, nord.sha256);
  if (corrupt) {
    const payload = await readFile(join(assetRoot, "themes/nord.fktheme"));
    payload[0] ^= 0xff;
    await writeFile(nordPath, payload);
  } else {
    await rm(nordPath);
  }
  const api = await import(pathToFileURL(join(core, "index.mjs")).href);
  const failing = api.createHighlighterCoreSync({
    assets: { remote: false, cacheDir: isolatedCache },
  });
  let observed;
  const html = toHtmlWithHighlighter("```rust\n<code> & text\n```", failing, {
    theme: "nord",
    onHighlightError: (error) => {
      observed = error;
    },
  });
  assert.ok(observed, `${name} must fail rather than highlight`);
  assert.equal(observed.code, "ERR_ASSET");
  assert.match(html, /&lt;code&gt; &amp; text/);
  failing.dispose();
  return { html, code: observed.code };
}

result.assetErrors = {
  missingCache: await assetError("missing", false),
  corruptCache: await assetError("corrupt", true),
};
assert.match(highlighter.codeToHtml("fn main() {}", { lang: "rust", theme: "nord" }), /<span/);
highlighter.dispose();
console.log(JSON.stringify(result, null, 2));
