import { remark } from "remark";
import remarkGfm from "remark-gfm";
import { normalizeMdast } from "./mdast-reference.mjs";

export async function runPluginReference({
  source,
  plugin,
  options = {},
  parserPlugins = ["remark-gfm"],
}) {
  const processor = remark();

  for (const parserPlugin of parserPlugins) {
    if (parserPlugin === "remark-gfm") {
      processor.use(remarkGfm);
    } else {
      throw new Error("Unsupported reference parser plugin: " + parserPlugin);
    }
  }

  processor.use(plugin, options);
  const parsed = processor.parse(source);
  const parsedTree = normalizeMdast(parsed);
  const transformed = await processor.run(parsed);

  return {
    parsedTree,
    transformedTree: normalizeMdast(transformed),
    markdown: String(processor.stringify(transformed)),
  };
}
