# Ikihs

<p align="center">
  <img src="assets/logo.svg" width="120" alt="Ikihs logo">
</p>

**Syntax highlight at native speed. No interpreter overhead. Use the same API as Shiki.**

[![Compat: 100% TS/JS · 96% overall](https://img.shields.io/badge/TS%2FJS-100%25%20·%2096%25%20overall-2ea44f)](https://github.com/CarlosEduJs/ikihs)
![License: MIT](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/Rust-1.85+-de5843)

Ikihs is a syntax highlight engine in Rust.
It reads VS Code themes and parses code with **tree-sitter** (TypeScript, JavaScript) or **Syntect** (everything else).
It compiles to a native binary or a Node addon.
It uses no WASM and no JS main thread.
It has no cold starts.

## Use in Rust

```rust
use ikihs_engine_composite::CompositeEngine;
use ikihs_themes::vscode_theme::VscodeThemeParser;

let theme = VscodeThemeParser::parse_json(&theme_json)?;
let engine = CompositeEngine::new();
let result = engine.highlight("fn main() {}", "Rust", &theme)?;
```

## Use in JavaScript

```javascript
import { createHighlighter } from "ikihsjs";

const h = await createHighlighter({ themes: [darkPlus], langs: ["rust"] });
const tokens = h.codeToTokensBase("fn main() {}", { lang: "rust", theme: "dark-plus" });
```

## Compatibility

Ikihs compares each byte position against Shiki.
A color mismatch counts as a failure.

**Tree-sitter engine** matches Shiki **100%** for TypeScript and JavaScript — every byte, every color.

| Language           | Engine     | Score |
| ------------------ | ---------- | ----- |
| TypeScript (types) | tree-sitter | 100% |
| TypeScript (generics) | tree-sitter | 100% |
| JavaScript (functions) | tree-sitter | 100% |
| JavaScript (classes) | tree-sitter | 100% |

**Syntect engine** handles the remaining 30+ languages and averages ~96%.
Most mismatches are harmless TextMate grammar differences (e.g. `constructor` scoped as `entity.name.function` instead of `keyword.constructor`).

## How it works

```
┌──────────────────────────────────────────────┐
│                CompositeEngine                │
│  ┌──────────────┐  ┌──────────────────────┐   │
│  │ TreeSitterEngine  │  SyntectEngine        │   │
│  │ (TS, JS, JSX) │  │ (all other langs)    │   │
│  └──────────────┘  └──────────────────────┘   │
│  Routes by language at highlight time          │
└──────────────────────────────────────────────┘
```

The tree-sitter engine walks the CST, maps each node to a scope string,
then resolves the final color via **scope-prefix matching** against the
VS Code theme — the same strategy as Shiki internally.

## Current status

Version 0.2 introduces the tree-sitter engine.
TypeScript and JavaScript output is byte-identical to Shiki.

**What works:**

- Parse VS Code theme JSON with scope-prefix matching
- Tree-sitter highlighting for TypeScript / TSX / JavaScript / JSX
- Syntect highlighting for 75+ other languages
- Classify scopes into 12 categories (keyword, string, comment, function, variable, and others)
- Use the CLI command `ikihs highlight`
- Install the npm package `ikihsjs` through napi-rs
- Compare fixtures against Shiki output

**Planned (next PRs):**

- Python, Rust, and more languages via tree-sitter
- JSX fixture tests
- Reduce syntect color diffs for remaining languages
- WASM build for browser

## Repository structure

```
crates/
  ikihs-core/               Types, traits, scope primitive
  ikihs-themes/             Theme parsers (VS Code JSON, TextMate XML)
  ikihs-engine-syntect/     Syntect-based highlight engine (legacy)
  ikihs-engine-treesitter/  Tree-sitter engine (TS, JS, JSX)
  ikihs-engine-composite/   Router: tree-sitter for TS/JS, syntect for rest
apps/
  ikihs-cli/                CLI binary
packages/
  ikihsjs/                  npm package (napi-rs)
  @ikihs/compare            Fixture comparison tool
```

## License

MIT
