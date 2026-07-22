// Shiki-compatible types for the ikihsjs Highlighter wrapper

export interface TokenSettings {
  foreground?: string;
  background?: string;
  fontStyle?: string;
}

export interface TokenColor {
  scope?: string | string[];
  settings: TokenSettings;
}

export interface ThemeRegistration {
  name?: string;
  type?: "light" | "dark";
  colors?: Record<string, string>;
  tokenColors?: TokenColor[];
}

export interface ThemedToken {
  content: string;
  offset: number;
  color?: string;
  fontStyle?: number;
  scope: string;
  category: string;
}

export interface CodeToTokensOptions {
  lang: string;
  theme: string;
}

export interface CodeToHtmlOptions {
  lang: string;
  theme: string;
}

export interface HighlighterOptions {
  themes: ThemeRegistration[];
  langs: string[];
}

export declare class Highlighter {
  constructor(options: HighlighterOptions);
  codeToTokensBase(code: string, options: CodeToTokensOptions): ThemedToken[][];
  codeToHtml(code: string, options: CodeToHtmlOptions): string;
  loadTheme(theme: ThemeRegistration): void;
  loadLanguage(lang: string): void;
  getLoadedLanguages(): string[];
  getLoadedThemes(): string[];
  getTheme(themeName: string): ThemeRegistration;
}

export declare function createHighlighter(options: HighlighterOptions): Promise<Highlighter>;
