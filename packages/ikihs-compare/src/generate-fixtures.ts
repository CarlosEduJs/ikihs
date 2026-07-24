import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { resolve, relative, extname, join, basename } from "node:path";
import { highlightShiki, getHighlighter } from "./shiki.js";

const FIXTURE_DIR = resolve(import.meta.dirname, "../../../crates/ikihs-engine-syntect/fixtures");

const SKIP_FILES = new Set(["dark-plus.json"]);

const LANG_MAP: Record<string, string> = {
  rs: "rust",
  js: "javascript",
  ts: "typescript",
  py: "python",
  css: "css",
  html: "html",
  json: "json",
  md: "markdown",
  yaml: "yaml",
  sh: "bash",
};

function findSources(dir: string): string[] {
  const results: string[] = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      results.push(...findSources(full));
    } else {
      const ext = extname(full).slice(1);
      if (
        LANG_MAP[ext] &&
        !full.endsWith(".shiki.json") &&
        !full.endsWith(".expected.json") &&
        !SKIP_FILES.has(basename(full))
      ) {
        results.push(full);
      }
    }
  }
  return results;
}

async function main() {
  await getHighlighter();

  const sources = findSources(FIXTURE_DIR);
  console.log(`Found ${sources.length} fixture source(s)\n`);

  for (const sourcePath of sources) {
    const ext = extname(sourcePath).slice(1);
    const lang = LANG_MAP[ext]!;
    const source = readFileSync(sourcePath, "utf-8");
    const outPath = sourcePath.replace(/\.\w+$/, ".shiki.json");

    console.log(`  ${relative(FIXTURE_DIR, sourcePath)} (${lang})`);

    const result = await highlightShiki(source, lang, "dark-plus");

    const simplified = {
      language: result.language,
      tokens: result.tokens.map((line) =>
        line.map((t) => ({
          content: t.content,
          offset: t.offset,
          color: t.color,
        })),
      ),
    };

    writeFileSync(outPath, JSON.stringify(simplified, null, 2) + "\n");
  }

  console.log(`\nDone. Generated ${sources.length} fixture(s).`);
}

main().catch(console.error);
