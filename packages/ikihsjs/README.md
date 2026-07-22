# ikihsjs

<p align="center">
  <img src="../../assets/logo.svg" width="100" alt="Ikihs logo">
</p>

**Native syntax highlight for Node.js. A drop-in API for Shiki users.**

```bash
npm install ikihsjs
```

```javascript
// Before: Shiki
import { createHighlighter } from "shiki";
const h = await createHighlighter({ themes: [darkPlus], langs: ["rust"] });
const tokens = h.codeToTokensBase(code, { lang: "rust", theme: "dark-plus" });

// After: ikihsjs
import { createHighlighter } from "ikihsjs";
const h = await createHighlighter({ themes: [darkPlus], langs: ["rust"] });
const tokens = h.codeToTokensBase(code, { lang: "rust", theme: "dark-plus" });
```

Themes, grammars, and the API are the same as Shiki.
`codeToHtml`, `getLoadedLanguages`, `getLoadedThemes`, `loadTheme`, `loadLanguage`, and `getTheme` work as expected.

## Differences from Shiki

- **Node.js only** in v1. A browser build (WASM) is planned for v2.
- **Prebuilt binary** per platform (napi-rs). No WASM startup cost. No interpreter overhead.
- Grammar output differs on some edge cases. See the [compatibility table](https://github.com/carlosedujs/ikihs#compatibility) for details.

## API

```typescript
import { createHighlighter, Highlighter } from "ikihsjs";
import type { ThemeRegistration, ThemedToken } from "ikihsjs/shiki-types";

const h: Highlighter = await createHighlighter({
  themes: [theme],          // ThemeRegistration[]
  langs: ["rust", "js"],     // string[]
});

// Tokenize code
const tokens: ThemedToken[][] = h.codeToTokensBase(source, { lang, theme });
// Each token: { content, offset, color?, fontStyle?, scope, category }

// Get HTML
const html: string = h.codeToHtml(source, { lang, theme });

// Load resources dynamically
h.loadLanguage("python");
h.loadTheme(myTheme);
h.getLoadedLanguages();   // string[]
h.getLoadedThemes();      // string[]
h.getTheme("dark-plus");  // ThemeRegistration
```

## How it works

`ikihsjs` is a native Node addon built with [napi-rs](https://napi.rs).
The crate compiles the Ikihs Rust engine into a `.node` binary.
Node loads this binary directly.
There is no child process, no WASM, and no JS fallback.
