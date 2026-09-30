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
  throw new Error(`Unknown Node competitor: ${engine}`);
}
