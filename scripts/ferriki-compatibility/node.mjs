import assert from "node:assert/strict";
import { cp, mkdir, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { createHighlighterCoreSync, ferrikiVersion } from "@ferriki/core";

const [casesPath, facadePath, output] = process.argv.slice(2);
const cases = JSON.parse(await readFile(casesPath, "utf8"));
assert.equal(ferrikiVersion(), cases.ferrikiVersion, "published native version");
const { toHtmlWithHighlighter } = await import(pathToFileURL(facadePath).href);
const highlighter = createHighlighterCoreSync();
highlighter.loadLanguageSync(cases.customLanguage);
highlighter.loadThemeSync(cases.customTheme);

function tokenResult(source, result, scopes) {
  return {
    tokens: result.tokens.map((line) =>
      line.map((token) => {
        assert.equal(
          source.slice(token.offset, token.offset + token.content.length),
          token.content,
        );
        return {
          content: token.content,
          offset: Buffer.byteLength(source.slice(0, token.offset), "utf8"),
          ...(token.color === undefined ? {} : { color: token.color }),
          ...(token.fontStyle === undefined ? {} : { fontStyle: token.fontStyle }),
          ...(token.type === undefined ? {} : { type: token.type }),
          ...(scopes
            ? { scopeNames: token.explanation[0].scopes.map((scope) => scope.scopeName) }
            : {}),
        };
      }),
    ),
    fg: result.fg,
    bg: result.bg,
    themeName: result.themeName,
  };
}

function tokenCase(case_) {
  const { id, code, lang, theme } = case_;
  try {
    return {
      id,
      tokens: tokenResult(
        code,
        highlighter.codeToTokens(code, { lang, theme, includeExplanation: "scopeName" }),
        true,
      ),
      typedTokens: tokenResult(
        code,
        highlighter.codeToTokens(code, { lang, theme, includeExplanation: "tokenType" }),
        false,
      ),
      html: highlighter.codeToHtml(code, { lang, theme }),
    };
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
  return { tokens: cases.tokens.map(tokenCase), markdown: cases.markdown.map(markdownCase) };
}

const result = pass();
const languages = highlighter.getLoadedLanguages().sort();
const themes = highlighter.getLoadedThemes().sort();
assert.deepEqual(pass(), result, "reuse must preserve token and Markdown output");
assert.deepEqual(highlighter.getLoadedLanguages().sort(), languages);
assert.deepEqual(highlighter.getLoadedThemes().sort(), themes);
result.reuse = { equal: true };

// Public Rust exposes one theme at a time. Preserve the Node-only dual-theme
// behavior independently; it is not silently presented as a parity claim.
const dual = cases.nodeDualTheme;
const dualTokens = highlighter.codeToTokens(dual.code, { lang: dual.lang, themes: dual.themes });
assert.ok(dualTokens.tokens.flat().every((token) => token.variants.light && token.variants.dark));
result.dualThemeHtml = highlighter.codeToHtml(dual.code, { lang: dual.lang, themes: dual.themes });
assert.match(result.dualThemeHtml, /--shiki-dark/);

// Corrupt only fresh package copies. This exercises the published facade and
// native decoder, while the installed package and its cache remain intact.
const require = createRequire(import.meta.url);
const core = dirname(require.resolve("@ferriki/core/package.json"));
const modules = join(dirname(fileURLToPath(import.meta.url)), "node_modules");
async function assetError(name, mutate) {
  const root = join(output, "node-assets", name);
  await mkdir(root, { recursive: true });
  await symlink(
    modules,
    join(root, "node_modules"),
    process.platform === "win32" ? "junction" : "dir",
  );
  const copy = join(root, "core");
  await cp(core, copy, { recursive: true });
  const theme = join(copy, "assets/shiki/themes/nord.fktheme");
  await mutate(theme);
  const api = await import(pathToFileURL(join(copy, "index.mjs")).href);
  const failing = api.createHighlighterCoreSync();
  let observed;
  const html = toHtmlWithHighlighter("```rust\n<code> & text\n```", failing, {
    theme: "nord",
    onHighlightError: (error) => {
      observed = error;
    },
  });
  assert.ok(observed, `${name} must fail rather than highlight`);
  assert.equal(observed.code, "ERR_ASSET");
  assert.match(
    observed.message,
    name === "missing" ? /Failed to read/ : /unsupported asset format version/,
  );
  assert.match(html, /&lt;code&gt; &amp; text/);
  failing.dispose();
  return { html, code: observed.code };
}
result.assetErrors = {
  missing: await assetError("missing", (path) => rm(path)),
  format: await assetError("format", async (path) => {
    const bytes = await readFile(path);
    bytes[0] = 2;
    await writeFile(path, bytes);
  }),
};
highlighter.dispose();
console.log(JSON.stringify(result, null, 2));
