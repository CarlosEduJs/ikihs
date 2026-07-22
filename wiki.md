# Ikihs

This wiki describes the architecture, design decisions, components, and development process of Ikihs.
It contains detailed technical documentation for contributors and developers.

## What Is Ikihs

Ikihs is a syntax highlight engine in Rust.
It gives a modern and efficient alternative to Shiki.
It keeps a high level of compatibility with the Shiki ecosystem.

Ikihs preserves the developer experience of Shiki.
It supports VS Code themes and TextMate grammars.
It solves common problems such as startup cost, runtime performance, memory use, and extensibility.

Ikihs does not recreate the ecosystem.
It builds a faster and more flexible engine underneath it.

## Primary Target (v1)

Ikihs v1 is optimized for **build-time use**.
This includes static site generators, CLI tools, and batch pipelines.
In these pipelines, the engine highlights a full document or many documents at once.

This choice affects several downstream decisions:

- Semantic tokens
- Incremental parsing
- Allocation strategy

Editor and IDE integration are **secondary targets**.
They are deferred to v2+.
Browser and WASM runtime use are also secondary targets.

## Design Principles

- **Compatibility comes first.** Existing VS Code themes, TextMate grammars, and Shiki workflows must keep working with little or no change.
- **Performance is a feature.** Startup time, throughput, and memory use are first-class concerns.
- **APIs are simple by default and powerful when needed.** Common use cases need minimal setup. Advanced scenarios (custom scope mapping, theme overrides, alternate engines) stay available through opt-in configuration.
- **The engine is swappable.** The public API does not assume Syntect. Internals are behind an abstraction. The engine can evolve without changes to the user code.
- **Ship v1 narrow and document v2 honestly.** Features that do not fit the build-time target are named as future work.

## Non-Goals (v1)

To keep the scope bounded, these items are **out of scope** for v1:

- Incremental re-highlight of partial documents (batch and full-document only)
- Live semantic token generation
- Runtime integration with editors, Tauri, Leptos, or Axum request paths
- A browser runtime target (WASM — see [ikihsjs](#ikihsjs-npm-package) for Node)
- Guaranteed zero-allocation execution

These may become v2+ goals after the v1 foundation is stable.

## Theme Compatibility

### Main Goal

Make 90–95% of VS Code and Shiki themes work in Ikihs with a near-identical appearance.

### Secondary Goal

Let users reuse the same theme they already use in VS Code, without adjustment.

### Layer 1: Direct Support for Existing Formats

**`.tmTheme` (classic TextMate and legacy Shiki format)**

- The parser is built on top of Syntect.

**VS Code themes (`theme.json` + `package.json`)**

- The parser reads `theme.json`, the most common format.
- It supports `colors`, `tokenColors`, `semanticHighlighting`, and `semanticTokenColors`.
- In v1, the engine parses and stores `semanticTokenColors`. Live semantic token generation is not in scope (see Layer 3).

**Shiki built-in themes** (`dark-plus`, `nord`, `github-dark`, and others)

- The most popular themes are pre-converted and embedded in the package.

### Layer 2: Scope Mapping

This layer does two tasks:

- It converts TextMate scopes into the internal Ikihs category representation.
- It is the abstraction boundary between the public API and the engine.

The layer has an updatable conversion table.
For example, the engine maps `source.ts` to `keyword.function.ts`.
When an exact match does not exist, the engine uses a fallback to the most generic scope.

Consumers never see raw Syntect output.
They see Ikihs internal categories.
This makes the engine swappable (see [Engine Strategy](#engine-strategy)).

### Layer 3: Semantic Highlighting (Injection, Not Generation)

A grammar-based engine (TextMate or Syntect) can only match lexical structure by pattern.
It cannot tell if `x` is a variable, a type, or a function.
This needs semantic analysis from a language server.

**v1 scope:** TextMate scopes only.
The engine parses and stores `semanticTokenColors` from the theme.
But there is no built-in mechanism to generate the tokens.

**v2+ direction:** Give an injection API.
If a consumer has already generated semantic tokens offline (for example, through `rust-analyzer` or `tsserver`), Ikihs accepts them as input.
It applies the theme `semanticTokenColors` on top of the TextMate result.
This fits the build-time model: generate once, cache, and reuse.

This is a change from earlier drafts.
Ikihs does not generate semantic tokens from the theme alone.

### Layer 4: Variables and Customization

- Support for VS Code theme variables
- Support for a `semanticClass` and CSS class output mode
- Theme overrides: users can change a subset of colors without a new theme definition

## Engine Strategy

Syntect is the v1 highlight engine.
It is mature and TextMate-compatible.
It helps Ikihs reach theme and grammar compatibility fast.

Two known tensions exist with the long-term goals:

- **Allocation.** Syntect uses `oniguruma` for regex matching.
  This allocates memory on the hot path.
  This conflicts with a zero-alloc mode.
  In v1, the goal is "low allocation" (reuse buffers and minimize allocation per line), not a guarantee.
- **Incremental parsing.** Syntect reuses line-level state to avoid reprocessing the full file.
  This is not the same as surgical sub-line re-parsing (for example, tree-sitter).
  This gap does not block v1 because v1 targets batch and build-time use.

Because the public API is behind the scope mapper (Layer 2), the engine can be replaced or supplemented later.
For example, tree-sitter can be added for a near-semantic mode.
The crate `ikihs-themes` keeps theme and scope logic separate from the engine.

## ikihsjs (npm package)

Most Shiki users reach it through npm, not through a Rust toolchain.
Astro, VitePress, Next.js, Eleventy, and rehype or remark pipelines are build-time tools.
They process a full document ahead of time, even though the tool is in JS.
For this reason, `ikihsjs` is **not** a v2 concern.
It is a v1 distribution channel for the same build-time use case.

**v1 scope:** `ikihsjs` ships as a native Node addon through `napi-rs`.
It binds directly to `ikihs-core` and `ikihs-engine-syntect`.
This gives native performance without WASM overhead.
It keeps the "no browser target in v1" boundary intact.

**v2+ direction:** A `wasm-bindgen` or `wasm-pack` build of the same core.
This is for consumers who need highlight to run in the browser.
This is the actual browser and WASM non-goal mentioned above.

The npm package needs prebuilt binaries per platform and architecture.
This is standard for `napi-rs` packages.
The package uses `@ikihs/core-{platform}-{arch}` optional dependencies.
The JS API must mirror the Rust API closely.

## Roadmap Shape

| v1 (build-time)                                                      | v2+ (editor and runtime-oriented)                                        |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `.tmTheme` and VS Code `theme.json` parsing                          | —                                                                        |
| Scope mapper (Layer 2)                                               | Alternate or additional engine (for example, tree-sitter)                |
| Syntect-based highlight                                              | —                                                                        |
| Low-allocation tuning (best-effort)                                  | Guaranteed zero-alloc mode                                               |
| Styled-span and CSS-class output modes                               | —                                                                        |
| Compiled and embedded themes (Rust structs)                          | —                                                                        |
| `ikihs highlight` CLI                                                | —                                                                        |
| Zola and mdBook integration                                          | Tauri, Leptos, and Axum runtime integration                              |
| `ikihsjs` npm package (Node, through napi-rs)                        | `ikihsjs` browser build (WASM, through wasm-bindgen)                     |
| —                                                                    | Incremental highlight                                                    |
| —                                                                    | Semantic token injection API                                             |

## Crate Structure

- `ikihs-core` — scope mapping, internal categories, engine trait and abstraction
- `ikihs-themes` — `.tmTheme` and `theme.json` parsing, built-in theme registry, theme overrides
- `ikihs-engine-syntect` — v1 engine implementation behind the `ikihs-core` trait
- `ikihs-cli` — the `ikihs highlight` binary
- `ikihsjs` — `napi-rs` bindings over `ikihs-core`, published to npm with per-platform prebuilt binaries

This structure lets `ikihs-engine-*` alternatives appear later without changes to theme or scope-mapping code.
It also keeps `ikihsjs` as a thin binding layer, not a second implementation.
