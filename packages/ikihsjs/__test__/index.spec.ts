import { describe, it } from "node:test";
import { strict as assert } from "node:assert";
import { createHighlighter, Highlighter } from "../index.mjs";
import type { ThemeRegistration, ThemedToken } from "../shiki-types.d.ts";

const darkPlus: ThemeRegistration = {
  name: "dark-plus",
  type: "dark",
  colors: { "editor.foreground": "#D4D4D4", "editor.background": "#1E1E1E" },
  tokenColors: [
    { scope: "comment", settings: { foreground: "#6A9955" } },
    { scope: "keyword", settings: { foreground: "#569CD6" } },
    { scope: "string", settings: { foreground: "#CE9178" } },
    { scope: "constant.numeric", settings: { foreground: "#B5CEA8" } },
    { scope: "entity.name.function", settings: { foreground: "#DCDCAA" } },
    { scope: "variable", settings: { foreground: "#9CDCFE" } },
    { scope: "storage", settings: { foreground: "#569CD6" } },
  ],
};

const minimal: ThemeRegistration = {
  name: "minimal",
  type: "light",
  colors: { "editor.foreground": "#000000", "editor.background": "#FFFFFF" },
  tokenColors: [{ scope: "comment", settings: { foreground: "#888888" } }],
};

describe("ikihsjs", () => {
  it("creates a highlighter and highlights Rust code", async () => {
    const h = await createHighlighter({
      themes: [darkPlus],
      langs: ["rust"],
    });

    const tokens: ThemedToken[][] = h.codeToTokensBase("fn main() {}", {
      lang: "rust",
      theme: "dark-plus",
    });

    assert.ok(Array.isArray(tokens));
    assert.ok(tokens.length > 0);
    assert.ok(tokens[0]!.length > 0);

    const first: ThemedToken = tokens[0]![0]!;
    assert.equal(first.content, "fn");
    assert.equal(typeof first.offset, "number");
    assert.ok(first.color);
    assert.equal(first.fontStyle, undefined);
    assert.ok(first.scope);
    assert.ok(first.category);
  });

  it("highlights JavaScript code", async () => {
    const h = await createHighlighter({
      themes: [darkPlus],
      langs: ["javascript", "rust"],
    });

    const tokens = h.codeToTokensBase("const x = 42;", { lang: "javascript", theme: "dark-plus" });
    assert.ok(tokens.length > 0);
    assert.ok(tokens[0].length > 0);
    assert.equal(tokens[0][0].content, "const");
  });

  it("returns loaded languages and themes", async () => {
    const h = await createHighlighter({
      themes: [darkPlus],
      langs: ["rust", "javascript"],
    });

    const langs: string[] = h.getLoadedLanguages();
    assert.ok(langs.includes("rust"));
    assert.ok(langs.includes("javascript"));

    const themes: string[] = h.getLoadedThemes();
    assert.ok(themes.includes("dark-plus"));
  });

  it("throws for unknown language", async () => {
    await assert.rejects(async () => {
      await createHighlighter({
        themes: [darkPlus],
        langs: ["nonexistent-lang"],
      });
    });
  });

  it("handles empty source", async () => {
    const h = await createHighlighter({
      themes: [darkPlus],
      langs: ["rust"],
    });

    const tokens = h.codeToTokensBase("", { lang: "rust", theme: "dark-plus" });
    assert.ok(Array.isArray(tokens));
    assert.equal(tokens.length, 0);
  });

  it("loads a theme dynamically", async () => {
    const h = await createHighlighter({
      themes: [darkPlus],
      langs: ["rust"],
    });

    h.loadTheme(minimal);

    const themes = h.getLoadedThemes();
    assert.ok(themes.includes("dark-plus"));
    assert.ok(themes.includes("minimal"));

    const tokens = h.codeToTokensBase("// comment", { lang: "rust", theme: "minimal" });
    assert.ok(tokens[0].length > 0);
    assert.equal(tokens[0][0].color, "#888888");
  });

  it("loads a language dynamically", async () => {
    const h = await createHighlighter({
      themes: [darkPlus],
      langs: ["rust"],
    });

    h.loadLanguage("python");

    const langs = h.getLoadedLanguages();
    assert.ok(langs.includes("rust"));
    assert.ok(langs.includes("python"));

    const tokens = h.codeToTokensBase("# python comment", { lang: "python", theme: "dark-plus" });
    assert.ok(tokens[0].length > 0);
    assert.equal(tokens[0][0].color, "#6A9955");
  });

  it("produces valid HTML from codeToHtml", async () => {
    const h = await createHighlighter({
      themes: [darkPlus],
      langs: ["rust"],
    });

    const html: string = h.codeToHtml("fn main() {}", { lang: "rust", theme: "dark-plus" });

    assert.ok(typeof html === "string");
    assert.ok(html.length > 0);
    assert.ok(html.includes("<span"));
    assert.ok(html.includes('style="color:'));
    assert.ok(html.includes("</span>"));
    assert.ok(!html.includes("&lt;"));
  });

  it("codeToHtml escapes HTML entities", async () => {
    const h = await createHighlighter({
      themes: [darkPlus],
      langs: ["html"],
    });

    const html: string = h.codeToHtml('<div class="foo">', { lang: "html", theme: "dark-plus" });

    assert.ok(html.includes("&lt;"), "should escape <");
    assert.ok(html.includes("&gt;"), "should escape >");
  });

  it("getTheme returns a theme", async () => {
    const h = await createHighlighter({
      themes: [darkPlus],
      langs: ["rust"],
    });

    const theme = h.getTheme("dark-plus");
    assert.ok(theme);
    assert.equal(theme.name, "dark-plus");
    assert.equal(theme.type, "dark");
  });

  it("Highlighter class is exported", () => {
    assert.ok(typeof Highlighter === "function");
  });
});
