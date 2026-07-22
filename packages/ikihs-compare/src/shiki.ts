import { createHighlighter } from "shiki";
import type { ShikiResult } from "./types.js";

let highlighter: Awaited<ReturnType<typeof createHighlighter>> | null = null;

export async function getHighlighter() {
  if (!highlighter) {
    highlighter = await createHighlighter({
      langs: [
        "rust",
        "javascript",
        "typescript",
        "python",
        "jsx",
        "tsx",
        "css",
        "html",
        "json",
        "yaml",
        "markdown",
        "bash",
        "shell",
      ],
      themes: ["dark-plus", "light-plus"],
    });
  }
  return highlighter;
}

export async function highlightShiki(
  source: string,
  lang: string,
  themeName: string = "dark-plus",
): Promise<ShikiResult> {
  const h = await getHighlighter();
  const result = h.codeToTokens(source, {
    lang: lang as never,
    theme: themeName,
  });

  return {
    tokens: result.tokens.map((line) =>
      line.map((t) => ({
        content: t.content,
        offset: t.offset,
        color: t.color ?? "#000000",
        fontStyle: t.fontStyle ?? 0,
      })),
    ),
    fg: result.fg ?? "#D4D4D4",
    bg: result.bg ?? "#1E1E1E",
    language: lang,
  };
}
