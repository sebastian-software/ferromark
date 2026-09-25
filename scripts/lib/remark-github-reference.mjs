import { remark } from "remark";
import remarkGfm from "remark-gfm";
import remarkGithub from "remark-github";

export function normalizeMdast(value) {
  if (Array.isArray(value)) return value.map(normalizeMdast);

  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .filter(([key]) => key !== "position")
        .map(([key, child]) => [key, normalizeMdast(child)]),
    );
  }

  return value;
}

export async function runRemarkGithubReference(source, repository, parserPlugins) {
  const processor = remark();
  for (const plugin of parserPlugins) {
    if (plugin === "remark-gfm") {
      processor.use(remarkGfm);
    } else {
      throw new Error(`Unsupported reference parser plugin: ${plugin}`);
    }
  }
  processor.use(remarkGithub, { repository });
  const parsed = processor.parse(source);
  const parsedTree = normalizeMdast(parsed);
  const transformed = await processor.run(parsed);

  return {
    parsedTree,
    transformedTree: normalizeMdast(transformed),
    markdown: processor.stringify(transformed),
  };
}
