import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { test } from "node:test";
import { generatedEmojiData } from "./generate-gemoji-data.mjs";

test("the bundled emoji aliases reproduce the pinned gemoji dataset", async () => {
  const repositoryRoot = join(new URL("..", import.meta.url).pathname);
  const bundled = await readFile(join(repositoryRoot, "transforms/src/emoji_data.rs"), "utf8");
  assert.equal(bundled, await generatedEmojiData());
});
