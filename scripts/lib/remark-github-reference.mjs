import remarkGithub from "remark-github";
import { normalizeMdast } from "./mdast-reference.mjs";
import { runPluginReference } from "./plugin-reference-runner.mjs";

export { normalizeMdast };

export function runRemarkGithubReference(source, repository, parserPlugins = ["remark-gfm"]) {
  return runPluginReference({
    source,
    plugin: remarkGithub,
    options: { repository },
    parserPlugins,
  });
}
