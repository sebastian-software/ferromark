// Public Markdown-to-HTML APIs; configuration is retained outside timing.
export async function createNodeRender(engine, gfm) {
  if (engine === "marked") {
    const { Marked } = await import("marked");
    const parser = new Marked({ gfm, breaks: false, pedantic: false, async: false });
    // Public tokenizer override: match the lane with literal URL autolinking off.
    parser.use({ tokenizer: { url: () => undefined } });
    return (source) => parser.parse(source);
  }
  if (engine === "markdown-it") {
    const { default: MarkdownIt } = await import("markdown-it");
    const parser = new MarkdownIt("commonmark", { html: true, linkify: false, typographer: false });
    parser.validateLink = () => true; // Same trusted-HTML/protocol contract as Ferromark.
    if (gfm) {
      const { default: tasks } = await import("markdown-it-task-lists");
      parser.enable(["table", "strikethrough"]).use(tasks);
    }
    return (source) => parser.render(source);
  }
  if (engine === "remark") {
    const { remark } = await import("remark");
    const { default: remarkRehype } = await import("remark-rehype");
    const { default: rehypeStringify } = await import("rehype-stringify");
    const processor = remark();
    if (gfm) {
      const { gfmTable } = await import("micromark-extension-gfm-table");
      const { gfmStrikethrough } = await import("micromark-extension-gfm-strikethrough");
      const { gfmTaskListItem } = await import("micromark-extension-gfm-task-list-item");
      const { gfmTableFromMarkdown } = await import("mdast-util-gfm-table");
      const { gfmStrikethroughFromMarkdown } = await import("mdast-util-gfm-strikethrough");
      const { gfmTaskListItemFromMarkdown } = await import("mdast-util-gfm-task-list-item");
      processor.data("micromarkExtensions", [gfmTable(), gfmStrikethrough(), gfmTaskListItem()]);
      processor.data("fromMarkdownExtensions", [
        gfmTableFromMarkdown(),
        gfmStrikethroughFromMarkdown(),
        gfmTaskListItemFromMarkdown(),
      ]);
    }
    processor
      .use(remarkRehype, { allowDangerousHtml: true })
      .use(rehypeStringify, { allowDangerousHtml: true })
      .freeze();
    return (source) => String(processor.processSync(source));
  }
  if (engine === "showdown") {
    const { default: showdown } = await import("showdown");
    const converter = new showdown.Converter({
      ellipsis: false,
      noHeaderId: true,
      ghCodeBlocks: true,
      tables: gfm,
      strikethrough: gfm,
      tasklists: gfm,
      simplifiedAutoLink: false,
      literalMidWordUnderscores: false,
      simpleLineBreaks: false,
      parseImgDimensions: false,
      headerLevelStart: 1,
      encodeEmails: false,
      openLinksInNewWindow: false,
      metadata: false,
      completeHTMLDocument: false,
    });
    return (source) => converter.makeHtml(source);
  }
  if (engine === "commonmark") {
    if (gfm) throw new Error("commonmark.js supports the CommonMark-only lane");
    const commonmark = await import("commonmark");
    const parser = new commonmark.Parser({ smart: false });
    const renderer = new commonmark.HtmlRenderer({ safe: false, softbreak: "\n" });
    return (source) => renderer.render(parser.parse(source));
  }
  if (engine === "remarkable") {
    if (gfm)
      throw new Error("Remarkable uses the CommonMark-only lane; no task-list plugin is installed");
    const { Remarkable } = await import("remarkable");
    const parser = new Remarkable("commonmark", { html: true, breaks: false, typographer: false });
    return (source) => parser.render(source);
  }
  if (engine === "markdown-exit" || engine === "markdown-it-ts") {
    const { default: MarkdownIt } = await import(engine);
    const parser = new MarkdownIt("commonmark", {
      html: true,
      linkify: false,
      typographer: false,
      ...(engine === "markdown-it-ts"
        ? { experimental: { stream: false, fullChunkedFallback: false } }
        : {}),
    });
    parser.validateLink = () => true;
    if (gfm) {
      const { default: tasks } = await import("markdown-it-task-lists");
      parser.enable(["table", "strikethrough"]).use(tasks);
    }
    return (source) => parser.render(source);
  }
  if (engine === "satteri") {
    const { markdownToHtml } = await import("satteri");
    const options = {
      features: {
        gfm: gfm ? { footnotes: false } : false,
        frontmatter: false,
        math: false,
        headingAttributes: false,
        directive: false,
        superscript: false,
        subscript: false,
        wikilinks: false,
        definitionList: false,
        smartPunctuation: false,
        rawHtml: false,
      },
    };
    // Public GFM enables autolinks too; retain that difference rather than rewriting HTML.
    return (source) => markdownToHtml(source, options).html;
  }
  if (engine === "md4x-napi" || engine === "md4x-wasm") {
    const api = await import(engine === "md4x-napi" ? "md4x/napi" : "md4x/wasm");
    if (engine === "md4x-wasm") {
      const { readFileSync } = await import("node:fs");
      const bytes = readFileSync(new URL("./node_modules/md4x/build/md4x.wasm", import.meta.url));
      await api.init({
        wasm: bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength),
      });
    } else await api.init();
    const options = { headingIds: false, full: false, heal: false };
    // The released public API cannot disable its parser extensions.
    return (source) => api.renderToHtml(source, options);
  }
  if (engine === "ox-content-napi") {
    const { parseAndRender } = await import("@ox-content/napi");
    const options = {
      gfm: false,
      mdx: false,
      footnotes: false,
      tables: gfm,
      strikethrough: gfm,
      taskLists: gfm,
      autolinks: false,
      superscript: false,
      subscript: false,
      smartPunctuation: false,
      math: false,
      definitionLists: false,
      headingAttributes: false,
      wikiLinks: false,
    };
    // Renderer builtins (IDs, TOC, callouts, URL transforms) remain part of this public API.
    return (source) => parseAndRender(source, options).html;
  }
  throw new Error(`Unknown Node competitor: ${engine}`);
}
