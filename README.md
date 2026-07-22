# Ikihs

<p align="center">
  <img src="assets/logo.svg" width="120" alt="Ikihs logo">
</p>

**Syntax highlight at native speed. No interpreter overhead. Use the same API as Shiki.**

[![Compat: 96%](https://img.shields.io/badge/compat-96%25-2ea44f)](https://github.com/carlosedujs/ikihs/tree/main/crates/ikihs-engine-syntect/fixtures)
![License: MIT](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/Rust-1.85+-de5843)

Ikihs is a syntax highlight engine in Rust.
It reads VS Code themes and TextMate grammars.
It compiles to a native binary or a Node addon.
It uses no WASM and no JS main thread.
It has no cold starts.

## Use in Rust

```rust
use ikihs_engine_syntect::SyntectEngine;
use ikihs_themes::vscode_theme::VscodeThemeParser;

let theme = VscodeThemeParser::parse_json(&theme_json)?;
let engine = SyntectEngine::new();
let result = engine.highlight("fn main() {}", "Rust", &theme)?;
```

## Use in JavaScript

```javascript
import { createHighlighter } from "ikihsjs";

const h = await createHighlighter({ themes: [darkPlus], langs: ["rust"] });
const tokens = h.codeToTokensBase("fn main() {}", { lang: "rust", theme: "dark-plus" });
```

## Compatibility

Ikihs compares each byte position against Shiki v4.
A color mismatch counts as a failure.
The current score is 96%.

| Language         | Score |
| ---------------- | ----- |
| Python decorator | 100%  |
| Rust hello world | 98%   |
| Rust comments    | 98%   |
| JS functions     | 96%   |
| Python functions | 94%   |
| JS classes       | 90%   |

The remaining 4% are grammar differences between Sublime Text grammars (Syntect) and VS Code grammars (Shiki).
For example, `constructor` becomes `entity.name.function`.
The variable `x` in `let x` has no scope.
These differences are known and tracked.

## Current status

Version 0.1 is a proof of concept.
It is not ready for production.

**What works:**

- Parse VS Code theme JSON with scope selector matching
- Use 75 bundled TextMate grammars through Syntect
- Classify scopes into 12 categories (keyword, string, comment, function, variable, and others)
- Use the CLI command `ikihs highlight`
- Install the npm package `ikihsjs` through napi-rs
- Compare fixtures against Shiki output

**Not yet implemented:**

- `loadLanguage` and `loadTheme` (dynamic loading)
- Production error handling
- WASM build for browser

## Architecture

Ikihs has three layers:

1. **Theme parser** — read VS Code `theme.json` and TextMate `.tmTheme` files into a neutral format.
2. **Engine** — tokenize source code with TextMate grammars and apply colors from the theme.
3. **Scope mapper** — convert TextMate scopes into 12 Ikihs categories.

The engine is behind a trait.
You can replace it with a different engine.

Detailed design decisions are in [wiki.md](./wiki.md).

## Repository structure

```
crates/
  ikihs-core/           Types, traits, scope mapper
  ikihs-themes/         Theme parsers (VS Code JSON, TextMate XML)
  ikihs-engine-syntect/ Syntect-based highlight engine
apps/
  ikihs-cli/            CLI binary
packages/
  ikihsjs/              npm package (napi-rs)
  @ikihs/compare        Fixture comparison tool
```

## License

MIT
