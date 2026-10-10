// Spec profiles are separate from the frozen timing adapters.
import { createNodeRender } from "./node-adapters.mjs";

export async function createConformanceRender(engine, gfm) {
  if (engine === "ferromark-node") {
    const { loadBenchmarkFacade } = await import("./benchmark-facade.mjs");
    const { toHtml } = await loadBenchmarkFacade();
    const options = {
      renderPolicy: "trusted",
      allowHtml: true,
      headingIds: false,
      callouts: false,
      tables: gfm,
      strikethrough: gfm,
      taskLists: gfm,
      autolinkLiterals: gfm,
      disallowedRawHtml: gfm,
    };
    return (source) => toHtml(source, options);
  }
  if (engine === "marked") {
    const { Marked } = await import("marked");
    const parser = new Marked({ gfm, breaks: false, pedantic: false, async: false });
    return (source) => parser.parse(source);
  }
  if (engine === "markdown-it") {
    const { default: MarkdownIt } = await import("markdown-it");
    const parser = new MarkdownIt("commonmark", { html: true, linkify: gfm, typographer: false });
    parser.validateLink = () => true;
    if (gfm) {
      const { default: tasks } = await import("markdown-it-task-lists");
      parser.enable(["table", "strikethrough", "linkify"]).use(tasks);
      // linkify-it 6 disables scheme-less URLs by default. GFM includes www links.
      parser.linkify.set({ fuzzyLink: true });
    }
    return (source) => parser.render(source);
  }
  if (engine === "micromark" || engine === "remark") {
    const extensions = [],
      htmlExtensions = [],
      fromMarkdownExtensions = [];
    if (gfm) {
      const { gfmTable, gfmTableHtml } = await import("micromark-extension-gfm-table");
      const { gfmStrikethrough, gfmStrikethroughHtml } =
        await import("micromark-extension-gfm-strikethrough");
      const { gfmTaskListItem, gfmTaskListItemHtml } =
        await import("micromark-extension-gfm-task-list-item");
      const { gfmAutolinkLiteral, gfmAutolinkLiteralHtml } =
        await import("micromark-extension-gfm-autolink-literal");
      const { gfmTagfilterHtml } = await import("micromark-extension-gfm-tagfilter");
      extensions.push(gfmTable(), gfmStrikethrough(), gfmTaskListItem(), gfmAutolinkLiteral());
      htmlExtensions.push(
        gfmTableHtml(),
        gfmStrikethroughHtml(),
        gfmTaskListItemHtml(),
        gfmAutolinkLiteralHtml(),
        gfmTagfilterHtml(),
      );
      if (engine === "remark") {
        const { gfmTableFromMarkdown } = await import("mdast-util-gfm-table");
        const { gfmStrikethroughFromMarkdown } = await import("mdast-util-gfm-strikethrough");
        const { gfmTaskListItemFromMarkdown } = await import("mdast-util-gfm-task-list-item");
        const { gfmAutolinkLiteralFromMarkdown } = await import("mdast-util-gfm-autolink-literal");
        fromMarkdownExtensions.push(
          gfmTableFromMarkdown(),
          gfmStrikethroughFromMarkdown(),
          gfmTaskListItemFromMarkdown(),
          gfmAutolinkLiteralFromMarkdown(),
        );
      }
    }
    if (engine === "micromark") {
      const { micromark } = await import("micromark");
      const options = {
        allowDangerousHtml: true,
        allowDangerousProtocol: true,
        extensions,
        htmlExtensions,
      };
      return (source) => micromark(source, options);
    }
    const { remark } = await import("remark");
    const { default: remarkRehype } = await import("remark-rehype");
    const { default: rehypeStringify } = await import("rehype-stringify");
    const processor = remark()
      .data("micromarkExtensions", extensions)
      .data("fromMarkdownExtensions", fromMarkdownExtensions)
      .use(remarkRehype, { allowDangerousHtml: true })
      .use(rehypeStringify, { allowDangerousHtml: true })
      .freeze();
    // Raw HAST nodes pass through unchanged; this pipeline has no tagfilter option.
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
      simplifiedAutoLink: gfm,
      literalMidWordUnderscores: true,
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
  if (engine === "ox-content-napi") {
    const { parseAndRender } = await import("@ox-content/napi");
    const options = {
      gfm: false,
      mdx: false,
      footnotes: false,
      tables: gfm,
      strikethrough: gfm,
      taskLists: gfm,
      autolinks: gfm,
      superscript: false,
      subscript: false,
      smartPunctuation: false,
      math: false,
      definitionLists: false,
      headingAttributes: false,
      wikiLinks: false,
    };
    return (source) => parseAndRender(source, options).html;
  }
  // These public APIs already use their closest available spec configuration.
  return createNodeRender(engine, gfm);
}
