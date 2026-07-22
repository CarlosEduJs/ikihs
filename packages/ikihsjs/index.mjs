import binding from "./index.js";

class Highlighter {
  constructor(options) {
    this._inner = new binding.JsHighlighter(options);
  }

  codeToTokensBase(code, options) {
    if (typeof options === "string") {
      return this._inner.codeToTokensBase(code, options);
    }
    return this._inner.codeToTokensBase(code, options.lang, options.theme);
  }

  codeToHtml(code, options) {
    let lang, theme;
    if (typeof options === "string") {
      lang = options;
      theme = arguments[2];
    } else {
      lang = options.lang;
      theme = options.theme;
    }
    return this._inner.codeToHtml(code, lang, theme);
  }

  loadTheme(themeJson) {
    return this._inner.loadTheme(JSON.parse(JSON.stringify(themeJson)));
  }

  loadLanguage(lang) {
    return this._inner.loadLanguage(lang);
  }

  getLoadedLanguages() {
    return this._inner.getLoadedLanguages();
  }

  getLoadedThemes() {
    return this._inner.getLoadedThemes();
  }

  getTheme(themeName) {
    return this._inner.getTheme(themeName);
  }
}

export async function createHighlighter(options) {
  return new Highlighter(options);
}

export { Highlighter };
export const JsHighlighter = binding.JsHighlighter;

export default binding;
