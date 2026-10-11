import type {
  MarkdownProjectionOptions,
  MarkdownProjectionResult,
  PreparedJsxDocument,
} from "../index.mjs";

export function checkProjection(prepared: PreparedJsxDocument): string {
  const options: MarkdownProjectionOptions = { frontMatter: "omit", omitTitleHeading: "Guide" };
  const projected: MarkdownProjectionResult = prepared.projectMarkdown(options);
  const upper = projected.markdown.toUpperCase();
  for (const diagnostic of projected.diagnostics) {
    const _code: string = diagnostic.code;
    const _start: number = diagnostic.start;
    const _end: number = diagnostic.end;
  }
  const _titleStart: number | undefined = projected.omittedTitleHeadingSpan?.start;
  // @ts-expect-error Projection does not accept parsing choices.
  prepared.projectMarkdown({ format: "mdx" });
  // @ts-expect-error Projection front matter choice is explicit.
  prepared.projectMarkdown({ frontMatter: false });
  return upper;
}
